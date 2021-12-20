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

    let mode = args.mode();

    args.color.apply();
    let colored_stderr = concolor_control::get(concolor_control::Stream::Stderr).ansi_color();

    log::init_logging(args.verbose.clone(), colored_stderr);

    let mut success = true;

    let metadata = args
        .manifest
        .metadata()
        .exec()
        .with_code(proc_exit::Code::CONFIG_ERR)?;
    let (selected, _) = args.workspace.partition_packages(&metadata);
    for selected in selected {
        let res = match mode {
            args::Mode::DumpRaw => dump_raw(selected, args.format),
            args::Mode::Api => api(selected, args.format),
            args::Mode::Diff => {
                let source = args.source().unwrap();
                diff(selected, source, args.format)
            }
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
    pkg: &cargo_metadata::Package,
    source: report::Source,
    format: args::Format,
) -> Result<(), eyre::Report> {
    let mut api =
        crate_api::RustDocBuilder::new().into_api(pkg.manifest_path.as_path().as_std_path())?;

    let manifest = crate_api::manifest::Manifest::from(pkg);
    manifest.into_api(&mut api);

    match format {
        args::Format::Silent => {}
        args::Format::Pretty => {
            // HACK: Real version (using `termtree`) isn't implemented yet
            let raw = report::Diff {
                manifest_path: pkg.manifest_path.clone().into_std_path_buf(),
                against: source,
                after: api,
            };
            let _ = writeln!(std::io::stdout(), "{}", serde_json::to_string_pretty(&raw)?);
        }
        args::Format::Md => {
            // HACK: Real version isn't implemented yet
            let raw = report::Diff {
                manifest_path: pkg.manifest_path.clone().into_std_path_buf(),
                against: source,
                after: api,
            };
            let _ = writeln!(std::io::stdout(), "{}", serde_json::to_string_pretty(&raw)?);
        }
        args::Format::Json => {
            let raw = report::Diff {
                manifest_path: pkg.manifest_path.clone().into_std_path_buf(),
                against: source,
                after: api,
            };
            let _ = writeln!(std::io::stdout(), "{}", serde_json::to_string(&raw)?);
        }
    }

    Ok(())
}
