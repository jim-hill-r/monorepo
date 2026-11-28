//! The `telemetry` Worker: one `POST /event` route that checks an event,
//! rate-limits it per IP address and writes it to Analytics Engine. See
//! `README.md` for the events and `standards/docs/plans/2026-11-deploy-feedback.md`
//! (phase 5) and `bear_down/docs/plans/2026-11-fan-feedback.md` for why.

pub mod event;

/// Security headers sent on every response (rule 3 in
/// `standards/docs/security.md`, checked by THR-007). The Worker only answers
/// with status codes and short error text, so the CSP allows nothing at all.
pub const SECURITY_HEADERS: [(&str, &str); 5] = [
    (
        "Content-Security-Policy",
        "default-src 'none'; frame-ancestors 'none'",
    ),
    ("X-Content-Type-Options", "nosniff"),
    ("Referrer-Policy", "no-referrer"),
    ("X-Frame-Options", "DENY"),
    (
        "Strict-Transport-Security",
        "max-age=31536000; includeSubDomains",
    ),
];

#[cfg(target_arch = "wasm32")]
mod worker_glue {
    use crate::SECURITY_HEADERS;
    use crate::event::{self, MAX_BODY};
    use worker::{
        AnalyticsEngineDataPointBuilder, Context, Env, Headers, Method, Request, Response, Result,
        event,
    };

    /// A fresh set of headers holding the security headers.
    fn security_headers() -> Result<Headers> {
        let headers = Headers::new();
        for (name, value) in SECURITY_HEADERS {
            headers.set(name, value)?;
        }
        Ok(headers)
    }

    /// The security headers, for responses outside `/event`.
    fn secure(response: Response) -> Result<Response> {
        Ok(response.with_headers(security_headers()?))
    }

    /// CORS headers: any origin may post, since no event carries
    /// credentials, and `text/plain` bodies (as `navigator.sendBeacon`
    /// sends) need no preflight. Includes the security headers.
    fn cors(response: Response) -> Result<Response> {
        let headers = security_headers()?;
        headers.set("Access-Control-Allow-Origin", "*")?;
        headers.set("Access-Control-Allow-Methods", "POST, OPTIONS")?;
        headers.set(
            "Access-Control-Allow-Headers",
            "content-type, x-jimhillr-synthetic",
        )?;
        headers.set("Access-Control-Max-Age", "86400")?;
        Ok(response.with_headers(headers))
    }

    fn header(req: &Request, name: &str) -> Option<String> {
        req.headers().get(name).ok().flatten()
    }

    #[event(fetch)]
    async fn fetch(mut req: Request, env: Env, _ctx: Context) -> Result<Response> {
        let path = req.path();
        match (req.method(), path.as_str()) {
            (Method::Options, "/event") => cors(Response::empty()?.with_status(204)),
            (Method::Post, "/event") => {
                // Do Not Track, Global Privacy Control and synthetic traffic
                // (eval engine phase 3) are accepted and dropped.
                let skip = event::is_dropped(|name| header(&req, name));
                if skip {
                    return cors(Response::empty()?.with_status(202));
                }
                let ip = header(&req, "cf-connecting-ip").unwrap_or_default();
                let limiter = env.rate_limiter("LIMITER")?;
                if !limiter.limit(ip).await?.success {
                    return cors(Response::error("too many events", 429)?);
                }
                let body = req.bytes().await?;
                if body.len() > MAX_BODY {
                    return cors(Response::error("body too large", 413)?);
                }
                let Ok(body) = String::from_utf8(body) else {
                    return cors(Response::error("body is not UTF-8", 400)?);
                };
                let point = match event::parse(&body) {
                    Ok(point) => point,
                    Err(error) => return cors(Response::error(error, 400)?),
                };
                let mut builder =
                    AnalyticsEngineDataPointBuilder::new().indexes([point.index.as_str()]);
                for blob in point.blobs {
                    builder = builder.add_blob(blob.as_str());
                }
                for double in point.doubles {
                    builder = builder.add_double(double);
                }
                builder.write_to(&env.analytics_engine("EVENTS")?)?;
                cors(Response::empty()?.with_status(202))
            }
            (_, "/event") => cors(Response::error("use POST", 405)?),
            _ => secure(Response::error("not found", 404)?),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SECURITY_HEADERS;

    fn value(name: &str) -> &'static str {
        SECURITY_HEADERS
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, v)| *v)
            .unwrap_or_else(|| panic!("{name} is not sent"))
    }

    /// Every header rule 3 of `standards/docs/security.md` asks for.
    #[test]
    fn sends_the_security_headers() {
        let csp = value("Content-Security-Policy");
        assert!(csp.contains("default-src 'none'"), "{csp}");
        assert!(csp.contains("frame-ancestors 'none'"), "{csp}");
        assert_eq!(value("X-Content-Type-Options"), "nosniff");
        assert_eq!(value("Referrer-Policy"), "no-referrer");
        assert_eq!(value("X-Frame-Options"), "DENY");
        assert!(value("Strict-Transport-Security").starts_with("max-age="));
    }
}
