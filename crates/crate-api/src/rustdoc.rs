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
        let manifest = std::fs::read_to_string(manifest_path).map_err(|e| {
            crate::Error::new(
                crate::ErrorKind::ApiParse,
                format!("Failed when reading {}: {}", manifest_path.display(), e),
            )
        })?;
        let manifest: toml_edit::Document = manifest.parse().map_err(|e| {
            crate::Error::new(
                crate::ErrorKind::ApiParse,
                format!("Failed to parse {}: {}", manifest_path.display(), e),
            )
        })?;
        let crate_name = manifest["package"]["name"].as_str().ok_or_else(|| {
            crate::Error::new(
                crate::ErrorKind::ApiParse,
                format!(
                    "Failed to parse {}: invalid package.name",
                    manifest_path.display()
                ),
            )
        })?;

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
                // HACK: Avoid potential errors when mixing toolchains
                .join("crate-api");
            manifest_target_directory.as_path()
        };

        let mut cmd = std::process::Command::new("cargo");
        cmd.env(
            "RUSTDOCFLAGS",
            "-Z unstable-options --document-hidden-items --output-format=json",
        )
        .args(["+nightly", "doc", "--all-features"])
        .arg("--manifest-path")
        .arg(manifest_path)
        .arg("--target-dir")
        .arg(target_dir);
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

        let json_path = target_dir.join(format!("doc/{}.json", crate_name));
        std::fs::read_to_string(&json_path).map_err(|e| {
            crate::Error::new(
                crate::ErrorKind::ApiParse,
                format!("Failed when loading {}: {}", json_path.display(), e),
            )
        })
    }

    pub fn into_api(self, manifest_path: &std::path::Path) -> Result<crate::Api, crate::Error> {
        let raw = self.dump_raw(manifest_path)?;
        parse_raw(&raw, manifest_path)
    }
}

impl Default for RustDocBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub fn parse_raw(raw: &str, manifest_path: &std::path::Path) -> Result<crate::Api, crate::Error> {
    RustDocParser::new().parse(raw, manifest_path)
}

#[derive(Default)]
struct RustDocParser {
    api: crate::Api,
    crate_ids: HashMap<u32, crate::CrateId>,
    path_ids: HashMap<rustdoc_json_types_fork::Id, crate::PathId>,
}

impl RustDocParser {
    fn new() -> Self {
        Self::default()
    }

    fn parse(
        mut self,
        raw: &str,
        manifest_path: &std::path::Path,
    ) -> Result<crate::Api, crate::Error> {
        let raw: rustdoc_json_types_fork::Crate = serde_json::from_str(raw).map_err(|e| {
            crate::Error::new(
                crate::ErrorKind::ApiParse,
                format!(
                    "Failed when parsing json for {}: {}",
                    manifest_path.display(),
                    e
                ),
            )
        })?;

        let mut deferred_imports = VecDeque::new();

        let mut unprocessed = VecDeque::new();
        unprocessed.push_back((None, &raw.root));
        while let Some((parent_path_id, raw_item_id)) = unprocessed.pop_front() {
            let raw_item = raw
                .index
                .get(raw_item_id)
                .expect("all item ids are in `index`");

            let crate_id = (raw_item.crate_id != 0).then(|| {
                *self.crate_ids.entry(raw_item.crate_id).or_insert_with(|| {
                    let raw_crate = raw
                        .external_crates
                        .get(&raw_item.crate_id)
                        .expect("all crate ids are in `external_crates`");
                    let crate_ = crate::Crate::new(&raw_crate.name);
                    self.api.crates.push(crate_)
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
                let path_id = self.api.paths.push(path);

                if let Some(parent_path_id) = parent_path_id {
                    self.api
                        .paths
                        .get_mut(parent_path_id)
                        .expect("parent_path_id to always be valid")
                        .children
                        .push(path_id);
                }
                self.api.root_id.get_or_insert(path_id);
                self.path_ids.insert(raw_item_id.clone(), path_id);

                path_id
            });

            match &raw_item.inner {
                rustdoc_json_types_fork::ItemEnum::Module(module) => {
                    unprocessed.extend(module.items.iter().map(move |i| (path_id, i)));
                }
                rustdoc_json_types_fork::ItemEnum::Import(_) => {
                    deferred_imports.push_back(raw_item_id);
                }
                rustdoc_json_types_fork::ItemEnum::Trait(trait_) => {
                    unprocessed.extend(trait_.items.iter().map(move |i| (path_id, i)));
                }
                rustdoc_json_types_fork::ItemEnum::Impl(impl_) => {
                    unprocessed.extend(impl_.items.iter().map(move |i| (path_id, i)));
                }
                _ => {
                    assert_ne!(self.api.root_id, None, "Module should be root");
                    let mut item = crate::Item::new();
                    item.crate_id = crate_id;
                    item.name = raw_item.name.clone();
                    item.span = raw_item.span.clone().map(|raw_span| crate::Span {
                        filename: raw_span.filename,
                        begin: raw_span.begin,
                        end: raw_span.end,
                    });
                    let item_id = self.api.items.push(item);

                    if let Some(path_id) = path_id {
                        self.api
                            .paths
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
            let target_path_id = *self.path_ids.get(raw_target_id).unwrap();
            let target_path = self
                .api
                .paths
                .get(target_path_id)
                .expect("path_id to always be valid")
                .clone();

            let path_id = *self.path_ids.get(raw_item_id).unwrap();
            let path = self
                .api
                .paths
                .get_mut(path_id)
                .expect("path_id to always be valid");
            path.item_id = target_path.item_id;
            path.children = target_path.children.clone();
        }

        Ok(self.api)
    }
}
