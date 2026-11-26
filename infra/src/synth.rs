//! Turning a `terrars` stack into the committed `out/<stack>/main.tf.json`.

use anyhow::Context;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

/// The private R2 bucket that holds every stack's state (decision 0009).
pub const STATE_BUCKET: &str = "jimhillr-infra-state";

/// The OpenTofu variable that carries the state encryption passphrase, from
/// the `TOFU_STATE_PASSPHRASE` secret as `TF_VAR_tofu_state_passphrase`.
pub const PASSPHRASE_VARIABLE: &str = "tofu_state_passphrase";

/// The `s3` backend for a stack's state in R2 (decision 0009). The endpoint
/// (it holds the account ID) and the access key come from the environment
/// (`AWS_ENDPOINT_URL_S3`, `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`), never
/// from this code. terrars' `BackendS3` has no `skip_s3_checksum`, which R2
/// needs, so the block is written here.
pub fn backend(stack: &str) -> Value {
    json!({
        "s3": {
            "bucket": STATE_BUCKET,
            "key": format!("{stack}.tfstate"),
            "region": "auto",
            "use_lockfile": true,
            "use_path_style": true,
            "skip_credentials_validation": true,
            "skip_region_validation": true,
            "skip_requesting_account_id": true,
            "skip_metadata_api_check": true,
            "skip_s3_checksum": true,
        }
    })
}

/// OpenTofu state and plan encryption (decision 0007, item 7): AES-GCM with
/// a key from a PBKDF2 passphrase, enforced so an unencrypted state is never
/// written. Defence in depth: IAC-002 already keeps secrets out of state.
///
/// In JSON the `method` and `keys` references are bare traversals, not
/// `${...}` templates, which OpenTofu rejects here (`tofu validate` doesn't
/// notice; the test below runs a real apply).
pub fn encryption() -> Value {
    let method = json!({ "method": "method.aes_gcm.state", "enforced": true });
    json!({
        "key_provider": { "pbkdf2": { "state": {
            "passphrase": format!("${{var.{PASSPHRASE_VARIABLE}}}"),
        } } },
        "method": { "aes_gcm": { "state": { "keys": "key_provider.pbkdf2.state" } } },
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
}

impl Synth {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            json: Value::Null,
            imports: Vec::new(),
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

    /// Serialize the stack, then add the import blocks, the R2 backend and
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
        terraform.insert("backend".into(), backend(&self.name));
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
        if !self.imports.is_empty() {
            object.insert(
                "import".into(),
                Value::Array(std::mem::take(&mut self.imports)),
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
    fn state_lives_in_r2_with_a_lock_file() -> anyhow::Result<()> {
        let synth = stack()?;
        let s3 = &synth.json["terraform"]["backend"]["s3"];
        assert_eq!(s3["bucket"], "jimhillr-infra-state");
        assert_eq!(s3["key"], "auth0.tfstate");
        assert_eq!(s3["use_lockfile"], true);
        assert!(s3.get("endpoints").is_none() && s3.get("access_key").is_none());
        assert!(synth.json["terraform"]["backend"].get("local").is_none());
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
