#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Raw {
    pub manifest_path: std::path::PathBuf,
    pub rustdoc: Option<rustdoc_json_types_fork::Crate>,
    pub manifest: Option<crate_api::manifest::Manifest>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Api {
    pub manifest_path: std::path::PathBuf,
    pub api: crate_api::Api,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Diff {
    pub manifest_path: std::path::PathBuf,
    pub against: Source,
    pub before: crate_api::Api,
    pub after: crate_api::Api,
    pub diffs: Vec<crate_api::diff::Diff>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    Git(String),
    Path(std::path::PathBuf),
    Registry(String),
}

pub fn render_api_markdown(
    writer: &mut dyn std::io::Write,
    api: &crate_api::Api,
) -> Result<(), eyre::Report> {
    let root_id = *match api.root_id.as_ref() {
        Some(root_id) => root_id,
        None => return Ok(()),
    };

    let root = api.paths.get(root_id).unwrap();
    let _ = writeln!(writer, "# `{}`", root.path);
    let _ = writeln!(writer);

    let mut paths: std::collections::VecDeque<crate_api::PathId> = Default::default();
    paths.extend(root.children.iter().copied().rev());
    while let Some(next_path_id) = paths.pop_back() {
        let next_path = api.paths.get(next_path_id).unwrap();

        match next_path.kind {
            crate_api::PathKind::Module => {
                for _ in 0..(next_path.path.matches("::").count() + 1) {
                    let _ = write!(writer, "#");
                }
                let _ = writeln!(writer, " `{}`", next_path.path);
                let _ = writeln!(writer);

                if let Some(crate_id) = next_path.crate_id.clone() {
                    let crate_ = api.crates.get(crate_id).unwrap();
                    let _ = writeln!(writer, "*from crate `{}`*", crate_.name);
                    let _ = writeln!(writer);
                }

                let (mut modules, mut other): (Vec<_>, Vec<_>) = next_path
                    .children
                    .iter()
                    .copied()
                    .partition(|next_path_id| {
                        let next_path = api.paths.get(*next_path_id).unwrap();
                        next_path.kind == crate_api::PathKind::Module
                    });
                modules.sort_unstable_by_key(|next_path_id| {
                    let next_path = api.paths.get(*next_path_id).unwrap();
                    (next_path.kind, next_path.path.as_str())
                });
                other.sort_unstable_by_key(|next_path_id| {
                    let next_path = api.paths.get(*next_path_id).unwrap();
                    (next_path.kind, next_path.path.as_str())
                });
                paths.extend(modules.into_iter().rev());
                paths.extend(other.into_iter().rev());
            }
            _ => {
                let _ = writeln!(writer, "**`{}`** *({:?})*", next_path.path, next_path.kind);
                let _ = writeln!(writer);

                if let Some(crate_id) = next_path.crate_id.clone() {
                    let crate_ = api.crates.get(crate_id).unwrap();
                    let _ = writeln!(writer, "*from crate `{}`*", crate_.name);
                    let _ = writeln!(writer);
                }

                let mut other = next_path.children.clone();
                other.sort_unstable_by_key(|next_path_id| {
                    let next_path = api.paths.get(*next_path_id).unwrap();
                    (next_path.kind, next_path.path.as_str())
                });
                paths.extend(other.into_iter().rev());
            }
        }
    }

    if !api.features.is_empty() {
        let _ = writeln!(writer, "## Feature Flags");
        let _ = writeln!(writer);
        for details in api.features.values() {
            match details {
                crate_api::AnyFeature::Feature(feature) => {
                    let _ = writeln!(writer, "`{}`", feature.name);
                    for dep in &feature.dependencies {
                        let _ = writeln!(writer, "- `{}`", dep);
                    }
                    let _ = writeln!(writer);
                }
                crate_api::AnyFeature::OptionalDependency(dep) => {
                    if let Some(package) = dep.package.as_deref() {
                        let _ = writeln!(writer, "`{}` *(dependency `{}`)*", dep.name, package);
                    } else {
                        let _ = writeln!(writer, "`{}` *(dependency)*", dep.name);
                    }
                    let _ = writeln!(writer);
                }
            }
        }
    }

    if !api.crates.is_empty() {
        let _ = writeln!(writer, "## Public Dependencies");
        let _ = writeln!(writer);

        for (_, crate_) in api.crates.iter() {
            let _ = writeln!(
                writer,
                "- `{}` (version {})",
                crate_.name,
                crate_
                    .version
                    .as_ref()
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "unknown".into())
            );
        }
        let _ = writeln!(writer);
    }

    Ok(())
}
