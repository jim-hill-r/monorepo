use anyhow::{Context, Result};
use clap::Parser;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "agent-copilot")]
#[command(about = "A tool to create GitHub issues that trigger GitHub Copilot agents", long_about = None)]
struct Args {
    /// Repository in the format "owner/repo"
    #[arg(short, long)]
    repo: String,

    /// Title for the GitHub issue
    #[arg(short, long)]
    title: String,

    /// Path to the agent prompt file
    #[arg(short, long)]
    prompt_file: PathBuf,

    /// GitHub token for authentication (can also use GITHUB_TOKEN env var)
    #[arg(long, env = "GITHUB_TOKEN")]
    token: String,
}

#[derive(Serialize, Debug)]
struct CreateIssueRequest {
    title: String,
    body: String,
}

#[derive(Deserialize, Debug)]
struct CreateIssueResponse {
    number: u64,
    html_url: String,
}

fn read_prompt_file(path: &PathBuf) -> Result<String> {
    fs::read_to_string(path)
        .context(format!("Failed to read prompt file: {}", path.display()))
}

fn create_github_issue(
    repo: &str,
    title: String,
    body: String,
    token: &str,
) -> Result<CreateIssueResponse> {
    let url = format!("https://api.github.com/repos/{}/issues", repo);
    
    let client = Client::new();
    let request_body = CreateIssueRequest { title, body };
    
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "agent-copilot")
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .json(&request_body)
        .send()
        .context("Failed to send request to GitHub API")?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().unwrap_or_else(|_| "Unknown error".to_string());
        anyhow::bail!(
            "GitHub API request failed with status {}: {}",
            status,
            error_text
        );
    }
    
    response
        .json::<CreateIssueResponse>()
        .context("Failed to parse GitHub API response")
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Validate that the prompt file exists
    if !args.prompt_file.exists() {
        anyhow::bail!(
            "Error: Agent prompt file not found at {}",
            args.prompt_file.display()
        );
    }
    
    // Read the prompt file
    let body = read_prompt_file(&args.prompt_file)?;
    
    // Create the GitHub issue
    println!("Creating GitHub issue in repository: {}", args.repo);
    println!("Issue title: {}", args.title);
    println!("Using prompt file: {}", args.prompt_file.display());
    
    let response = create_github_issue(&args.repo, args.title, body, &args.token)?;
    
    println!("✓ Successfully created issue #{}", response.number);
    println!("  URL: {}", response.html_url);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_read_prompt_file_success() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Test content").unwrap();
        
        let path = temp_file.path().to_path_buf();
        let content = read_prompt_file(&path).unwrap();
        
        assert_eq!(content, "Test content\n");
    }

    #[test]
    fn test_read_prompt_file_not_found() {
        let path = PathBuf::from("/nonexistent/file.md");
        let result = read_prompt_file(&path);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_create_issue_request_serialization() {
        let request = CreateIssueRequest {
            title: "Test Issue".to_string(),
            body: "Test body".to_string(),
        };
        
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("Test Issue"));
        assert!(json.contains("Test body"));
    }
}
