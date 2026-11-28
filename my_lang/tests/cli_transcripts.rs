//! CLI transcript tests: run the real `my_lang` binary on the programs in
//! `tests/cmd/<case>.in/` and compare its output with the transcripts in
//! `tests/cmd/`. These are the `trycmd` evals in `evals.toml`, one or more
//! case files per requirement, named `LANG-00n-<what>.toml`.
//!
//! To update a transcript after an intended output change, run
//! `TRYCMD=overwrite cargo test --test cli_transcripts`.

#[test]
fn cli_transcripts() {
    trycmd::TestCases::new().case("tests/cmd/*.toml");
}
