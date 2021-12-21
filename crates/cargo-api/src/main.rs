use std::io::Write;

use proc_exit::WithCodeResultExt;
use structopt::StructOpt;

mod args;
mod log;
mod report;

fn main() {
    human_panic::setup_panic!();
    let result = run();
    proc_exit::exit(result);
}

fn run() -> proc_exit::ExitResult {
    // clap2's `get_matches` uses Failure rather than Unknown, so bypass it for `get_matches_safe`.
    let args::Command::Api(args) = match args::Command::from_args_safe() {
        Ok(args) => args,
        Err(e) if e.use_stderr() => {
            return Err(proc_exit::Code::UNKNOWN.with_message(e));
        }
        Err(e) => {
            writeln!(std::io::stdout(), "{}", e)?;
            return proc_exit::Code::SUCCESS.ok();
        }
    };

    args.color.apply();
    let colored_stderr = concolor_control::get(concolor_control::Stream::Stderr).ansi_color();

    log::init_logging(args.verbose.clone(), colored_stderr);

    let mut success = true;

    let metadata = args
        .manifest
        .metadata()
        .exec()
        .with_code(proc_exit::Code::CONFIG_ERR)?;

    let mode = args.mode();
    let base = match mode {
        args::Mode::DumpRaw => None,
        args::Mode::Api => None,
        args::Mode::Diff => {
            let base = args
                .base()
                .map(Ok)
                .unwrap_or_else(|| find_default_base(metadata.workspace_root.as_std_path()))
                .with_code(proc_exit::Code::FAILURE)?;
            Some(base)
        }
    };

    let (selected, _) = args.workspace.partition_packages(&metadata);
    for selected in selected {
        let res = match mode {
            args::Mode::DumpRaw => dump_raw(selected, args.format),
            args::Mode::Api => api(selected, args.format),
            args::Mode::Diff => diff(&metadata, selected, base.clone().unwrap(), args.format),
        };
        match res {
            Ok(()) => {}
            Err(err) => {
                ::log::error!("{}", err);
                success = false;
                continue;
            }
        };
    }

    if success {
        proc_exit::Code::SUCCESS.ok()
    } else {
        proc_exit::Code::FAILURE.ok()
    }
}

fn dump_raw(pkg: &cargo_metadata::Package, format: args::Format) -> Result<(), eyre::Report> {
    let raw =
        crate_api::RustDocBuilder::new().dump_raw(pkg.manifest_path.as_path().as_std_path())?;
    let raw: rustdoc_json_types_fork::Crate = serde_json::from_str(&raw)?;

    let manifest = crate_api::manifest::Manifest::from(pkg);

    let raw = report::Raw {
        manifest_path: pkg.manifest_path.clone().into_std_path_buf(),
        rustdoc: Some(raw),
        manifest: Some(manifest),
    };

    match format {
        args::Format::Silent => {}
        args::Format::Pretty => {
            let _ = writeln!(std::io::stdout(), "{}", serde_json::to_string_pretty(&raw)?);
        }
        args::Format::Md => {
            let _ = writeln!(
                std::io::stdout(),
                "```json
{}
```",
                serde_json::to_string_pretty(&raw)?
            );
        }
        args::Format::Json => {
            let _ = writeln!(std::io::stdout(), "{}", serde_json::to_string(&raw)?);
        }
    }

    Ok(())
}

fn api(pkg: &cargo_metadata::Package, format: args::Format) -> Result<(), eyre::Report> {
    let mut api =
        crate_api::RustDocBuilder::new().into_api(pkg.manifest_path.as_path().as_std_path())?;

    let manifest = crate_api::manifest::Manifest::from(pkg);
    manifest.into_api(&mut api);

    match format {
        args::Format::Silent => {}
        args::Format::Pretty => {
            // HACK: Real version (using `termtree`) isn't implemented yet
            let _ = writeln!(std::io::stdout(), "{}", serde_json::to_string_pretty(&api)?);
        }
        args::Format::Md => {
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();
            report::render_api_markdown(&mut stdout, &api)?;
        }
        args::Format::Json => {
            let _ = writeln!(std::io::stdout(), "{}", serde_json::to_string(&api)?);
        }
    }

    Ok(())
}

fn diff(
    metadata: &cargo_metadata::Metadata,
    pkg: &cargo_metadata::Package,
    base: report::Source,
    format: args::Format,
) -> Result<(), eyre::Report> {
    let mut after =
        crate_api::RustDocBuilder::new().into_api(pkg.manifest_path.as_path().as_std_path())?;
    let manifest = crate_api::manifest::Manifest::from(pkg);
    manifest.into_api(&mut after);

    let base_path = resolve_source_path(metadata, pkg, &base)?;
    let mut before = crate_api::RustDocBuilder::new().into_api(&base_path)?;
    let old_pkg = resolve_package(&base_path)?;
    let manifest = crate_api::manifest::Manifest::from(&old_pkg);
    manifest.into_api(&mut before);

    let mut diffs = Vec::new();
    crate_api::diff::diff(&before, &after, &mut diffs);

    match format {
        args::Format::Silent => {}
        args::Format::Pretty => {
            // HACK: Real version (using `termtree`) isn't implemented yet
            let raw = report::Diff {
                manifest_path: pkg.manifest_path.clone().into_std_path_buf(),
                against: base,
                before,
                after,
                diffs,
            };
            let _ = writeln!(std::io::stdout(), "{}", serde_json::to_string_pretty(&raw)?);
        }
        args::Format::Md => {
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();
            report::render_diff_markdown(&mut stdout, &before, &after, &diffs)?;
        }
        args::Format::Json => {
            let raw = report::Diff {
                manifest_path: pkg.manifest_path.clone().into_std_path_buf(),
                against: base,
                before,
                after,
                diffs,
            };
            let _ = writeln!(std::io::stdout(), "{}", serde_json::to_string(&raw)?);
        }
    }

    Ok(())
}

fn find_default_base(path: &std::path::Path) -> Result<report::Source, eyre::Report> {
    let repo = git2::Repository::discover(path)?;

    let mut tags = std::collections::HashMap::new();
    repo.tag_foreach(|oid, name| {
        if let Ok(name) = std::str::from_utf8(name) {
            tags.insert(oid, name.to_owned());
        }
        true
    })?;

    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    for oid in revwalk {
        let oid = oid?;
        if let Some(tag) = tags.remove(&oid) {
            return Ok(report::Source::Git(tag));
        }
    }

    eyre::bail!("Could not find a tag for {} for base", path.display());
}

fn resolve_package(path: &std::path::Path) -> Result<cargo_metadata::Package, eyre::Report> {
    let metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path(path)
        .exec()?;
    let root_id = metadata
        .resolve
        .expect("present because called with deps")
        .root
        .ok_or_else(|| {
            eyre::eyre!(
                "Expected package manifest, got virtual workspace at {}",
                path.display()
            )
        })?;
    let pkg = metadata
        .packages
        .into_iter()
        .find(|p| p.id == root_id)
        .expect("resolved root_id to exist");
    Ok(pkg)
}

fn resolve_source_path(
    metadata: &cargo_metadata::Metadata,
    pkg: &cargo_metadata::Package,
    source: &report::Source,
) -> Result<std::path::PathBuf, eyre::Report> {
    match source {
        report::Source::Git(rev) => {
            let target = metadata
                .target_directory
                .join(format!("crate-api/{}-base", pkg.name))
                .into_std_path_buf();
            checkout_ref(pkg.manifest_path.as_std_path(), &target, rev)?;
            find_by_package_name(&pkg.name, &target)
        }
        report::Source::Path(path) => Ok(path.to_owned()),
        report::Source::Registry(_) => {
            todo!()
        }
    }
}

fn find_by_package_name(
    name: &str,
    target: &std::path::Path,
) -> Result<std::path::PathBuf, eyre::Report> {
    let metadata = cargo_metadata::MetadataCommand::new()
        .current_dir(target)
        .no_deps()
        .exec()?;
    metadata
        .packages
        .iter()
        .filter(|p| metadata.workspace_members.contains(&p.id))
        .filter(|p| p.name == name)
        .map(|p| p.manifest_path.as_std_path().to_owned())
        .next()
        .ok_or_else(|| eyre::eyre!("Could no find {} at {}", name, target.display()))
}

fn checkout_ref(
    source: &std::path::Path,
    target: &std::path::Path,
    rev: &str,
) -> Result<(), eyre::Report> {
    let repo = git2::Repository::discover(source)?;

    let rev = repo.revparse_single(rev)?;

    let mut co = git2::build::CheckoutBuilder::new();
    co.target_dir(target)
        .remove_untracked(true)
        .remove_ignored(true)
        .use_ours(true)
        .force();
    repo.checkout_tree(&rev, Some(&mut co))?;

    Ok(())
}
