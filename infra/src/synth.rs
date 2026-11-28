//! Turning a `terrars` stack into the committed `out/<stack>/main.tf.json`.

use anyhow::Context;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

/// The OpenTofu variable that carries the state encryption passphrase, from
/// the `TOFU_STATE_PASSPHRASE` secret as `TF_VAR_tofu_state_passphrase`.
pub const PASSPHRASE_VARIABLE: &str = "tofu_state_passphrase";

/// The `http` backend for a stack's state, served by the
/// `jimhillr-tofu-state` Worker with D1 (decision 0009). Only the methods are
/// here: the workflows pass the stack's URL (`address`, `lock_address` and
/// `unlock_address`, all `<worker>/states/<stack>`) with `-backend-config`,
/// and the password in `TF_HTTP_PASSWORD`, so nothing environment-specific
/// or secret is in the code.
pub fn backend() -> Value {
    json!({
        "http": {
            "lock_method": "LOCK",
            "unlock_method": "UNLOCK",
            "update_method": "POST",
            "username": "tofu",
            "retry_max": 5,
        }
    })
}

/// The backend URL path for a stack, under the Worker's origin.
pub fn state_url_path(stack: &str) -> String {
    format!("/states/{stack}")
}

/// The `-backend-config` arguments that point a stack at its state URL, one
/// per line, for `tofu init $(infra backend-config <stack> <origin>)`. The
/// stack must be one we synthesize, and the origin an `https://` URL.
pub fn backend_config_args(stack: &str, origin: &str) -> anyhow::Result<Vec<String>> {
    anyhow::ensure!(
        stack
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
        "not a stack name: {stack}"
    );
    let origin = origin.trim_end_matches('/');
    anyhow::ensure!(
        origin.starts_with("https://") && !origin[8..].contains('/'),
        "the state origin must be https://<host>, got {origin}"
    );
    let url = format!("{origin}{}", state_url_path(stack));
    Ok(["address", "lock_address", "unlock_address"]
        .iter()
        .map(|key| format!("-backend-config={key}={url}"))
        .collect())
}

/// The OpenTofu variable that carries the previous passphrase while the
/// state is re-encrypted after a rotation, from `TOFU_STATE_PASSPHRASE_OLD`
/// as `TF_VAR_tofu_state_passphrase_old`. Empty, and unused, at generation 1.
pub const OLD_PASSPHRASE_VARIABLE: &str = "tofu_state_passphrase_old";

/// The key provider and method name for a passphrase generation. Generation
/// 1 is `state`, the name every state was first written with; later ones are
/// `gen_<n>`. OpenTofu stores each key's PBKDF2 salt in the state under the
/// key provider's name, so a new passphrase needs a new name: the old key
/// can then be the fallback under the name it encrypted with.
pub fn key_name(generation: u32) -> String {
    if generation <= 1 {
        "state".to_string()
    } else {
        format!("gen_{generation}")
    }
}

/// OpenTofu state and plan encryption (decision 0007, item 7): AES-GCM with
/// a key from a PBKDF2 passphrase, enforced so an unencrypted state is never
/// written. Defence in depth: IAC-002 already keeps secrets out of state.
///
/// In JSON the `method` and `keys` references are bare traversals, not
/// `${...}` templates, which OpenTofu rejects here (`tofu validate` doesn't
/// notice; the test below runs a real apply).
pub fn encryption() -> Value {
    encryption_for(1)
}

/// The encryption block for a passphrase generation. From generation 2 on,
/// the current passphrase encrypts under [`key_name`] of that generation, and
/// the previous one (`tofu_state_passphrase_old`) is the fallback, so state
/// written before the rotation still reads until it is re-encrypted
/// (`tofu.sh rekey`). The committed stacks hold generation 1; `tofu.sh`
/// passes a later one in `TF_ENCRYPTION`, which OpenTofu merges over it.
pub fn encryption_for(generation: u32) -> Value {
    let current = key_name(generation);
    let passphrase = |variable: &str| json!({ "passphrase": format!("${{var.{variable}}}") });
    let keys = |name: &str| json!({ "keys": format!("key_provider.pbkdf2.{name}") });
    let mut method = json!({ "method": format!("method.aes_gcm.{current}"), "enforced": true });
    let mut providers = serde_json::Map::new();
    let mut methods = serde_json::Map::new();
    providers.insert(current.clone(), passphrase(PASSPHRASE_VARIABLE));
    methods.insert(current.clone(), keys(&current));
    if generation > 1 {
        let previous = key_name(generation - 1);
        providers.insert(previous.clone(), passphrase(OLD_PASSPHRASE_VARIABLE));
        methods.insert(previous.clone(), keys(&previous));
        method["fallback"] = json!({ "method": format!("method.aes_gcm.{previous}") });
    }
    json!({
        "key_provider": { "pbkdf2": providers },
        "method": { "aes_gcm": methods },
        "state": method,
        "plan": method,
    })
}

/// A synthesized stack.
pub struct Synth {
    /// The stack's name and directory under `out/`.
    pub name: String,
    /// The Terraform JSON. `serde_json` keeps object keys sorted, so the
    /// committed file diffs cleanly.
    pub json: Value,
    imports: Vec<Value>,
    moves: Vec<Value>,
}

impl Synth {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            json: Value::Null,
            imports: Vec::new(),
            moves: Vec::new(),
        }
    }

    /// Bring an existing resource under IaC: `to` is its address, such as
    /// `auth0_client.cookbook`, and `id` the provider's import ID.
    ///
    /// terrars 0.4.0 has no `import` blocks, so they are added to the JSON
    /// here. Remove this once the upstream PR adding `Stack::add_import`
    /// (infra/upstream/terrars-import-blocks.patch) is released.
    pub fn import(&mut self, to: String, id: &str) {
        self.imports.push(json!({ "to": to, "id": id }));
    }

    /// Rename a resource's address from `from` to `to`, such as
    /// `github_repository.monorepo` to `github_repository.jimhillr`, so
    /// OpenTofu moves it in state instead of destroying and recreating it.
    ///
    /// terrars 0.4.0 has no `moved` blocks either, so they are added to the
    /// JSON here, like the import blocks.
    pub fn moved(&mut self, from: &str, to: String) {
        self.moves.push(json!({ "from": from, "to": to }));
    }

    /// Serialize the stack, then add the import and moved blocks, the HTTP backend and
    /// state encryption.
    pub fn finish(mut self, stack: &terrars::Stack) -> anyhow::Result<Self> {
        let bytes = stack
            .serialize(Path::new("unused.tfstate"))
            .map_err(|e| anyhow::anyhow!("serializing stack {}: {e}", self.name))?;
        let mut json: Value = serde_json::from_slice(&bytes)
            .with_context(|| format!("stack {} is not valid JSON", self.name))?;
        let object = json
            .as_object_mut()
            .context("a stack serializes to a JSON object")?;
        let terraform = object
            .entry("terraform")
            .or_insert_with(|| json!({}))
            .as_object_mut()
            .context("the terraform block is an object")?;
        terraform.insert("backend".into(), backend());
        terraform.insert("encryption".into(), encryption());
        let variables = object
            .entry("variable")
            .or_insert_with(|| json!({}))
            .as_object_mut()
            .context("the variable block is an object")?;
        variables.insert(
            PASSPHRASE_VARIABLE.into(),
            json!({ "type": "string", "sensitive": true, "nullable": false }),
        );
        variables.insert(
            OLD_PASSPHRASE_VARIABLE.into(),
            json!({ "type": "string", "sensitive": true, "nullable": false, "default": "" }),
        );
        if !self.imports.is_empty() {
            object.insert(
                "import".into(),
                Value::Array(std::mem::take(&mut self.imports)),
            );
        }
        if !self.moves.is_empty() {
            object.insert(
                "moved".into(),
                Value::Array(std::mem::take(&mut self.moves)),
            );
        }
        self.json = json;
        Ok(self)
    }

    /// The file contents: pretty JSON with a trailing newline.
    pub fn contents(&self) -> anyhow::Result<String> {
        Ok(format!("{}\n", serde_json::to_string_pretty(&self.json)?))
    }

    /// The file's path under `out_dir`.
    pub fn path(&self, out_dir: &Path) -> PathBuf {
        out_dir.join(&self.name).join("main.tf.json")
    }
}

/// Write every stack to `out_dir`.
pub fn write(stacks: &[Synth], out_dir: &Path) -> anyhow::Result<()> {
    for stack in stacks {
        let path = stack.path(out_dir);
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
        }
        std::fs::write(&path, stack.contents()?)
            .with_context(|| format!("writing {}", path.display()))?;
    }
    Ok(())
}

/// The stacks whose file under `out_dir` is missing or differs from a fresh
/// synth.
pub fn stale(stacks: &[Synth], out_dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut stale = Vec::new();
    for stack in stacks {
        let path = stack.path(out_dir);
        let committed = std::fs::read_to_string(&path).unwrap_or_default();
        if committed != stack.contents()? {
            stale.push(path);
        }
    }
    Ok(stale)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constructs::{Auth0App, WebApp};
    use crate::stacks::auth0;

    fn stack() -> anyhow::Result<Synth> {
        auth0::synth(&[WebApp::new("cookbook").sign_in(Auth0App::import("abc", "cookbook"))])
    }

    #[test]
    fn state_uses_the_http_backend_with_locking() -> anyhow::Result<()> {
        let synth = stack()?;
        let http = &synth.json["terraform"]["backend"]["http"];
        assert_eq!(http["lock_method"], "LOCK");
        assert_eq!(http["unlock_method"], "UNLOCK");
        // The URL and password come from the workflow, never the code.
        assert!(http.get("address").is_none() && http.get("password").is_none());
        assert!(synth.json["terraform"]["backend"].get("local").is_none());
        assert_eq!(state_url_path("auth0"), "/states/auth0");
        Ok(())
    }

    #[test]
    fn backend_config_points_every_address_at_the_stack() -> anyhow::Result<()> {
        let args = backend_config_args("auth0", "https://state.example.workers.dev/")?;
        assert_eq!(
            args,
            [
                "-backend-config=address=https://state.example.workers.dev/states/auth0",
                "-backend-config=lock_address=https://state.example.workers.dev/states/auth0",
                "-backend-config=unlock_address=https://state.example.workers.dev/states/auth0",
            ]
        );
        assert!(backend_config_args("auth0", "http://state.example").is_err());
        assert!(backend_config_args("auth0", "https://state.example/other").is_err());
        assert!(backend_config_args("../x", "https://state.example").is_err());
        Ok(())
    }

    #[test]
    fn state_and_plans_are_encrypted_with_a_sensitive_passphrase() -> anyhow::Result<()> {
        let synth = stack()?;
        let encryption = &synth.json["terraform"]["encryption"];
        assert_eq!(encryption["state"]["enforced"], true);
        assert_eq!(encryption["plan"]["enforced"], true);
        assert_eq!(
            encryption["key_provider"]["pbkdf2"]["state"]["passphrase"],
            "${var.tofu_state_passphrase}"
        );
        assert_eq!(
            synth.json["variable"]["tofu_state_passphrase"]["sensitive"],
            true
        );
        Ok(())
    }

    /// Applies a stack with our encryption block and OpenTofu's built-in
    /// `terraform_data`, so it needs no provider or network, and checks the
    /// state is encrypted and a wrong passphrase can't read it. Skipped when
    /// `tofu` isn't on PATH (CI installs it with mise).
    #[test]
    fn opentofu_accepts_the_encryption_block() -> anyhow::Result<()> {
        use std::process::Command;
        if Command::new("tofu").arg("version").output().is_err() {
            eprintln!("skipped: tofu is not on PATH");
            return Ok(());
        }
        let dir = tempfile::tempdir()?;
        let config = json!({
            "terraform": {
                "backend": { "local": { "path": "test.tfstate" } },
                "encryption": encryption(),
            },
            "variable": { PASSPHRASE_VARIABLE: { "type": "string", "sensitive": true } },
            "resource": { "terraform_data": { "probe": { "input": "probe" } } },
        });
        std::fs::write(dir.path().join("main.tf.json"), config.to_string())?;
        let tofu = |passphrase: &str, args: &[&str]| -> anyhow::Result<bool> {
            Ok(Command::new("tofu")
                .args(args)
                .arg("-no-color")
                .current_dir(dir.path())
                .env(format!("TF_VAR_{PASSPHRASE_VARIABLE}"), passphrase)
                .output()?
                .status
                .success())
        };
        let right = "a-long-test-passphrase-for-the-probe";
        assert!(tofu(right, &["init", "-input=false"])?, "tofu init failed");
        assert!(
            tofu(right, &["apply", "-auto-approve", "-input=false"])?,
            "tofu apply failed"
        );
        let state = std::fs::read_to_string(dir.path().join("test.tfstate"))?;
        assert!(state.contains("encrypted_data"), "state is not encrypted");
        assert!(!state.contains("\"probe\""), "state shows plain values");
        assert!(!tofu(
            "another-wrong-passphrase-entirely",
            &["plan", "-input=false"]
        )?);
        Ok(())
    }

    #[test]
    fn a_later_generation_falls_back_to_the_previous_key() {
        let first = encryption_for(1);
        assert_eq!(first, encryption());
        assert!(first["state"].get("fallback").is_none());
        let third = encryption_for(3);
        let pbkdf2 = &third["key_provider"]["pbkdf2"];
        assert_eq!(
            pbkdf2["gen_3"]["passphrase"],
            "${var.tofu_state_passphrase}"
        );
        assert_eq!(
            pbkdf2["gen_2"]["passphrase"],
            "${var.tofu_state_passphrase_old}"
        );
        for block in ["state", "plan"] {
            assert_eq!(third[block]["method"], "method.aes_gcm.gen_3");
            assert_eq!(third[block]["enforced"], true);
            assert_eq!(third[block]["fallback"]["method"], "method.aes_gcm.gen_2");
        }
        assert_eq!(
            encryption_for(2)["state"]["fallback"]["method"],
            "method.aes_gcm.state",
            "generation 2 reads the state generation 1 wrote"
        );
    }

    /// A rotation end to end with real OpenTofu (the proof
    /// infra/docs/issues/iac-rotate-state-passphrase.md asked for): state written at
    /// generation 1 still plans at generation 2 through the fallback, `state
    /// pull | state push` (what `tofu.sh rekey` runs) re-encrypts it under the
    /// new key, and after that the old passphrase isn't needed. Skipped when
    /// `tofu` isn't on PATH.
    #[test]
    fn opentofu_rotates_the_passphrase() -> anyhow::Result<()> {
        use std::io::Write;
        use std::process::{Command, Stdio};
        if Command::new("tofu").arg("version").output().is_err() {
            eprintln!("skipped: tofu is not on PATH");
            return Ok(());
        }
        let dir = tempfile::tempdir()?;
        let mut synth = Synth::new("probe");
        synth.json = json!({
            "terraform": {
                "backend": { "local": { "path": "test.tfstate" } },
                "encryption": encryption(),
            },
            "variable": {
                PASSPHRASE_VARIABLE: { "type": "string", "sensitive": true, "nullable": false },
                OLD_PASSPHRASE_VARIABLE: { "type": "string", "sensitive": true, "nullable": false, "default": "" },
            },
            "resource": { "terraform_data": { "probe": { "input": "probe" } } },
        });
        std::fs::write(dir.path().join("main.tf.json"), synth.contents()?)?;
        let (a, b, c) = (
            "the-first-long-test-passphrase",
            "the-second-long-test-passphrase",
            "the-third-long-test-passphrase",
        );
        let tofu = |generation: u32, new: &str, old: &str, args: &[&str]| {
            let mut command = Command::new("tofu");
            command
                .args(args)
                .current_dir(dir.path())
                .env(format!("TF_VAR_{PASSPHRASE_VARIABLE}"), new)
                .env(format!("TF_VAR_{OLD_PASSPHRASE_VARIABLE}"), old)
                .env_remove("TF_ENCRYPTION");
            if generation > 1 {
                command.env("TF_ENCRYPTION", encryption_for(generation).to_string());
            }
            command
        };
        let ok = |mut command: Command| -> anyhow::Result<bool> {
            Ok(command.arg("-no-color").output()?.status.success())
        };
        // No changes, and the state read: exit 0 with -detailed-exitcode.
        let plans = |generation: u32, new: &str, old: &str| {
            ok(tofu(
                generation,
                new,
                old,
                &["plan", "-input=false", "-detailed-exitcode"],
            ))
        };
        let rekey = |generation: u32, new: &str, old: &str| -> anyhow::Result<bool> {
            let pulled = tofu(generation, new, old, &["state", "pull"]).output()?;
            anyhow::ensure!(pulled.status.success(), "state pull failed");
            let mut push = tofu(generation, new, old, &["state", "push", "-no-color", "-"])
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .spawn()?;
            push.stdin
                .take()
                .context("stdin")?
                .write_all(&pulled.stdout)?;
            Ok(push.wait()?.success())
        };
        let meta = || -> anyhow::Result<Vec<String>> {
            let state: Value =
                serde_json::from_str(&std::fs::read_to_string(dir.path().join("test.tfstate"))?)?;
            Ok(state["meta"]
                .as_object()
                .context("meta")?
                .keys()
                .cloned()
                .collect())
        };

        assert!(
            ok(tofu(1, a, "", &["init", "-input=false"]))?,
            "init failed"
        );
        assert!(
            ok(tofu(1, a, "", &["apply", "-auto-approve", "-input=false"]))?,
            "apply failed"
        );
        assert_eq!(meta()?, ["key_provider.pbkdf2.state"]);

        // Rotated to generation 2: the old key reads, a wrong one doesn't.
        assert!(
            plans(2, b, a)?,
            "generation 2 can't read generation 1 state"
        );
        assert!(!plans(2, b, c)?, "a wrong old passphrase read the state");
        assert!(rekey(2, b, a)?, "rekey to generation 2 failed");
        assert_eq!(meta()?, ["key_provider.pbkdf2.gen_2"]);
        assert!(
            plans(2, b, "")?,
            "re-encrypted state still needs the old key"
        );
        assert!(!plans(1, a, "")?, "the old key still reads the new state");

        // And again: generation 3 falls back to gen_2.
        assert!(
            plans(3, c, b)?,
            "generation 3 can't read generation 2 state"
        );
        assert!(rekey(3, c, b)?, "rekey to generation 3 failed");
        assert_eq!(meta()?, ["key_provider.pbkdf2.gen_3"]);
        assert!(plans(3, c, "")?);
        Ok(())
    }

    #[test]
    fn keys_are_sorted() -> anyhow::Result<()> {
        let contents = stack()?.contents()?;
        let import = contents.find("\"import\"").context("import key")?;
        let resource = contents.find("\"resource\"").context("resource key")?;
        let terraform = contents.find("\"terraform\"").context("terraform key")?;
        assert!(import < resource && resource < terraform);
        Ok(())
    }

    #[test]
    fn write_then_check_finds_nothing_stale() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let stacks = vec![stack()?];
        assert_eq!(
            stale(&stacks, dir.path())?.len(),
            1,
            "missing file is stale"
        );
        write(&stacks, dir.path())?;
        assert!(stale(&stacks, dir.path())?.is_empty());
        std::fs::write(stacks[0].path(dir.path()), "{}\n")?;
        assert_eq!(stale(&stacks, dir.path())?.len(), 1, "edited file is stale");
        Ok(())
    }
}
