//! OpenTofu's `http` state backend protocol (decision 0009), independent of
//! Workers and D1 so it can be tested natively.
//!
//! Each stack has one URL, `/states/<stack>`:
//!
//! - `GET` returns the latest state, or `204` when there is none.
//! - `POST` stores a new version. While the stack is locked, only the lock
//!   holder may write (OpenTofu sends its lock ID as `?ID=`).
//! - `LOCK` takes the lock, or fails with `409` and the holder's lock info.
//! - `UNLOCK` releases it, only for the same lock ID.
//!
//! The read password may only `GET` (PR plans run with `-lock=false`); the
//! write password may do everything. Passwords come as HTTP basic auth
//! (`TF_HTTP_PASSWORD`); the user name is ignored.

use base64::Engine;

/// The largest state body accepted. D1 allows 2 MB a row; ours are a few KB.
pub const MAX_STATE_BYTES: usize = 1_000_000;

/// How many versions of each stack's state are kept (decision 0009, item 3).
pub const KEEP_VERSIONS: u32 = 10;

/// What a password allows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Read,
    Write,
}

/// The two passwords, from the Worker's secrets. An empty password never
/// matches, so a Worker deployed without its secrets refuses everything.
#[derive(Debug, Clone, Copy)]
pub struct Passwords<'a> {
    pub read: &'a str,
    pub write: &'a str,
}

/// One request, as plain data.
#[derive(Debug, Clone, Copy)]
pub struct Request<'a> {
    /// The raw method: OpenTofu uses `LOCK` and `UNLOCK`, which most HTTP
    /// libraries don't have.
    pub method: &'a str,
    pub path: &'a str,
    /// The query string without its `?`.
    pub query: &'a str,
    pub authorization: Option<&'a str>,
    pub body: &'a [u8],
}

/// One response. `body` is empty for `204`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub status: u16,
    pub body: String,
}

impl Response {
    fn new(status: u16, body: impl Into<String>) -> Self {
        Self {
            status,
            body: body.into(),
        }
    }

    fn json(status: u16, body: impl Into<String>) -> Self {
        Self::new(status, body)
    }

    fn error(status: u16, message: &str) -> Self {
        Self::new(status, format!("{message}\n"))
    }
}

/// A store failure. Its message is for the Worker's own log and never holds
/// state contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreError(pub String);

/// Where states and locks live: D1 in the Worker, memory in tests.
#[allow(async_fn_in_trait)]
pub trait Store {
    /// The latest state of a stack.
    async fn state(&self, stack: &str) -> Result<Option<String>, StoreError>;
    /// Add a new version and delete all but the last [`KEEP_VERSIONS`].
    async fn put_state(&self, stack: &str, body: &str) -> Result<(), StoreError>;
    /// Take the lock atomically. `Ok(None)` means it was taken; `Ok(Some(info))`
    /// means someone else holds it, with their lock info.
    async fn lock(&self, stack: &str, id: &str, info: &str) -> Result<Option<String>, StoreError>;
    /// The current lock holder's ID and lock info.
    async fn holder(&self, stack: &str) -> Result<Option<(String, String)>, StoreError>;
    /// Release the lock if `id` holds it.
    async fn unlock(&self, stack: &str, id: &str) -> Result<(), StoreError>;
}

/// Handle one request.
pub async fn handle<S: Store>(store: &S, passwords: Passwords<'_>, req: Request<'_>) -> Response {
    let Some(stack) = stack_from_path(req.path) else {
        return Response::error(404, "not found: state lives at /states/<stack>");
    };
    let Some(role) = role(passwords, req.authorization) else {
        return Response::error(401, "unauthorized");
    };
    let result = match req.method {
        "GET" => get(store, stack).await,
        "POST" | "LOCK" | "UNLOCK" if role != Role::Write => {
            return Response::error(403, "the read password can only GET");
        }
        "POST" => post(store, stack, req).await,
        "LOCK" => lock(store, stack, req.body).await,
        "UNLOCK" => unlock(store, stack, req.body).await,
        _ => return Response::error(405, "method not allowed"),
    };
    result.unwrap_or_else(|_| Response::error(500, "state store error"))
}

async fn get<S: Store>(store: &S, stack: &str) -> Result<Response, StoreError> {
    Ok(match store.state(stack).await? {
        Some(body) => Response::json(200, body),
        None => Response::new(204, ""),
    })
}

async fn post<S: Store>(store: &S, stack: &str, req: Request<'_>) -> Result<Response, StoreError> {
    if req.body.len() > MAX_STATE_BYTES {
        return Ok(Response::error(413, "state too large"));
    }
    let Ok(body) = std::str::from_utf8(req.body) else {
        return Ok(Response::error(400, "state must be UTF-8 JSON"));
    };
    if serde_json::from_str::<serde_json::Value>(body).is_err() {
        return Ok(Response::error(400, "state must be JSON"));
    }
    let id = query_param(req.query, "ID");
    match (store.holder(stack).await?, id) {
        (Some((holder, info)), Some(id)) if holder != id => return Ok(Response::json(409, info)),
        (Some((_, info)), None) => return Ok(Response::json(409, info)),
        (None, Some(_)) => {
            return Ok(Response::error(
                409,
                "the lock this write names is not held",
            ));
        }
        _ => {}
    }
    store.put_state(stack, body).await?;
    Ok(Response::new(200, ""))
}

async fn lock<S: Store>(store: &S, stack: &str, body: &[u8]) -> Result<Response, StoreError> {
    let Some((id, info)) = lock_info(body) else {
        return Ok(Response::error(
            400,
            "LOCK needs OpenTofu's lock info JSON with an ID",
        ));
    };
    Ok(match store.lock(stack, &id, &info).await? {
        None => Response::new(200, ""),
        Some(holder) => Response::json(409, holder),
    })
}

async fn unlock<S: Store>(store: &S, stack: &str, body: &[u8]) -> Result<Response, StoreError> {
    let Some((id, _)) = lock_info(body) else {
        return Ok(Response::error(
            400,
            "UNLOCK needs OpenTofu's lock info JSON with an ID",
        ));
    };
    Ok(match store.holder(stack).await? {
        None => Response::new(200, ""),
        Some((holder, _)) if holder == id => {
            store.unlock(stack, &id).await?;
            Response::new(200, "")
        }
        Some((_, info)) => Response::json(409, info),
    })
}

/// The stack in `/states/<stack>`, if the path is that and the name is
/// lowercase letters, digits, `-` and `_`, at most 64 long.
pub fn stack_from_path(path: &str) -> Option<&str> {
    let stack = path.strip_prefix("/states/")?;
    let ok = !stack.is_empty()
        && stack.len() <= 64
        && stack
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_');
    ok.then_some(stack)
}

/// The role a basic-auth header grants. Both passwords are always compared,
/// in constant time, so the timing says nothing about either.
pub fn role(passwords: Passwords<'_>, authorization: Option<&str>) -> Option<Role> {
    let password = basic_password(authorization?)?;
    let write = !passwords.write.is_empty() && constant_time_eq(&password, passwords.write);
    let read = !passwords.read.is_empty() && constant_time_eq(&password, passwords.read);
    if write {
        Some(Role::Write)
    } else if read {
        Some(Role::Read)
    } else {
        None
    }
}

fn basic_password(authorization: &str) -> Option<String> {
    let encoded = authorization.strip_prefix("Basic ")?;
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(encoded.trim())
        .ok()?;
    let decoded = String::from_utf8(decoded).ok()?;
    let (_user, password) = decoded.split_once(':')?;
    Some(password.to_string())
}

/// Compare two secrets without stopping at the first difference.
pub fn constant_time_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let mut diff = a.len() ^ b.len();
    for i in 0..a.len().max(b.len()) {
        let x = a.get(i).copied().unwrap_or(0);
        let y = b.get(i).copied().unwrap_or(0);
        diff |= usize::from(x ^ y);
    }
    diff == 0
}

/// The `ID` of OpenTofu's lock info JSON, and the JSON itself.
fn lock_info(body: &[u8]) -> Option<(String, String)> {
    let text = std::str::from_utf8(body).ok()?;
    if text.len() > 16_384 {
        return None;
    }
    let value: serde_json::Value = serde_json::from_str(text).ok()?;
    let id = value.get("ID")?.as_str()?;
    (!id.is_empty()).then(|| (id.to_string(), text.to_string()))
}

fn query_param<'a>(query: &'a str, name: &str) -> Option<&'a str> {
    query
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .find(|(k, _)| *k == name)
        .map(|(_, v)| v)
        .filter(|v| !v.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::BTreeMap;

    /// The same rules as the D1 store, in memory.
    #[derive(Default)]
    struct MemStore {
        states: RefCell<BTreeMap<String, Vec<String>>>,
        locks: RefCell<BTreeMap<String, (String, String)>>,
        fail: bool,
    }

    impl Store for MemStore {
        async fn state(&self, stack: &str) -> Result<Option<String>, StoreError> {
            if self.fail {
                return Err(StoreError("down".into()));
            }
            Ok(self
                .states
                .borrow()
                .get(stack)
                .and_then(|v| v.last().cloned()))
        }
        async fn put_state(&self, stack: &str, body: &str) -> Result<(), StoreError> {
            let mut states = self.states.borrow_mut();
            let versions = states.entry(stack.to_string()).or_default();
            versions.push(body.to_string());
            let keep = KEEP_VERSIONS as usize;
            if versions.len() > keep {
                versions.drain(..versions.len() - keep);
            }
            Ok(())
        }
        async fn lock(
            &self,
            stack: &str,
            id: &str,
            info: &str,
        ) -> Result<Option<String>, StoreError> {
            let mut locks = self.locks.borrow_mut();
            if let Some((_, held)) = locks.get(stack) {
                return Ok(Some(held.clone()));
            }
            locks.insert(stack.to_string(), (id.to_string(), info.to_string()));
            Ok(None)
        }
        async fn holder(&self, stack: &str) -> Result<Option<(String, String)>, StoreError> {
            Ok(self.locks.borrow().get(stack).cloned())
        }
        async fn unlock(&self, stack: &str, id: &str) -> Result<(), StoreError> {
            let mut locks = self.locks.borrow_mut();
            if locks.get(stack).is_some_and(|(held, _)| held == id) {
                locks.remove(stack);
            }
            Ok(())
        }
    }

    const PASSWORDS: Passwords<'static> = Passwords {
        read: "read-secret",
        write: "write-secret",
    };

    fn auth(password: &str) -> String {
        let encoded = base64::engine::general_purpose::STANDARD.encode(format!("tofu:{password}"));
        format!("Basic {encoded}")
    }

    fn block_on<F: std::future::Future>(f: F) -> F::Output {
        let mut f = std::pin::pin!(f);
        let mut cx = std::task::Context::from_waker(std::task::Waker::noop());
        loop {
            if let std::task::Poll::Ready(out) = f.as_mut().poll(&mut cx) {
                return out;
            }
        }
    }

    fn send(store: &MemStore, method: &str, path: &str, password: &str, body: &str) -> Response {
        let (path, query) = path.split_once('?').unwrap_or((path, ""));
        let authorization = auth(password);
        block_on(handle(
            store,
            PASSWORDS,
            Request {
                method,
                path,
                query,
                authorization: Some(&authorization),
                body: body.as_bytes(),
            },
        ))
    }

    fn lock_body(id: &str) -> String {
        format!(r#"{{"ID":"{id}","Operation":"OperationTypeApply","Who":"ci@runner"}}"#)
    }

    #[test]
    fn empty_stack_reads_as_no_content() {
        let store = MemStore::default();
        assert_eq!(
            send(&store, "GET", "/states/auth0", "read-secret", "").status,
            204
        );
    }

    #[test]
    fn write_then_read_returns_the_latest_state() {
        let store = MemStore::default();
        assert_eq!(
            send(
                &store,
                "POST",
                "/states/auth0",
                "write-secret",
                r#"{"v":1}"#
            )
            .status,
            200
        );
        assert_eq!(
            send(
                &store,
                "POST",
                "/states/auth0",
                "write-secret",
                r#"{"v":2}"#
            )
            .status,
            200
        );
        let got = send(&store, "GET", "/states/auth0", "read-secret", "");
        assert_eq!(got, Response::json(200, r#"{"v":2}"#));
    }

    #[test]
    fn stacks_are_separate() {
        let store = MemStore::default();
        send(
            &store,
            "POST",
            "/states/auth0",
            "write-secret",
            r#"{"s":"auth0"}"#,
        );
        assert_eq!(
            send(&store, "GET", "/states/cloudflare", "read-secret", "").status,
            204
        );
    }

    #[test]
    fn read_password_can_only_get() {
        let store = MemStore::default();
        for method in ["POST", "LOCK", "UNLOCK"] {
            let got = send(
                &store,
                method,
                "/states/auth0",
                "read-secret",
                &lock_body("a"),
            );
            assert_eq!(got.status, 403, "{method}");
        }
        assert!(store.states.borrow().is_empty());
        assert!(store.locks.borrow().is_empty());
    }

    #[test]
    fn wrong_or_missing_password_is_unauthorized() {
        let store = MemStore::default();
        assert_eq!(
            send(&store, "GET", "/states/auth0", "guess", "").status,
            401
        );
        let got = block_on(handle(
            &store,
            PASSWORDS,
            Request {
                method: "GET",
                path: "/states/auth0",
                query: "",
                authorization: None,
                body: b"",
            },
        ));
        assert_eq!(got.status, 401);
    }

    #[test]
    fn unset_passwords_refuse_everything() {
        let empty = Passwords {
            read: "",
            write: "",
        };
        assert_eq!(role(empty, Some(&auth(""))), None);
    }

    #[test]
    fn lock_is_exclusive_and_conflict_returns_the_holder() {
        let store = MemStore::default();
        assert_eq!(
            send(
                &store,
                "LOCK",
                "/states/auth0",
                "write-secret",
                &lock_body("one")
            )
            .status,
            200
        );
        let second = send(
            &store,
            "LOCK",
            "/states/auth0",
            "write-secret",
            &lock_body("two"),
        );
        assert_eq!(second.status, 409);
        assert!(second.body.contains(r#""ID":"one""#));
    }

    #[test]
    fn only_the_lock_holder_can_write_while_locked() {
        let store = MemStore::default();
        send(
            &store,
            "LOCK",
            "/states/auth0",
            "write-secret",
            &lock_body("one"),
        );
        assert_eq!(
            send(&store, "POST", "/states/auth0?ID=two", "write-secret", "{}").status,
            409
        );
        assert_eq!(
            send(&store, "POST", "/states/auth0", "write-secret", "{}").status,
            409
        );
        assert_eq!(
            send(&store, "POST", "/states/auth0?ID=one", "write-secret", "{}").status,
            200
        );
    }

    #[test]
    fn a_write_naming_a_lost_lock_fails() {
        let store = MemStore::default();
        assert_eq!(
            send(
                &store,
                "POST",
                "/states/auth0?ID=gone",
                "write-secret",
                "{}"
            )
            .status,
            409
        );
    }

    #[test]
    fn unlock_only_for_the_same_id() {
        let store = MemStore::default();
        send(
            &store,
            "LOCK",
            "/states/auth0",
            "write-secret",
            &lock_body("one"),
        );
        assert_eq!(
            send(
                &store,
                "UNLOCK",
                "/states/auth0",
                "write-secret",
                &lock_body("two")
            )
            .status,
            409
        );
        assert!(store.locks.borrow().contains_key("auth0"));
        assert_eq!(
            send(
                &store,
                "UNLOCK",
                "/states/auth0",
                "write-secret",
                &lock_body("one")
            )
            .status,
            200
        );
        assert!(store.locks.borrow().is_empty());
        // Unlocking a free stack is fine, so a retried UNLOCK doesn't fail.
        assert_eq!(
            send(
                &store,
                "UNLOCK",
                "/states/auth0",
                "write-secret",
                &lock_body("one")
            )
            .status,
            200
        );
    }

    #[test]
    fn lock_needs_lock_info_with_an_id() {
        let store = MemStore::default();
        assert_eq!(
            send(&store, "LOCK", "/states/auth0", "write-secret", "{}").status,
            400
        );
        assert_eq!(
            send(&store, "LOCK", "/states/auth0", "write-secret", "nope").status,
            400
        );
    }

    #[test]
    fn only_the_last_versions_are_kept() {
        let store = MemStore::default();
        for v in 0..15 {
            send(
                &store,
                "POST",
                "/states/auth0",
                "write-secret",
                &format!(r#"{{"v":{v}}}"#),
            );
        }
        let states = store.states.borrow();
        assert_eq!(states["auth0"].len(), KEEP_VERSIONS as usize);
        assert_eq!(states["auth0"][0], r#"{"v":5}"#);
    }

    #[test]
    fn bad_bodies_are_rejected() {
        let store = MemStore::default();
        assert_eq!(
            send(&store, "POST", "/states/auth0", "write-secret", "not json").status,
            400
        );
        let huge = format!(r#"{{"x":"{}"}}"#, "a".repeat(MAX_STATE_BYTES));
        assert_eq!(
            send(&store, "POST", "/states/auth0", "write-secret", &huge).status,
            413
        );
        assert!(store.states.borrow().is_empty());
    }

    #[test]
    fn other_paths_and_methods_are_refused() {
        let store = MemStore::default();
        assert_eq!(send(&store, "GET", "/", "read-secret", "").status, 404);
        assert_eq!(
            send(&store, "GET", "/states/", "read-secret", "").status,
            404
        );
        assert_eq!(
            send(&store, "GET", "/states/Auth0", "read-secret", "").status,
            404
        );
        assert_eq!(
            send(&store, "GET", "/states/a/b", "read-secret", "").status,
            404
        );
        assert_eq!(
            send(&store, "DELETE", "/states/auth0", "write-secret", "").status,
            405
        );
    }

    #[test]
    fn store_errors_say_nothing_about_state() {
        let store = MemStore {
            fail: true,
            ..MemStore::default()
        };
        let got = send(&store, "GET", "/states/auth0", "read-secret", "");
        assert_eq!(got, Response::error(500, "state store error"));
    }

    #[test]
    fn constant_time_eq_matches_only_equal_strings() {
        assert!(constant_time_eq("abc", "abc"));
        assert!(!constant_time_eq("abc", "abd"));
        assert!(!constant_time_eq("abc", "abcd"));
        assert!(!constant_time_eq("", "a"));
        assert!(constant_time_eq("", ""));
    }

    #[test]
    fn write_password_wins_when_both_match() {
        let same = Passwords {
            read: "same",
            write: "same",
        };
        assert_eq!(role(same, Some(&auth("same"))), Some(Role::Write));
    }
}
