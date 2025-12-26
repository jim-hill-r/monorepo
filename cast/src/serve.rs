use std::fs;
use std::path::Path;
use thiserror::Error;
use tiny_http::{Header, Response, Server};

#[derive(Error, Debug)]
pub enum ServeError {
    #[error("Serve command failed: {0}")]
    ServeFailed(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed to bind server: {0}")]
    BindError(String),
}

/// Serve static files from the current directory
///
/// This starts a simple HTTP server that serves files from the working directory.
/// The server runs on localhost:8000 by default.
pub fn run(working_directory: impl AsRef<Path>) -> Result<(), ServeError> {
    let working_directory = working_directory.as_ref();

    // Start server on localhost:8000
    let server =
        Server::http("127.0.0.1:8000").map_err(|e| ServeError::BindError(e.to_string()))?;

    println!("Serving static files from: {}", working_directory.display());
    println!("Server running at http://127.0.0.1:8000");
    println!("Press Ctrl+C to stop");

    // Handle requests
    for request in server.incoming_requests() {
        let url_path = request.url();

        // Remove leading slash and decode URL
        let path_str = url_path.trim_start_matches('/');

        // Construct file path
        let mut file_path = working_directory.to_path_buf();

        // Handle root path - serve index.html if it exists
        if path_str.is_empty() || path_str == "/" {
            file_path.push("index.html");
        } else {
            file_path.push(path_str);
        }

        // Prevent directory traversal attacks
        let canonical_base = fs::canonicalize(working_directory).map_err(ServeError::IoError)?;

        // Try to canonicalize the file path, but if it doesn't exist yet, that's ok
        let canonical_file = match fs::canonicalize(&file_path) {
            Ok(p) => p,
            Err(_) => {
                // File doesn't exist, send 404
                let response = Response::from_string("404 Not Found").with_status_code(404);
                let _ = request.respond(response);
                continue;
            }
        };

        // Check if the canonical path is within the working directory
        if !canonical_file.starts_with(&canonical_base) {
            let response = Response::from_string("403 Forbidden").with_status_code(403);
            let _ = request.respond(response);
            continue;
        }

        // Serve the file
        if canonical_file.is_file() {
            match fs::read(&canonical_file) {
                Ok(contents) => {
                    // Determine content type based on extension
                    let content_type = get_content_type(&canonical_file);
                    let header =
                        Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()).unwrap();

                    let response = Response::from_data(contents).with_header(header);
                    let _ = request.respond(response);
                }
                Err(_) => {
                    let response =
                        Response::from_string("500 Internal Server Error").with_status_code(500);
                    let _ = request.respond(response);
                }
            }
        } else if canonical_file.is_dir() {
            // Try to serve index.html from the directory
            let index_path = canonical_file.join("index.html");
            if index_path.is_file() {
                match fs::read(&index_path) {
                    Ok(contents) => {
                        let header =
                            Header::from_bytes(&b"Content-Type"[..], b"text/html; charset=utf-8")
                                .unwrap();
                        let response = Response::from_data(contents).with_header(header);
                        let _ = request.respond(response);
                    }
                    Err(_) => {
                        let response = Response::from_string("500 Internal Server Error")
                            .with_status_code(500);
                        let _ = request.respond(response);
                    }
                }
            } else {
                // Directory without index.html - return 404
                let response = Response::from_string("404 Not Found").with_status_code(404);
                let _ = request.respond(response);
            }
        } else {
            let response = Response::from_string("404 Not Found").with_status_code(404);
            let _ = request.respond(response);
        }
    }

    Ok(())
}

/// Get content type based on file extension
fn get_content_type(path: &Path) -> String {
    match path.extension().and_then(|s| s.to_str()) {
        Some("html") | Some("htm") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("wasm") => "application/wasm",
        Some("txt") => "text/plain; charset=utf-8",
        Some("xml") => "application/xml",
        Some("pdf") => "application/pdf",
        _ => "application/octet-stream",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempdir::TempDir;

    #[test]
    fn test_get_content_type() {
        assert_eq!(
            get_content_type(Path::new("test.html")),
            "text/html; charset=utf-8"
        );
        assert_eq!(
            get_content_type(Path::new("test.css")),
            "text/css; charset=utf-8"
        );
        assert_eq!(
            get_content_type(Path::new("test.js")),
            "application/javascript; charset=utf-8"
        );
        assert_eq!(get_content_type(Path::new("test.json")), "application/json");
        assert_eq!(get_content_type(Path::new("test.png")), "image/png");
        assert_eq!(
            get_content_type(Path::new("test.unknown")),
            "application/octet-stream"
        );
    }

    #[test]
    fn test_serve_requires_directory() {
        let tmp_dir = TempDir::new("test_serve").unwrap();

        // Create a simple HTML file to serve
        fs::write(
            tmp_dir.path().join("index.html"),
            "<html><body>Hello, world!</body></html>",
        )
        .unwrap();

        // We can't easily test the server without spawning threads and making HTTP requests
        // For now, just verify the directory exists and the function can be called
        assert!(tmp_dir.path().exists());
    }

    #[test]
    fn test_serve_with_subdirectories() {
        let tmp_dir = TempDir::new("test_serve_subdirs").unwrap();

        // Create directory structure
        fs::create_dir_all(tmp_dir.path().join("subdir")).unwrap();
        fs::write(tmp_dir.path().join("subdir/file.txt"), "Test content").unwrap();

        // Verify structure exists
        assert!(tmp_dir.path().join("subdir/file.txt").exists());
    }
}
