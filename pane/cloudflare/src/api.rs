//! pane's `/api/*` routes, kept free of the Workers runtime so `cargo test`
//! checks them natively. Phase 3 of the review dashboard plan
//! (`pane/docs/plans/2026-11-review-dashboard.md`) adds the GitHub routes and
//! the Cloudflare Access check here.

/// The prefix the Worker runs for (`run_worker_first` in `wrangler.toml`).
/// Every other path is a static asset that never reaches the Worker.
pub const PREFIX: &str = "/api/";

/// An HTTP response, before it is turned into a `worker::Response`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiResponse {
    pub status: u16,
    /// A JSON body.
    pub body: String,
}

impl ApiResponse {
    fn json(status: u16, body: &str) -> Self {
        Self {
            status,
            body: body.to_string(),
        }
    }

    fn error(status: u16, message: &str) -> Self {
        // The messages are fixed strings with no quotes, so no escaping.
        Self::json(status, &format!("{{\"error\":\"{message}\"}}"))
    }
}

/// Whether `path` is one the Worker answers rather than the static assets.
pub fn is_api(path: &str) -> bool {
    path.starts_with(PREFIX) || path == "/api"
}

/// Answer one `/api/*` request. `method` is the upper-case HTTP method.
pub fn route(method: &str, path: &str) -> ApiResponse {
    match path {
        "/api/health" => match method {
            "GET" | "HEAD" => ApiResponse::json(200, "{\"status\":\"ok\"}"),
            _ => ApiResponse::error(405, "use GET"),
        },
        _ => ApiResponse::error(404, "no such API route"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_answers_ok() {
        let response = route("GET", "/api/health");
        assert_eq!(response.status, 200);
        assert_eq!(response.body, "{\"status\":\"ok\"}");
        assert_eq!(route("HEAD", "/api/health").status, 200);
    }

    #[test]
    fn health_refuses_writes() {
        for method in ["POST", "PUT", "DELETE", "PATCH"] {
            let response = route(method, "/api/health");
            assert_eq!(response.status, 405, "{method}");
            assert_eq!(response.body, "{\"error\":\"use GET\"}");
        }
    }

    #[test]
    fn unknown_routes_are_json_404s() {
        for path in ["/api/", "/api", "/api/nope", "/api/health/extra"] {
            let response = route("GET", path);
            assert_eq!(response.status, 404, "{path}");
            assert_eq!(response.body, "{\"error\":\"no such API route\"}");
        }
    }

    #[test]
    fn only_api_paths_reach_the_worker() {
        assert!(is_api("/api/health"));
        assert!(is_api("/api"));
        assert!(!is_api("/"));
        assert!(!is_api("/apis"));
        assert!(!is_api("/data/snapshot.json"));
        assert!(!is_api("/projects/pane"));
    }
}
