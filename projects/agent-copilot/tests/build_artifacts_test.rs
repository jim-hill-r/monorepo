use std::path::Path;
use std::process::Command;

#[test]
fn test_linux_x86_64_artifact_exists() {
    let artifact_path = Path::new("artifacts/x86_64-unknown-linux-gnu/agent-copilot");
    assert!(
        artifact_path.exists(),
        "Linux x86_64 artifact should exist at {}",
        artifact_path.display()
    );
}

#[test]
fn test_linux_x86_64_artifact_is_executable() {
    let artifact_path = Path::new("artifacts/x86_64-unknown-linux-gnu/agent-copilot");
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(artifact_path).expect("Failed to read artifact metadata");
        let permissions = metadata.permissions();
        let mode = permissions.mode();
        
        // Check if the file has execute permission for owner (0o100)
        assert!(
            mode & 0o100 != 0,
            "Artifact should be executable (current mode: {:o})",
            mode
        );
    }
}

#[test]
fn test_linux_x86_64_artifact_shows_help() {
    let artifact_path = Path::new("artifacts/x86_64-unknown-linux-gnu/agent-copilot");
    
    let output = Command::new(artifact_path)
        .arg("--help")
        .output()
        .expect("Failed to execute artifact");
    
    assert!(
        output.status.success(),
        "Artifact should execute successfully with --help flag"
    );
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("agent-copilot"),
        "Help output should contain 'agent-copilot'"
    );
    assert!(
        stdout.contains("--repo"),
        "Help output should contain '--repo' option"
    );
    assert!(
        stdout.contains("--token"),
        "Help output should contain '--token' option"
    );
}

#[test]
fn test_artifact_is_elf_binary() {
    let artifact_path = Path::new("artifacts/x86_64-unknown-linux-gnu/agent-copilot");
    
    // Read the first 4 bytes (ELF magic number)
    let mut file = std::fs::File::open(artifact_path).expect("Failed to open artifact");
    use std::io::Read;
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic).expect("Failed to read ELF magic");
    
    // ELF files start with 0x7F 'E' 'L' 'F'
    assert_eq!(
        magic,
        [0x7F, b'E', b'L', b'F'],
        "Artifact should be a valid ELF binary"
    );
}
