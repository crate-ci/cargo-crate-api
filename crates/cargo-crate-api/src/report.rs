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

                if let Some(crate_id) = next_path.crate_id {
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

                if let Some(crate_id) = next_path.crate_id {
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

pub fn render_diff_markdown(
    writer: &mut dyn std::io::Write,
    before: &crate_api::Api,
    after: &crate_api::Api,
    diffs: &[crate_api::diff::Diff],
) -> Result<(), eyre::Report> {
    let mut diffs = diffs.to_vec();
    diffs.sort_by_key(|diff| (diff.severity, diff.id.category, diff.id.name));

    let mut last_severity = crate_api::diff::Severity::Allow;
    let mut last_category = None;
    for diff in diffs {
        if diff.severity != last_severity {
            match diff.severity {
                crate_api::diff::Severity::Allow => unreachable!(),
                crate_api::diff::Severity::Report => {
                    let _ = writeln!(writer, "## Changes");
                    let _ = writeln!(writer);
                }
                crate_api::diff::Severity::Warn => {
                    let _ = writeln!(writer, "## Breaking Changes");
                    let _ = writeln!(writer);
                }
            }
            last_severity = diff.severity;
        }
        if Some(diff.id.category) != last_category {
            match diff.id.category {
                crate_api::diff::Category::Unknown => {}
                crate_api::diff::Category::Added => {
                    let _ = writeln!(writer, "**Added**");
                }
                crate_api::diff::Category::Removed => {
                    let _ = writeln!(writer, "**Removed**");
                }
                crate_api::diff::Category::Changed => {
                    let _ = writeln!(writer, "**Changed**");
                }
            }
            last_category = Some(diff.id.category);
        }

        match diff.id {
            crate_api::diff::DEPENDENCY_REQUIREMENT => {
                let before_crate = before
                    .crates
                    .get(diff.before.unwrap().crate_id.unwrap())
                    .unwrap();
                let after_crate = after
                    .crates
                    .get(diff.after.unwrap().crate_id.unwrap())
                    .unwrap();
                let _ = writeln!(
                    writer,
                    "- `{}` (public dependency): changed version requirement from {} to {}",
                    after_crate.name,
                    before_crate.version.as_ref().unwrap(),
                    after_crate.version.as_ref().unwrap()
                );
            }
            _ => {
                let name = diff
                    .after
                    .map(|loc| location_name(after, loc))
                    .or_else(|| diff.before.map(|loc| location_name(before, loc)))
                    .expect("at least before or after exists");
                let _ = writeln!(writer, "- `{}`: {}", name, diff.id.explanation);
            }
        }
    }

    Ok(())
}

fn location_name(api: &crate_api::Api, location: crate_api::diff::Location) -> &str {
    if let Some(path_id) = location.path_id {
        api.paths.get(path_id).unwrap().path.as_str()
    } else if let Some(item_id) = location.item_id {
        api.items.get(item_id).unwrap().name.as_deref().unwrap()
    } else if let Some(crate_id) = location.crate_id {
        api.crates.get(crate_id).unwrap().name.as_str()
    } else {
        unimplemented!("{:?} had no location", location)
    }
}
