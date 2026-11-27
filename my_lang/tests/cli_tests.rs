//! End-to-end tests for the `my_lang` binary: `build` and `run` on real
//! source files.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// A fresh, empty directory for one test's files.
fn scratch_dir(test: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("my_lang_cli_{}_{test}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

/// Write `source` to `name` in `dir` and return its path.
fn write_source(dir: &Path, name: &str, source: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, source).expect("write source file");
    path
}

/// Run the `my_lang` binary with `args`.
fn my_lang(args: &[&std::ffi::OsStr]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_my_lang"))
        .args(args)
        .output()
        .expect("run my_lang")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn run_prints_main_result() {
    let dir = scratch_dir("run");
    let file = write_source(
        &dir,
        "answer.my",
        "function square(x) { return x * x }\nfunction main() { return square(6) + 6 }",
    );
    let output = my_lang(&["run".as_ref(), file.as_os_str()]);
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "42\n");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn run_prints_trailing_expression_without_return() {
    let dir = scratch_dir("run-trailing");
    let file = write_source(
        &dir,
        "answer.my",
        "function square(x) { x * x }\nfunction main() { square(6) + 6 }",
    );
    let output = my_lang(&["run".as_ref(), file.as_os_str()]);
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "42\n");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn run_passes_arguments_to_main() {
    let dir = scratch_dir("run_args");
    let file = write_source(&dir, "sub.my", "function main(a, b) { return a - b }");
    let output = my_lang(&[
        "run".as_ref(),
        file.as_os_str(),
        "-5".as_ref(),
        "3".as_ref(),
    ]);
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output), "-8\n");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn run_reports_wrong_argument_count() {
    let dir = scratch_dir("run_arity");
    let file = write_source(&dir, "sub.my", "function main(a, b) { return a - b }");
    let output = my_lang(&["run".as_ref(), file.as_os_str(), "1".as_ref()]);
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("'main' takes 2 arguments, but 1 were given"),
        "stderr: {}",
        stderr(&output)
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn run_reports_missing_main() {
    let dir = scratch_dir("run_no_main");
    let file = write_source(&dir, "lib.my", "function add(a, b) { return a + b }");
    let output = my_lang(&["run".as_ref(), file.as_os_str()]);
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("no entry function 'main'"),
        "stderr: {}",
        stderr(&output)
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn build_with_output_writes_a_working_executable() {
    let dir = scratch_dir("build_o");
    let file = write_source(&dir, "answer.my", "function main() { return 6 * 7 }");
    let exe = dir.join("out");
    let output = my_lang(&[
        "build".as_ref(),
        file.as_os_str(),
        "-o".as_ref(),
        exe.as_os_str(),
    ]);
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let ran = Command::new(&exe).output().expect("run built executable");
    assert_eq!(stdout(&ran), "42\n");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn build_names_the_executable_after_the_source() {
    let dir = scratch_dir("build_default");
    let file = write_source(&dir, "seven.my", "function main() { return 3 + 4 }");
    let output = my_lang(&["build".as_ref(), file.as_os_str()]);
    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let ran = Command::new(dir.join("seven"))
        .output()
        .expect("run built executable");
    assert_eq!(stdout(&ran), "7\n");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn build_reports_missing_file() {
    let dir = scratch_dir("build_missing");
    let output = my_lang(&["build".as_ref(), dir.join("nope.my").as_os_str()]);
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("could not read"),
        "stderr: {}",
        stderr(&output)
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn build_reports_parse_errors() {
    let dir = scratch_dir("build_parse");
    let file = write_source(&dir, "bad.my", "function main( { return 1 }");
    let output = my_lang(&["build".as_ref(), file.as_os_str()]);
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("parse error"),
        "stderr: {}",
        stderr(&output)
    );
    let _ = std::fs::remove_dir_all(&dir);
}
