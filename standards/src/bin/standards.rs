use clap::{Parser, Subcommand};
use standards::{
    audit::{configuration, naming},
    discovery,
};
use std::path::PathBuf;
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
}

#[derive(Parser)]
struct AuditCommand {
    /// Path to the monorepo root (defaults to current directory)
    #[arg(short, long)]
    path: Option<String>,
}

fn main() {
    let args = Args::parse();

    let result = match args.cmd {
        Commands::Audit(audit_cmd) => audit(audit_cmd),
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

    // Summary
    let total_violations = naming_result.violations.len() + config_result.violations.len();
    report.push_str("=== Summary ===\n");
    report.push_str(&format!("Total violations found: {}\n", total_violations));

    Ok(report)
}

#[cfg(test)]
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
}
