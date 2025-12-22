use std::process::Output;

/// Format the output from a cargo command failure
pub fn format_cargo_output(output: &Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let mut result = String::new();
    if !stdout.is_empty() {
        result.push_str(&stdout);
    }
    if !stderr.is_empty() {
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str(&stderr);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Output;

    #[test]
    fn test_format_cargo_output_with_both_stdout_and_stderr() {
        let output = Output {
            status: std::process::ExitStatus::default(),
            stdout: b"stdout content".to_vec(),
            stderr: b"stderr content".to_vec(),
        };

        let result = format_cargo_output(&output);
        assert_eq!(result, "stdout content\nstderr content");
    }

    #[test]
    fn test_format_cargo_output_with_only_stdout() {
        let output = Output {
            status: std::process::ExitStatus::default(),
            stdout: b"stdout content".to_vec(),
            stderr: vec![],
        };

        let result = format_cargo_output(&output);
        assert_eq!(result, "stdout content");
    }

    #[test]
    fn test_format_cargo_output_with_only_stderr() {
        let output = Output {
            status: std::process::ExitStatus::default(),
            stdout: vec![],
            stderr: b"stderr content".to_vec(),
        };

        let result = format_cargo_output(&output);
        assert_eq!(result, "stderr content");
    }

    #[test]
    fn test_format_cargo_output_with_empty_output() {
        let output = Output {
            status: std::process::ExitStatus::default(),
            stdout: vec![],
            stderr: vec![],
        };

        let result = format_cargo_output(&output);
        assert_eq!(result, "");
    }
}
