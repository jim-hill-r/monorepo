//! End-to-end check of pane's Worker: builds it with worker-build, serves it
//! with `wrangler dev` over a stand-in assets folder (the real Dioxus build
//! takes minutes and is checked by pane/web's own CI), and checks that pages
//! and data come from the assets, client-side routes get index.html, and
//! only /api/* reaches the Worker.
//!
//! Ignored by plain `cargo test` because it needs worker-build and wrangler;
//! `mise run ci` runs it with `cargo test --test e2e -- --ignored`
//! (standards/tasks/cloudflare-worker-assets.toml).

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Stops `wrangler dev` and removes the assets folder, even when a check fails.
struct Server {
    child: Child,
    assets: PathBuf,
    log: PathBuf,
}

impl Server {
    fn log(&self) -> String {
        std::fs::read_to_string(&self.log).unwrap_or_default()
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        // SIGTERM first, so wrangler stops the workerd processes it started;
        // a SIGKILL would leave them holding the port.
        let _ = Command::new("kill")
            .arg(self.child.id().to_string())
            .status();
        let deadline = Instant::now() + Duration::from_secs(10);
        while Instant::now() < deadline {
            if matches!(self.child.try_wait(), Ok(Some(_))) {
                break;
            }
            std::thread::sleep(Duration::from_millis(200));
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_dir_all(&self.assets);
        let _ = std::fs::remove_file(&self.log);
    }
}

/// A port nothing is listening on right now.
fn free_port() -> Result<u16> {
    Ok(std::net::TcpListener::bind(("127.0.0.1", 0))?
        .local_addr()?
        .port())
}

/// One HTTP/1.1 request; returns the status code and the body.
fn request(port: u16, method: &str, path: &str, headers: &[&str]) -> Result<(u16, String)> {
    let mut stream = TcpStream::connect(("127.0.0.1", port))?;
    stream.set_read_timeout(Some(Duration::from_secs(20)))?;
    let mut head = format!("{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n");
    for h in headers {
        head.push_str(h);
        head.push_str("\r\n");
    }
    head.push_str("Content-Length: 0\r\n\r\n");
    stream.write_all(head.as_bytes())?;
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw)?;
    let text = String::from_utf8_lossy(&raw).into_owned();
    let status = text
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .ok_or("no status line")?;
    let body = text.split_once("\r\n\r\n").map(|(_, b)| b).unwrap_or("");
    Ok((status, body.to_owned()))
}

fn start(port: u16) -> Result<Server> {
    let assets = std::env::temp_dir().join(format!("pane-e2e-{}", std::process::id()));
    std::fs::create_dir_all(assets.join("data"))?;
    std::fs::write(
        assets.join("index.html"),
        "<!doctype html><title>pane e2e</title>",
    )?;
    std::fs::write(assets.join("data/snapshot.json"), "{\"e2e\":true}")?;

    let built = Command::new("worker-build").arg("--release").status()?;
    if !built.success() {
        return Err("worker-build failed".into());
    }
    let log = assets.with_extension("log");
    let out = std::fs::File::create(&log)?;
    let child = Command::new("wrangler")
        .args(["dev", "--ip", "127.0.0.1", "--port", &port.to_string()])
        .arg("--assets")
        .arg(&assets)
        .stdin(Stdio::null())
        .stdout(out.try_clone()?)
        .stderr(out)
        .spawn()?;
    let server = Server { child, assets, log };

    let deadline = Instant::now() + Duration::from_secs(90);
    while Instant::now() < deadline {
        if matches!(request(port, "GET", "/", &[]), Ok((200, _))) {
            return Ok(server);
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    Err(format!(
        "wrangler dev did not start within 90 seconds; its log:\n{}",
        server.log()
    )
    .into())
}

#[test]
#[ignore = "needs worker-build and wrangler; mise run ci runs it"]
fn worker_serves_assets_and_only_api_runs_rust() -> Result<()> {
    let port = free_port()?;
    let _server = start(port)?;

    let cases: [(&str, &str, &[&str], u16, &str); 6] = [
        ("GET", "/", &[], 200, "pane e2e"),
        ("GET", "/data/snapshot.json", &[], 200, "\"e2e\":true"),
        (
            "GET",
            "/projects/pane",
            &["Sec-Fetch-Mode: navigate"],
            200,
            "pane e2e",
        ),
        ("GET", "/api/health", &[], 200, "{\"status\":\"ok\"}"),
        ("POST", "/api/health", &[], 405, "use GET"),
        ("GET", "/api/nope", &[], 404, "no such API route"),
    ];
    let mut failures = Vec::new();
    for (method, path, headers, want_status, want_body) in cases {
        let (status, body) = request(port, method, path, headers)?;
        if status != want_status || !body.contains(want_body) {
            failures.push(format!(
                "{method} {path}: got {status} '{body}', want {want_status} '{want_body}'"
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
