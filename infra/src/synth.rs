//! Turning a `terrars` stack into the committed `out/<stack>/main.tf.json`.

use anyhow::Context;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

/// Where each stack's state lives, relative to its `out/<stack>/` directory.
/// `infra/state/` is not committed on `main`; the apply job keeps it,
/// encrypted, on the `infra-state` branch (decision 0007, item 7).
pub fn state_path(stack: &str) -> PathBuf {
    Path::new("../../state").join(format!("{stack}.tfstate"))
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

    /// Serialize the stack and add the import blocks.
    pub fn finish(mut self, stack: &terrars::Stack) -> anyhow::Result<Self> {
        let bytes = stack
            .serialize(&state_path(&self.name))
            .map_err(|e| anyhow::anyhow!("serializing stack {}: {e}", self.name))?;
        let mut json: Value = serde_json::from_slice(&bytes)
            .with_context(|| format!("stack {} is not valid JSON", self.name))?;
        if !self.imports.is_empty() {
            let object = json
                .as_object_mut()
                .context("a stack serializes to a JSON object")?;
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
    fn state_lives_outside_the_committed_output() -> anyhow::Result<()> {
        let synth = stack()?;
        assert_eq!(
            synth.json["terraform"]["backend"]["local"]["path"],
            "../../state/auth0.tfstate"
        );
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
