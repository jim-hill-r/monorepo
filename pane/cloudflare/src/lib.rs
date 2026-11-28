//! `jimhillr-pane`: pane's Cloudflare Worker. Workers static assets serve the
//! Dioxus build (`../web/dist/public`) for free, and `run_worker_first` in
//! `wrangler.toml` sends only `/api/*` here. The routes are in [`api`]; this
//! file only connects them to the Workers runtime.

pub mod api;

#[cfg(target_arch = "wasm32")]
mod worker_glue {
    use crate::api;
    use worker::{Context, Env, Headers, Request, Response, Result, event};

    #[event(fetch)]
    async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
        let path = req.path();
        if !api::is_api(&path) {
            // Only reached if run_worker_first changes: hand the request to
            // the static assets as Cloudflare would.
            return env.assets("ASSETS")?.fetch_request(req).await;
        }
        let answer = api::route(req.method().as_ref(), &path);
        let headers = Headers::new();
        headers.set("content-type", "application/json; charset=utf-8")?;
        headers.set("cache-control", "no-store")?;
        Ok(Response::ok(answer.body)?
            .with_status(answer.status)
            .with_headers(headers))
    }
}
