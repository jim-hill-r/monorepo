use anyhow::{Context, Result};
use clap::Parser;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "agent-copilot")]
#[command(about = "A tool to create GitHub Copilot agent tasks directly using the Copilot API", long_about = None)]
struct Args {
    /// Repository in the format "owner/repo"
    #[arg(short, long)]
    repo: String,

    /// Title for the agent task (unused, kept for backwards compatibility - the problem statement is used instead)
    #[arg(short, long)]
    title: String,

    /// Path to the agent prompt file
    #[arg(short, long)]
    prompt_file: PathBuf,

    /// GitHub token for authentication (can also use GITHUB_TOKEN env var)
    #[arg(long, env = "GITHUB_TOKEN")]
    token: String,

    /// Base branch for the pull request (optional)
    #[arg(long)]
    base_branch: Option<String>,

    /// Custom agent to use (optional)
    #[arg(long)]
    custom_agent: Option<String>,
}

#[derive(Serialize, Debug)]
struct PullRequestOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    base_ref: Option<String>,
}

#[derive(Serialize, Debug)]
struct CreateJobRequest {
    problem_statement: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    custom_agent: Option<String>,
    event_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pull_request: Option<PullRequestOptions>,
}

#[derive(Deserialize, Debug)]
struct CreateJobResponse {
    job_id: String,
    #[serde(default)]
    session_id: Option<String>,
    #[serde(default)]
    pull_request: Option<PullRequestInfo>,
}

#[derive(Deserialize, Debug)]
struct PullRequestInfo {
    number: i64,
}

fn read_prompt_file(path: &PathBuf) -> Result<String> {
    fs::read_to_string(path)
        .context(format!("Failed to read prompt file: {}", path.display()))
}

fn create_agent_task(
    repo: &str,
    problem_statement: String,
    token: &str,
    base_branch: Option<String>,
    custom_agent: Option<String>,
) -> Result<CreateJobResponse> {
    // Parse owner and repo from the repo string
    let parts: Vec<&str> = repo.split('/').collect();
    if parts.len() != 2 {
        anyhow::bail!("Repository must be in format 'owner/repo'");
    }
    let owner = parts[0];
    let repo_name = parts[1];
    
    // GitHub Copilot Jobs API endpoint
    // This is the same endpoint used by `gh agent-task create`
    let url = format!(
        "https://api.githubcopilot.com/agents/swe/v1/jobs/{}/{}",
        urlencoding::encode(owner),
        urlencoding::encode(repo_name)
    );
    
    let client = Client::new();
    
    let pull_request = if base_branch.is_some() || custom_agent.is_some() {
        // The Copilot API expects a pull_request object to be present
        // even when only specifying custom_agent without base_ref
        Some(PullRequestOptions {
            base_ref: base_branch.map(|branch| format!("refs/heads/{}", branch)),
        })
    } else {
        Some(PullRequestOptions { base_ref: None })
    };
    
    let request_body = CreateJobRequest {
        problem_statement,
        custom_agent,
        event_type: "gh_cli".to_string(),
        pull_request,
    };
    
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .header("Copilot-Integration-Id", "copilot-4-cli")
        .json(&request_body)
        .send()
        .context("Failed to send request to GitHub Copilot API")?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().unwrap_or_else(|_| "Unknown error".to_string());
        anyhow::bail!(
            "GitHub Copilot API request failed with status {}: {}",
            status,
            error_text
        );
    }
    
    response
        .json::<CreateJobResponse>()
        .context("Failed to parse GitHub Copilot API response")
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
    let problem_statement = read_prompt_file(&args.prompt_file)?;
    
    // Create the agent task
    println!("Creating GitHub Copilot agent task in repository: {}", args.repo);
    println!("Using prompt file: {}", args.prompt_file.display());
    
    let response = create_agent_task(
        &args.repo,
        problem_statement,
        &args.token,
        args.base_branch,
        args.custom_agent,
    )?;
    
    println!("✓ Successfully created GitHub Copilot agent task");
    println!("  Job ID: {}", response.job_id);
    if let Some(session_id) = response.session_id {
        println!("  Session ID: {}", session_id);
    }
    if let Some(pr) = response.pull_request {
        println!("  Pull Request: #{}", pr.number);
        
        // Construct the PR URL
        let pr_url = format!("https://github.com/{}/pull/{}", args.repo, pr.number);
        println!("  URL: {}", pr_url);
    }
    
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
    fn test_create_job_request_serialization() {
        let request = CreateJobRequest {
            problem_statement: "Test problem statement".to_string(),
            custom_agent: None,
            event_type: "gh_cli".to_string(),
            pull_request: Some(PullRequestOptions { base_ref: None }),
        };
        
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("Test problem statement"));
        assert!(json.contains("gh_cli"));
    }

    #[test]
    fn test_create_job_request_with_custom_agent() {
        let request = CreateJobRequest {
            problem_statement: "Test problem".to_string(),
            custom_agent: Some("my-agent".to_string()),
            event_type: "gh_cli".to_string(),
            pull_request: Some(PullRequestOptions {
                base_ref: Some("refs/heads/main".to_string()),
            }),
        };
        
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("my-agent"));
        assert!(json.contains("refs/heads/main"));
    }

    #[test]
    fn test_create_job_response_deserialization() {
        let json = r#"{
            "job_id": "test-job-123",
            "session_id": "session-456",
            "pull_request": {
                "number": 42
            }
        }"#;
        
        let response: CreateJobResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.job_id, "test-job-123");
        assert_eq!(response.session_id, Some("session-456".to_string()));
        assert!(response.pull_request.is_some());
        assert_eq!(response.pull_request.unwrap().number, 42);
    }

    #[test]
    fn test_create_job_response_minimal() {
        let json = r#"{"job_id": "test-job"}"#;
        
        let response: CreateJobResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.job_id, "test-job");
        assert_eq!(response.session_id, None);
        assert!(response.pull_request.is_none());
    }
}
