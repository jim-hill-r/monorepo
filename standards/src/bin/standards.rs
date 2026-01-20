use clap::{Parser, Subcommand};
use standards::{
    audit::{configuration, documentation, naming},
    discovery,
    issues::IssuesFile,
};
use std::path::{Path, PathBuf};
use std::process;

#[derive(Parser)]
#[command(
    author,
    version,
    about = "Standards enforcement CLI for the monorepo",
    long_about = "A tool to audit and enforce coding standards across all projects in the monorepo"
)]
struct Args {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Audit projects for standards compliance
    Audit(AuditCommand),
    /// Add a TODO item to an ISSUES.md file
    AddTodo(AddTodoCommand),
    /// Audit projects and write violations to their ISSUES.md files
    AuditToIssues(AuditToIssuesCommand),
}

#[derive(Parser)]
struct AuditCommand {
    /// Path to the monorepo root (defaults to current directory)
    #[arg(short, long)]
    path: Option<String>,
}

#[derive(Parser)]
struct AddTodoCommand {
    /// Path to the ISSUES.md file (defaults to ./ISSUES.md)
    #[arg(short, long)]
    file: Option<String>,

    /// The TODO text to add
    #[arg(short, long)]
    todo: String,

    /// Add to Backlog section instead of Priority Issues
    #[arg(short, long)]
    backlog: bool,
}

#[derive(Parser)]
struct AuditToIssuesCommand {
    /// Path to the monorepo root (defaults to current directory)
    #[arg(short, long)]
    path: Option<String>,

    /// Generate a summary report (defaults to true)
    #[arg(short, long, default_value = "true")]
    summary: bool,
}

fn main() {
    let args = Args::parse();

    let result = match args.cmd {
        Commands::Audit(audit_cmd) => audit(audit_cmd),
        Commands::AddTodo(add_todo_cmd) => add_todo(add_todo_cmd),
        Commands::AuditToIssues(audit_to_issues_cmd) => audit_to_issues(audit_to_issues_cmd),
    };

    match result {
        Ok(message) => {
            println!("{}", message);
            process::exit(0);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn audit(cmd: AuditCommand) -> Result<String, String> {
    let path = cmd.path.unwrap_or_else(|| ".".to_string());
    let root_path = PathBuf::from(&path);

    // Discover projects in the repository
    let projects = discovery::discover_projects(&root_path)
        .map_err(|e| format!("Failed to discover projects: {}", e))?;

    // Build the audit report
    let mut report = String::new();
    report.push_str(&format!("Standards audit for path: {}\n\n", path));
    report.push_str(&format!("Discovered {} project(s)\n\n", projects.len()));

    // Run naming standards audit
    let naming_result = naming::audit_naming_standards(&projects);

    if naming_result.has_violations() {
        report.push_str("=== Naming Standards Violations ===\n\n");

        for violation in &naming_result.violations {
            report.push_str(&format!(
                "[{}] {} - {}\n",
                violation.standard_id, violation.project_name, violation.message
            ));
            report.push_str(&format!("  Path: {}\n", violation.project_path));
            report.push_str(&format!("  Severity: {:?}\n\n", violation.severity));
        }

        report.push_str(&format!(
            "Total violations: {}\n\n",
            naming_result.violations.len()
        ));
    } else {
        report.push_str("✓ All naming standards checks passed\n\n");
    }

    // Run configuration standards audit
    let config_result = configuration::audit_configuration_standards(&projects);

    if config_result.has_violations() {
        report.push_str("=== Configuration Standards Violations ===\n\n");

        for violation in &config_result.violations {
            report.push_str(&format!(
                "[{}] {} - {}\n",
                violation.standard_id, violation.project_name, violation.message
            ));
            report.push_str(&format!("  Path: {}\n", violation.project_path));
            report.push_str(&format!("  Severity: {:?}\n\n", violation.severity));
        }

        report.push_str(&format!(
            "Total violations: {}\n\n",
            config_result.violations.len()
        ));
    } else {
        report.push_str("✓ All configuration standards checks passed\n\n");
    }

    // Run documentation standards audit
    let doc_result = documentation::audit_documentation_standards(&projects);

    if doc_result.has_violations() {
        report.push_str("=== Documentation Standards Violations ===\n\n");

        for violation in &doc_result.violations {
            report.push_str(&format!(
                "[{}] {} - {}\n",
                violation.standard_id, violation.project_name, violation.message
            ));
            report.push_str(&format!("  Path: {}\n", violation.project_path));
            report.push_str(&format!("  Severity: {:?}\n\n", violation.severity));
        }

        report.push_str(&format!(
            "Total violations: {}\n\n",
            doc_result.violations.len()
        ));
    } else {
        report.push_str("✓ All documentation standards checks passed\n\n");
    }

    // Summary
    let total_violations = naming_result.violations.len()
        + config_result.violations.len()
        + doc_result.violations.len();
    report.push_str("=== Summary ===\n");
    report.push_str(&format!("Total violations found: {}\n", total_violations));

    Ok(report)
}

fn add_todo(cmd: AddTodoCommand) -> Result<String, String> {
    let file_path = cmd.file.unwrap_or_else(|| "./ISSUES.md".to_string());
    let path = Path::new(&file_path);

    // Try to parse existing file, or create new one if it doesn't exist
    let mut issues_file = if path.exists() {
        IssuesFile::parse(path).map_err(|e| format!("Failed to parse {}: {}", file_path, e))?
    } else {
        IssuesFile::new(file_path.clone())
    };

    // Add the TODO to the appropriate section
    let added = if cmd.backlog {
        issues_file.add_backlog_todo(cmd.todo.clone())
    } else {
        issues_file.add_priority_todo(cmd.todo.clone())
    };

    if !added {
        return Ok(format!(
            "TODO already exists in {}\nTODO: {}",
            file_path, cmd.todo
        ));
    }

    // Write the updated file
    issues_file
        .write()
        .map_err(|e| format!("Failed to write {}: {}", file_path, e))?;

    let section = if cmd.backlog {
        "Backlog"
    } else {
        "Priority Issues"
    };
    Ok(format!(
        "Successfully added TODO to {} section in {}\nTODO: {}",
        section, file_path, cmd.todo
    ))
}

fn audit_to_issues(cmd: AuditToIssuesCommand) -> Result<String, String> {
    use standards::audit;

    let path = cmd.path.unwrap_or_else(|| ".".to_string());
    let root_path = PathBuf::from(&path);

    // Discover projects in the repository
    let projects = discovery::discover_projects(&root_path)
        .map_err(|e| format!("Failed to discover projects: {}", e))?;

    // Build report header
    let mut report = String::new();
    report.push_str(&format!(
        "Audit-to-Issues for path: {}\n\n",
        path
    ));
    report.push_str(&format!("Discovered {} project(s)\n\n", projects.len()));

    // Run all audits
    let naming_result = naming::audit_naming_standards(&projects);
    let config_result = configuration::audit_configuration_standards(&projects);
    let doc_result = documentation::audit_documentation_standards(&projects);

    // Combine all violations
    let mut all_violations = naming_result.violations.clone();
    all_violations.extend(config_result.violations.clone());
    all_violations.extend(doc_result.violations.clone());

    if all_violations.is_empty() {
        return Ok(format!("{}✓ No violations found. All projects comply with standards.", report));
    }

    // Group violations by project path
    let mut violations_by_project: std::collections::HashMap<String, Vec<&audit::Violation>> =
        std::collections::HashMap::new();
    for violation in &all_violations {
        violations_by_project
            .entry(violation.project_path.clone())
            .or_default()
            .push(violation);
    }

    // Write violations to each project's ISSUES.md
    let mut total_written = 0;
    let mut projects_updated = 0;

    for (project_path, violations) in &violations_by_project {
        let issues_path = PathBuf::from(project_path).join("ISSUES.md");

        // Create audit result for this project
        let mut project_result = audit::AuditResult::new();
        for violation in violations {
            project_result.add_violation((*violation).clone());
        }

        match audit::write_violations_to_issues(&project_result, &issues_path) {
            Ok(count) => {
                if count > 0 {
                    total_written += count;
                    projects_updated += 1;
                    if cmd.summary {
                        report.push_str(&format!(
                            "✓ Wrote {} TODO(s) to {}\n",
                            count,
                            issues_path.display()
                        ));
                    }
                }
            }
            Err(e) => {
                report.push_str(&format!(
                    "✗ Failed to write to {}: {}\n",
                    issues_path.display(),
                    e
                ));
            }
        }
    }

    // Summary
    report.push_str("\n=== Summary ===\n");
    report.push_str(&format!("Total violations found: {}\n", all_violations.len()));
    report.push_str(&format!("Projects updated: {}\n", projects_updated));
    report.push_str(&format!("TODOs written: {}\n", total_written));

    Ok(report)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_default_path() {
        let cmd = AuditCommand { path: None };
        let result = audit(cmd);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Standards audit for path"));
        assert!(output.contains("Discovered"));
    }

    #[test]
    fn test_audit_custom_path() {
        let cmd = AuditCommand {
            path: Some("/custom/path".to_string()),
        };
        let result = audit(cmd);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("/custom/path"));
    }

    #[test]
    fn test_add_todo_creates_new_file() {
        let temp_dir = std::env::temp_dir().join("test_add_todo_cli");
        std::fs::create_dir_all(&temp_dir).ok();

        let issues_path = temp_dir.join("ISSUES.md");
        let cmd = AddTodoCommand {
            file: Some(issues_path.to_string_lossy().to_string()),
            todo: "TODO: Test item".to_string(),
            backlog: false,
        };

        let result = add_todo(cmd);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Successfully added TODO"));

        // Verify file was created and contains the TODO
        let content = std::fs::read_to_string(&issues_path).expect("Failed to read file");
        assert!(content.contains("TODO: Test item"));
        assert!(content.contains("# Priority Issues"));

        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_add_todo_to_backlog() {
        let temp_dir = std::env::temp_dir().join("test_add_todo_backlog");
        std::fs::create_dir_all(&temp_dir).ok();

        let issues_path = temp_dir.join("ISSUES.md");
        let cmd = AddTodoCommand {
            file: Some(issues_path.to_string_lossy().to_string()),
            todo: "TODO: Backlog item".to_string(),
            backlog: true,
        };

        let result = add_todo(cmd);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Backlog"));

        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_add_todo_duplicate_detection() {
        let temp_dir = std::env::temp_dir().join("test_add_todo_duplicate");
        std::fs::create_dir_all(&temp_dir).ok();

        let issues_path = temp_dir.join("ISSUES.md");

        // Add first TODO
        let cmd1 = AddTodoCommand {
            file: Some(issues_path.to_string_lossy().to_string()),
            todo: "TODO: Duplicate test".to_string(),
            backlog: false,
        };
        let result1 = add_todo(cmd1);
        assert!(result1.is_ok());

        // Try to add same TODO again
        let cmd2 = AddTodoCommand {
            file: Some(issues_path.to_string_lossy().to_string()),
            todo: "TODO: Duplicate test".to_string(),
            backlog: false,
        };
        let result2 = add_todo(cmd2);
        assert!(result2.is_ok());
        assert!(result2.unwrap().contains("already exists"));

        std::fs::remove_dir_all(&temp_dir).ok();
    }
}
