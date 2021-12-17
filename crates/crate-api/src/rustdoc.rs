use std::collections::HashMap;
use std::collections::VecDeque;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RustDocBuilder {
    deps: bool,
    target_directory: Option<std::path::PathBuf>,
}

impl RustDocBuilder {
    pub fn new() -> Self {
        Self {
            deps: false,
            target_directory: None,
        }
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

    pub fn target_directory(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.target_directory = Some(path.into());
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
        let manifest_target_directory;
        let target_dir = if let Some(target_dir) = self.target_directory.as_deref() {
            target_dir
        } else {
            let metadata = cargo_metadata::MetadataCommand::new()
                .manifest_path(manifest_path)
                .no_deps()
                .exec()
                .map_err(|e| crate::Error::new(crate::ErrorKind::ApiParse, e))?;
            manifest_target_directory = metadata
                .target_directory
                .as_path()
                .as_std_path()
                .join("crate-api");
            manifest_target_directory.as_path()
        };

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

        let raw: rustdoc_json_types_fork::Crate = serde_json::from_str(&data).map_err(|e| {
            crate::Error::new(
                crate::ErrorKind::ApiParse,
                format!("Failed when parsing json at {}: {}", json_path.display(), e),
            )
        })?;

        let mut api = crate::Api::new();

        let mut crate_ids = HashMap::new();
        let mut path_ids = HashMap::new();
        let mut deferred_imports = Vec::new();

        let mut unprocessed = VecDeque::new();
        unprocessed.push_back((None, &raw.root));
        while let Some((parent_path_id, raw_item_id)) = unprocessed.pop_front() {
            let raw_item = raw
                .index
                .get(raw_item_id)
                .expect("all item ids are in `index`");

            let crate_id = (raw_item.crate_id != 0).then(|| {
                *crate_ids.entry(raw_item.crate_id).or_insert_with(|| {
                    let raw_crate = raw
                        .external_crates
                        .get(&raw_item.crate_id)
                        .expect("all crate ids are in `external_crates`");
                    let crate_ = crate::Crate::new(&raw_crate.name);
                    api.crates.push(crate_)
                })
            });

            let path_id = raw.paths.get(raw_item_id).map(|raw_path| {
                let mut path = crate::Path::new(raw_path.path.join("::"));
                path.crate_id = crate_id;
                path.span = raw_item.span.clone().map(|raw_span| crate::Span {
                    filename: raw_span.filename,
                    begin: raw_span.begin,
                    end: raw_span.end,
                });
                let path_id = api.paths.push(path);

                if let Some(parent_path_id) = parent_path_id {
                    api.paths
                        .get_mut(parent_path_id)
                        .expect("parent_path_id to always be valid")
                        .children
                        .push(path_id);
                }
                api.root_id.get_or_insert(path_id);
                path_ids.insert(raw_item_id, path_id);

                path_id
            });

            match &raw_item.inner {
                rustdoc_json_types_fork::ItemEnum::Module(module) => {
                    unprocessed.extend(module.items.iter().map(move |i| (path_id, i)));
                }
                rustdoc_json_types_fork::ItemEnum::Import(_) => {
                    deferred_imports.push(raw_item_id);
                }
                rustdoc_json_types_fork::ItemEnum::Trait(trait_) => {
                    unprocessed.extend(trait_.items.iter().map(move |i| (path_id, i)));
                }
                rustdoc_json_types_fork::ItemEnum::Impl(impl_) => {
                    unprocessed.extend(impl_.items.iter().map(move |i| (path_id, i)));
                }
                _ => {
                    assert_ne!(api.root_id, None, "Module should be root");
                    let mut item = crate::Item::new();
                    item.crate_id = crate_id;
                    item.name = raw_item.name.clone();
                    item.span = raw_item.span.clone().map(|raw_span| crate::Span {
                        filename: raw_span.filename,
                        begin: raw_span.begin,
                        end: raw_span.end,
                    });
                    let item_id = api.items.push(item);

                    if let Some(path_id) = path_id {
                        api.paths
                            .get_mut(path_id)
                            .expect("path_id to always be valid")
                            .item_id = Some(item_id);
                    }
                }
            }
        }

        for raw_item_id in deferred_imports {
            let raw_item = raw
                .index
                .get(raw_item_id)
                .expect("all item ids are in `index`");
            let import = match &raw_item.inner {
                rustdoc_json_types_fork::ItemEnum::Import(import) => import,
                _ => unreachable!("deferred_imports only contains imports"),
            };
            let raw_target_id = import.id.as_ref().unwrap();
            let target_path_id = *path_ids.get(raw_target_id).unwrap();
            let target_path = api
                .paths
                .get(target_path_id)
                .expect("path_id to always be valid")
                .clone();

            let path_id = *path_ids.get(raw_item_id).unwrap();
            let path = api
                .paths
                .get_mut(path_id)
                .expect("path_id to always be valid");
            path.item_id = target_path.item_id;
            path.children = target_path.children.clone();
        }

        Ok(api)
    }
}

impl Default for RustDocBuilder {
    fn default() -> Self {
        Self::new()
    }
}
