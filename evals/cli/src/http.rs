//! The `http` eval kind: one request against the deployed site, checked for
//! its status, a header, a piece of the body and how long it took. Runs at
//! stage `cd`, right after a deploy (standards/docs/evals.md).

use std::time::{Duration, Instant};

use evals_model::{Kind, ResultStatus};

/// The largest body read, so a wrong URL can't fill the runner's memory.
const MAX_BODY: u64 = 20 * 1024 * 1024;

/// Every request gives up after this long.
const TIMEOUT: Duration = Duration::from_secs(30);

/// Sent with every request, so these checks can be told apart from people
/// in the site's analytics.
pub const USER_AGENT: &str = "jimhillr-evals/0.1 (synthetic check)";

/// One response, as much of it as the checks need.
#[derive(Debug, Clone, Default)]
pub struct Response {
    pub status: u16,
    /// Header names in lower case.
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub elapsed_ms: u64,
}

/// Sends requests. A trait so tests can answer without a network.
pub trait Fetch {
    /// Sends one request. `follow_redirects` is false when the eval expects
    /// a redirect itself. An `Err` means no response came back at all.
    fn fetch(&self, method: &str, url: &str, follow_redirects: bool) -> Result<Response, String>;
}

/// The real client: ureq, with HTTP error statuses returned as responses.
pub struct Client {
    follow: ureq::Agent,
    stay: ureq::Agent,
}

impl Client {
    pub fn new() -> Self {
        let agent = |redirects: u32| {
            ureq::Agent::config_builder()
                .http_status_as_error(false)
                .max_redirects(redirects)
                .max_redirects_will_error(false)
                .timeout_global(Some(TIMEOUT))
                .user_agent(USER_AGENT)
                .build()
                .new_agent()
        };
        Self {
            follow: agent(10),
            stay: agent(0),
        }
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

impl Fetch for Client {
    fn fetch(&self, method: &str, url: &str, follow_redirects: bool) -> Result<Response, String> {
        let agent = if follow_redirects {
            &self.follow
        } else {
            &self.stay
        };
        let request = ureq::http::Request::builder()
            .method(method)
            .uri(url)
            .body(())
            .map_err(|e| format!("bad request {method} {url}: {e}"))?;
        let start = Instant::now();
        let mut response = agent
            .run(request)
            .map_err(|e| format!("{method} {url}: {e}"))?;
        let status = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .map(|(name, value)| {
                (
                    name.as_str().to_ascii_lowercase(),
                    String::from_utf8_lossy(value.as_bytes()).into_owned(),
                )
            })
            .collect();
        let body = response
            .body_mut()
            .with_config()
            .limit(MAX_BODY)
            .read_to_vec()
            .map_err(|e| format!("{method} {url}: reading the body: {e}"))?;
        let elapsed_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
        Ok(Response {
            status,
            headers,
            body,
            elapsed_ms,
        })
    }
}

/// What an `http` eval expects, with the defaults filled in.
#[derive(Debug, Clone, PartialEq)]
pub struct Expect {
    pub method: String,
    pub url: String,
    pub status: u16,
    /// Name in lower case, and the text its value must contain.
    pub header: Option<(String, String)>,
    pub body_contains: Option<String>,
    pub max_ms: Option<u64>,
}

impl Expect {
    /// The request an `http` eval makes. `Err` says why it can't be made:
    /// the eval isn't an `http` one, or it has a `path` and no deploy URL
    /// was given.
    pub fn from_kind(kind: &Kind, deploy_url: Option<&str>) -> Result<Self, String> {
        let Kind::Http {
            url,
            path,
            method,
            status,
            header,
            body_contains,
            max_ms,
        } = kind
        else {
            return Err(format!("not an http eval: {}", kind.name()));
        };
        let url = match (url, path) {
            (Some(url), _) => url.clone(),
            (None, Some(path)) => match deploy_url {
                Some(base) => format!("{}{path}", base.trim_end_matches('/')),
                None => return Err(format!("no DEPLOY_URL to join `{path}` to")),
            },
            (None, None) => return Err("an http eval needs url or path".into()),
        };
        let header = header.as_ref().and_then(|h| {
            h.split_once(':')
                .map(|(name, value)| (name.trim().to_ascii_lowercase(), value.trim().to_string()))
        });
        Ok(Self {
            method: method.clone().unwrap_or_else(|| "GET".into()),
            url,
            status: status.unwrap_or(200),
            header,
            body_contains: body_contains.clone(),
            max_ms: *max_ms,
        })
    }

    /// The eval expects a redirect, so the client must not follow it.
    pub fn follows_redirects(&self) -> bool {
        !(300..400).contains(&self.status)
    }

    /// `max_ms` as a result's threshold.
    pub fn threshold(&self) -> Option<String> {
        self.max_ms.map(|ms| format!("<= {ms}"))
    }

    /// Checks one response: `Pass`, or `Fail` with every check that missed.
    pub fn judge(&self, response: &Response) -> (ResultStatus, String) {
        let mut misses = Vec::new();
        if response.status != self.status {
            misses.push(format!(
                "status {} (expected {})",
                response.status, self.status
            ));
        }
        if let Some((name, want)) = &self.header {
            let values: Vec<&str> = response
                .headers
                .iter()
                .filter(|(n, _)| n == name)
                .map(|(_, v)| v.as_str())
                .collect();
            let want_lower = want.to_ascii_lowercase();
            if values.is_empty() {
                misses.push(format!("no `{name}` header"));
            } else if !values
                .iter()
                .any(|v| v.to_ascii_lowercase().contains(&want_lower))
            {
                misses.push(format!(
                    "`{name}: {}` (expected it to contain `{want}`)",
                    values.join(", ")
                ));
            }
        }
        if let Some(needle) = &self.body_contains
            && !contains(&response.body, needle.as_bytes())
        {
            misses.push(format!(
                "body ({} bytes) doesn't contain `{needle}`",
                response.body.len()
            ));
        }
        if let Some(max) = self.max_ms
            && response.elapsed_ms > max
        {
            misses.push(format!("took {} ms (limit {max} ms)", response.elapsed_ms));
        }
        let what = format!("{} {}", self.method, self.url);
        if misses.is_empty() {
            (
                ResultStatus::Pass,
                format!("{what}: {} in {} ms", response.status, response.elapsed_ms),
            )
        } else {
            (ResultStatus::Fail, format!("{what}: {}", misses.join("; ")))
        }
    }
}

/// How hard to try before calling an eval failed. A fresh deploy can take a
/// few seconds to reach every edge, so a miss is retried.
#[derive(Debug, Clone, Copy)]
pub struct Retry {
    pub attempts: u32,
    pub delay: Duration,
}

impl Default for Retry {
    fn default() -> Self {
        Self {
            attempts: 3,
            delay: Duration::from_secs(5),
        }
    }
}

/// The outcome of one `http` eval.
#[derive(Debug, Clone, PartialEq)]
pub struct Outcome {
    pub status: ResultStatus,
    pub note: String,
    /// The last response's time in milliseconds, when one came back.
    pub value: Option<f64>,
    pub threshold: Option<String>,
}

/// Runs one `http` eval, retrying a miss up to `retry.attempts` times. A
/// site that never answers fails: from outside, a down site and a broken
/// one look the same. An eval that can't be run at all (no deploy URL)
/// is an `error`.
pub fn run(kind: &Kind, deploy_url: Option<&str>, fetch: &dyn Fetch, retry: Retry) -> Outcome {
    let expect = match Expect::from_kind(kind, deploy_url) {
        Ok(expect) => expect,
        Err(why) => {
            return Outcome {
                status: ResultStatus::Error,
                note: why,
                value: None,
                threshold: None,
            };
        }
    };
    let attempts = retry.attempts.max(1);
    let mut last = None;
    for attempt in 1..=attempts {
        if attempt > 1 {
            std::thread::sleep(retry.delay);
        }
        let (status, mut note, value) =
            match fetch.fetch(&expect.method, &expect.url, expect.follows_redirects()) {
                Ok(response) => {
                    let (status, note) = expect.judge(&response);
                    #[allow(clippy::cast_precision_loss)]
                    let ms = response.elapsed_ms as f64;
                    (status, note, Some(ms))
                }
                Err(why) => (ResultStatus::Fail, format!("no response: {why}"), None),
            };
        if attempt > 1 {
            note.push_str(&format!(" (attempt {attempt} of {attempts})"));
        }
        let done = status == ResultStatus::Pass;
        last = Some(Outcome {
            status,
            note,
            value,
            threshold: expect.threshold(),
        });
        if done {
            break;
        }
    }
    last.unwrap_or(Outcome {
        status: ResultStatus::Error,
        note: "no attempt was made".into(),
        value: None,
        threshold: None,
    })
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    needle.is_empty() || haystack.windows(needle.len()).any(|w| w == needle)
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpListener;

    use super::*;

    fn http(fields: &str) -> Kind {
        let text = format!(
            "prefix = \"COOK\"\n[[eval]]\nid = \"e\"\nrequirement = \"COOK-001\"\nstages = [\"cd\"]\nkind = \"http\"\n{fields}"
        );
        match evals_model::parse_evals(&text) {
            Ok(mut file) => file.evals.remove(0).kind,
            Err(errors) => panic!("{errors:?}"),
        }
    }

    fn ok_page(body: &str) -> Response {
        Response {
            status: 200,
            headers: vec![("content-type".into(), "text/html; charset=utf-8".into())],
            body: body.as_bytes().to_vec(),
            elapsed_ms: 120,
        }
    }

    /// Answers each request with the next canned response, and records it.
    struct Canned {
        answers: RefCell<Vec<Result<Response, String>>>,
        asked: RefCell<Vec<(String, String, bool)>>,
    }

    impl Canned {
        fn new(answers: Vec<Result<Response, String>>) -> Self {
            Self {
                answers: RefCell::new(answers),
                asked: RefCell::new(Vec::new()),
            }
        }
    }

    impl Fetch for Canned {
        fn fetch(&self, method: &str, url: &str, follow: bool) -> Result<Response, String> {
            self.asked
                .borrow_mut()
                .push((method.into(), url.into(), follow));
            let mut answers = self.answers.borrow_mut();
            if answers.is_empty() {
                Err("no more answers".into())
            } else {
                answers.remove(0)
            }
        }
    }

    const NOW: Retry = Retry {
        attempts: 3,
        delay: Duration::ZERO,
    };

    #[test]
    fn a_path_is_joined_to_the_deploy_url() -> Result<(), String> {
        let expect = Expect::from_kind(
            &http("path = \"/recipe/1\""),
            Some("https://jimhillr-cookbook.pages.dev/"),
        )?;
        assert_eq!(expect.url, "https://jimhillr-cookbook.pages.dev/recipe/1");
        assert_eq!(expect.method, "GET");
        assert_eq!(expect.status, 200);
        assert!(expect.follows_redirects());

        let expect = Expect::from_kind(&http("url = \"https://example.com/x\""), None)?;
        assert_eq!(expect.url, "https://example.com/x");

        let expect = Expect::from_kind(&http("path = \"/old\"\nstatus = 301"), Some("https://a"))?;
        assert!(!expect.follows_redirects());
        Ok(())
    }

    #[test]
    fn a_path_without_a_deploy_url_is_an_error() {
        let outcome = run(&http("path = \"/\""), None, &Canned::new(vec![]), NOW);
        assert_eq!(outcome.status, ResultStatus::Error);
        assert_eq!(outcome.note, "no DEPLOY_URL to join `/` to");
    }

    #[test]
    fn passes_when_every_check_holds() -> Result<(), String> {
        let expect = Expect::from_kind(
            &http(
                "path = \"/\"\nheader = \"Content-Type: TEXT/HTML\"\nbody_contains = \"Cookbook\"\nmax_ms = 500",
            ),
            Some("https://a"),
        )?;
        assert_eq!(
            expect.judge(&ok_page("<h1>Cookbook</h1>")),
            (ResultStatus::Pass, "GET https://a/: 200 in 120 ms".into())
        );
        assert_eq!(expect.threshold().as_deref(), Some("<= 500"));
        Ok(())
    }

    #[test]
    fn names_every_check_that_missed() -> Result<(), String> {
        let expect = Expect::from_kind(
            &http(
                "path = \"/deck.pdf\"\nheader = \"content-type: application/pdf\"\nbody_contains = \"%PDF\"\nmax_ms = 100",
            ),
            Some("https://a"),
        )?;
        let mut response = ok_page("Not found");
        response.status = 404;
        let (status, note) = expect.judge(&response);
        assert_eq!(status, ResultStatus::Fail);
        assert_eq!(
            note,
            "GET https://a/deck.pdf: status 404 (expected 200); \
             `content-type: text/html; charset=utf-8` (expected it to contain `application/pdf`); \
             body (9 bytes) doesn't contain `%PDF`; took 120 ms (limit 100 ms)"
        );

        response.headers.clear();
        let (_, note) = expect.judge(&response);
        assert!(note.contains("no `content-type` header"), "{note}");
        Ok(())
    }

    #[test]
    fn a_binary_body_is_searched_as_bytes() -> Result<(), String> {
        let expect = Expect::from_kind(
            &http("path = \"/\"\nbody_contains = \"%PDF\""),
            Some("https://a"),
        )?;
        let mut response = ok_page("");
        response.body = b"%PDF-1.7\n\xff\xfe\x00binary".to_vec();
        assert_eq!(expect.judge(&response).0, ResultStatus::Pass);
        Ok(())
    }

    #[test]
    fn a_miss_is_retried_until_it_passes() {
        let mut stale = ok_page("old build");
        stale.status = 404;
        let fetch = Canned::new(vec![
            Err("connection refused".into()),
            Ok(stale),
            Ok(ok_page("Cookbook")),
        ]);
        let outcome = run(
            &http("path = \"/\"\nbody_contains = \"Cookbook\""),
            Some("https://a"),
            &fetch,
            NOW,
        );
        assert_eq!(outcome.status, ResultStatus::Pass);
        assert_eq!(
            outcome.note,
            "GET https://a/: 200 in 120 ms (attempt 3 of 3)"
        );
        assert_eq!(outcome.value, Some(120.0));
        assert_eq!(fetch.asked.borrow().len(), 3);
    }

    #[test]
    fn a_site_that_never_answers_fails() {
        let fetch = Canned::new(vec![Err("timed out".into()), Err("timed out".into())]);
        let retry = Retry {
            attempts: 2,
            delay: Duration::ZERO,
        };
        let outcome = run(&http("path = \"/\""), Some("https://a"), &fetch, retry);
        assert_eq!(outcome.status, ResultStatus::Fail);
        assert_eq!(outcome.note, "no response: timed out (attempt 2 of 2)");
        assert_eq!(outcome.value, None);
    }

    #[test]
    fn a_pass_is_not_retried() {
        let fetch = Canned::new(vec![Ok(ok_page("")), Ok(ok_page(""))]);
        let outcome = run(
            &http("url = \"https://a/\"\nmethod = \"HEAD\""),
            None,
            &fetch,
            NOW,
        );
        assert_eq!(outcome.status, ResultStatus::Pass);
        assert_eq!(
            *fetch.asked.borrow(),
            vec![("HEAD".to_string(), "https://a/".to_string(), true)]
        );
    }

    /// Serves one canned HTTP/1.1 reply per connection on localhost and
    /// returns the base URL.
    fn serve(replies: Vec<&'static str>) -> std::io::Result<String> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let base = format!("http://{}", listener.local_addr()?);
        std::thread::spawn(move || {
            for reply in replies {
                let Ok((mut stream, _)) = listener.accept() else {
                    return;
                };
                let mut reader = BufReader::new(match stream.try_clone() {
                    Ok(s) => s,
                    Err(_) => return,
                });
                let mut line = String::new();
                while reader.read_line(&mut line).is_ok_and(|n| n > 2) {
                    line.clear();
                }
                let _ = stream.write_all(reply.as_bytes());
            }
        });
        Ok(base)
    }

    #[test]
    fn the_client_reads_a_real_response() -> anyhow::Result<()> {
        let base = serve(vec![
            "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nX-Evals: yes\r\nContent-Length: 4\r\nConnection: close\r\n\r\nnope",
        ])?;
        let response = Client::new()
            .fetch("GET", &format!("{base}/missing"), true)
            .map_err(anyhow::Error::msg)?;
        assert_eq!(response.status, 404);
        assert_eq!(response.body, b"nope");
        assert!(response.headers.contains(&("x-evals".into(), "yes".into())));
        Ok(())
    }

    #[test]
    fn the_client_does_not_follow_a_redirect_the_eval_expects() -> anyhow::Result<()> {
        let base = serve(vec![
            "HTTP/1.1 301 Moved Permanently\r\nLocation: /new\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        ])?;
        let outcome = run(
            &http("path = \"/old\"\nstatus = 301\nheader = \"location: /new\""),
            Some(&base),
            &Client::new(),
            NOW,
        );
        assert_eq!(outcome.status, ResultStatus::Pass, "{}", outcome.note);
        Ok(())
    }

    #[test]
    fn the_client_reports_a_refused_connection() -> anyhow::Result<()> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let base = format!("http://{}", listener.local_addr()?);
        drop(listener);
        let retry = Retry {
            attempts: 1,
            delay: Duration::ZERO,
        };
        let outcome = run(&http("path = \"/\""), Some(&base), &Client::new(), retry);
        assert_eq!(outcome.status, ResultStatus::Fail);
        assert!(
            outcome
                .note
                .starts_with("no response: GET http://127.0.0.1:"),
            "{}",
            outcome.note
        );
        Ok(())
    }
}
