use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RustDocBuilder {
    deps: bool,
}

impl RustDocBuilder {
    pub fn new() -> Self {
        Self { deps: false }
    }

    /// Include dependencies
    ///
    /// Reasons to have this disabled:
    /// - Faster API extraction
    /// - Less likely to hit bugs in rustdoc, like
    ///   - rust-lang/rust#89097
    ///   - rust-lang/rust#83718
    ///
    /// Reasons to have this enabled:
    /// - Check for accidental inclusion of dependencies in your API
    /// - Detect breaking changes from dependencies in your API
    pub fn deps(mut self, yes: bool) -> Self {
        self.deps = yes;
        self
    }

    pub fn dump_raw(self, manifest_path: &std::path::Path) -> Result<String, crate::Error> {
        let json_path = self._dump_raw(manifest_path)?;
        std::fs::read_to_string(&json_path).map_err(|e| {
            crate::Error::new(
                crate::ErrorKind::ApiParse,
                format!("Failed when loading {}: {}", json_path.display(), e),
            )
        })
    }

    pub fn into_api(self, manifest_path: &std::path::Path) -> Result<crate::Api, crate::Error> {
        let json_path = self._dump_raw(manifest_path)?;
        Self::_parse_api(&json_path)
    }

    fn _dump_raw(
        self,
        manifest_path: &std::path::Path,
    ) -> Result<std::path::PathBuf, crate::Error> {
        let metadata = cargo_metadata::MetadataCommand::new()
            .manifest_path(manifest_path)
            .no_deps()
            .exec()
            .map_err(|e| crate::Error::new(crate::ErrorKind::ApiParse, e))?;
        let target_dir = metadata
            .target_directory
            .as_path()
            .as_std_path()
            .join("crate-api");

        let mut cmd = std::process::Command::new("cargo");
        cmd.env(
            "RUSTDOCFLAGS",
            "-Z unstable-options --document-hidden-items --output-format=json",
        )
        // HACK: Avoid compilation conflicts between nightly and regular toolchains
        .env("CARGO_TARGET_DIR", &target_dir)
        .args(["+nightly", "doc", "--all-features"])
        .arg("--manifest-path")
        .arg(manifest_path);
        if !self.deps {
            // HACK: Trying to reduce chance of hitting
            // - rust-lang/rust#89097
            // - rust-lang/rust#83718
            cmd.arg("--no-deps");
        }

        let output = cmd
            .output()
            .map_err(|e| crate::Error::new(crate::ErrorKind::ApiParse, e))?;
        if !output.status.success() {
            return Err(crate::Error::new(
                crate::ErrorKind::ApiParse,
                format!(
                    "Failed when running cargo-doc on {}: {}",
                    manifest_path.display(),
                    String::from_utf8_lossy(&output.stderr)
                ),
            ));
        }

        Ok(target_dir.join("doc/cargo_api.json"))
    }

    fn _parse_api(json_path: &std::path::Path) -> Result<crate::Api, crate::Error> {
        let data = std::fs::read_to_string(&json_path).map_err(|e| {
            crate::Error::new(
                crate::ErrorKind::ApiParse,
                format!("Failed when loading {}: {}", json_path.display(), e),
            )
        })?;

        let docs: rustdoc_json_types_fork::Crate = serde_json::from_str(&data).map_err(|e| {
            crate::Error::new(
                crate::ErrorKind::ApiParse,
                format!("Failed when parsing json at {}: {}", json_path.display(), e),
            )
        })?;

        let mut api = crate::Api::new();

        let mut crate_ids = HashMap::new();
        for (raw_id, raw) in &docs.external_crates {
            let crate_ = crate::Crate::new(&raw.name);
            let crate_id = api.crates.push(crate_);
            crate_ids.insert(raw_id, crate_id);
        }

        Ok(api)
    }
}

impl Default for RustDocBuilder {
    fn default() -> Self {
        Self::new()
    }
}
