use std::io::Write;

use proc_exit::WithCodeResultExt;
use structopt::StructOpt;

mod args;
mod log;

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

    let metadata = args
        .manifest
        .metadata()
        .exec()
        .with_code(proc_exit::Code::CONFIG_ERR)?;
    let (selected, _) = args.workspace.partition_packages(&metadata);
    for selected in selected {
        dbg!(selected);
    }

    Ok(())
}
