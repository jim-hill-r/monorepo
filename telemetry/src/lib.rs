//! The `telemetry` Worker: one `POST /event` route that checks an event,
//! rate-limits it per IP address and writes it to Analytics Engine. See
//! `README.md` for the events and `standards/docs/plans/2026-11-deploy-feedback.md`
//! (phase 5) and `bear_down/docs/plans/2026-11-fan-feedback.md` for why.

pub mod event;

#[cfg(target_arch = "wasm32")]
mod worker_glue {
    use crate::event::{self, MAX_BODY};
    use worker::{
        AnalyticsEngineDataPointBuilder, Context, Env, Headers, Method, Request, Response, Result,
        event,
    };

    /// CORS headers: any origin may post, since no event carries
    /// credentials, and `text/plain` bodies (as `navigator.sendBeacon`
    /// sends) need no preflight.
    fn cors(response: Response) -> Result<Response> {
        let headers = Headers::new();
        headers.set("Access-Control-Allow-Origin", "*")?;
        headers.set("Access-Control-Allow-Methods", "POST, OPTIONS")?;
        headers.set(
            "Access-Control-Allow-Headers",
            "content-type, x-monorepo-synthetic",
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
            _ => Response::error("not found", 404),
        }
    }
}
