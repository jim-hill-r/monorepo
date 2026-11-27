//! Integration tests for ahead-of-time (AOT) compilation: my_lang source is
//! compiled to a native object file, linked into an executable, and run.

use std::path::PathBuf;
use std::process::Command;

use my_lang::aot::{self, AotError};

/// A fresh, empty directory for one test's output files.
fn scratch_dir(test: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("my_lang_aot_{}_{test}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

/// Build `source` into an executable, run it, and return its stdout.
fn build_and_run(test: &str, source: &str) -> String {
    let dir = scratch_dir(test);
    let exe = dir.join("program");
    aot::build_executable(source, &exe).expect("build should succeed");
    let output = Command::new(&exe).output().expect("run executable");
    assert!(output.status.success(), "executable should exit 0");
    let _ = std::fs::remove_dir_all(&dir);
    String::from_utf8(output.stdout).expect("stdout is UTF-8")
}

#[test]
fn executable_prints_main_result() {
    assert_eq!(
        build_and_run("literal", "function main() { return 42 }"),
        "42\n"
    );
}

#[test]
fn executable_prints_trailing_expression_of_main() {
    let source = "function square(x) { x * x }\nfunction main() { square(6) + 6 }";
    assert_eq!(build_and_run("trailing", source), "42\n");
}

#[test]
fn executable_calls_other_functions() {
    let source = "function square(x) { return x * x }\n\
                  function main() { return square(6) + 1 }";
    assert_eq!(build_and_run("calls", source), "37\n");
}

#[test]
fn executable_prints_negative_results() {
    assert_eq!(
        build_and_run("negative", "function main() { return 3 - 10 }"),
        "-7\n"
    );
}

#[test]
fn executable_prints_full_64_bit_values() {
    // 3_000_000_000 * 3 does not fit in 32 bits.
    let source = "function main() { return 3000000000 * 3 }";
    assert_eq!(build_and_run("wide", source), "9000000000\n");
}

#[test]
fn output_declaration_can_be_main() {
    assert_eq!(
        build_and_run("output_main", "output main() { return 7 }"),
        "7\n"
    );
}

#[test]
fn build_without_main_fails() {
    let dir = scratch_dir("no_main");
    let err = aot::build_executable("function add(a, b) { return a + b }", &dir.join("p"))
        .expect_err("build should fail without main");
    assert!(matches!(err, AotError::EntryNotFound(ref name) if name == "main"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn build_with_main_parameters_fails() {
    let dir = scratch_dir("main_params");
    let err = aot::build_executable("function main(x) { return x }", &dir.join("p"))
        .expect_err("build should fail when main takes parameters");
    assert!(matches!(err, AotError::EntryHasParameters { count: 1, .. }));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn build_reports_parse_errors() {
    let dir = scratch_dir("parse_error");
    let err = aot::build_executable("function main( { return 1 }", &dir.join("p"))
        .expect_err("build should fail to parse");
    assert!(matches!(err, AotError::ParseError(_)));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn build_leaves_no_object_file_behind() {
    let dir = scratch_dir("cleanup");
    aot::build_executable("function main() { return 1 }", &dir.join("program"))
        .expect("build should succeed");
    let files: Vec<_> = std::fs::read_dir(&dir)
        .expect("read dir")
        .map(|e| e.expect("entry").file_name())
        .collect();
    assert_eq!(files, vec!["program"]);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn object_file_exports_functions_for_c_linking() {
    let dir = scratch_dir("object");
    let obj = dir.join("lib.o");
    aot::compile_to_object("function add(a, b) { return a + b }", &obj)
        .expect("object compile should succeed");
    let bytes = std::fs::read(&obj).expect("read object file");
    assert!(!bytes.is_empty(), "object file should not be empty");
    #[cfg(target_os = "linux")]
    assert_eq!(&bytes[..4], b"\x7fELF", "object file should be ELF");

    // Link the object into a C program that calls `add`.
    let c_src = dir.join("main.c");
    std::fs::write(
        &c_src,
        "#include <stdio.h>\nlong long add(long long, long long);\n\
         int main(void) { printf(\"%lld\\n\", add(40, 2)); return 0; }\n",
    )
    .expect("write C source");
    let exe = dir.join("c_program");
    let status = Command::new("cc")
        .arg(&c_src)
        .arg(&obj)
        .arg("-o")
        .arg(&exe)
        .status()
        .expect("run cc");
    assert!(status.success(), "cc should link the object file");
    let output = Command::new(&exe).output().expect("run C program");
    assert_eq!(String::from_utf8_lossy(&output.stdout), "42\n");
    let _ = std::fs::remove_dir_all(&dir);
}
