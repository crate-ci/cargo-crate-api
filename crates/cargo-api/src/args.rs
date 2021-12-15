#[derive(structopt::StructOpt)]
#[structopt(
        setting = structopt::clap::AppSettings::UnifiedHelpMessage,
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::DontCollapseArgsInUsage,
        setting = structopt::clap::AppSettings::ColoredHelp,
        setting = concolor_clap::color_choice(),
        about,
    )]
pub struct Args {
    #[structopt(flatten)]
    pub manifest: clap_cargo::Manifest,

    #[structopt(flatten)]
    pub workspace: clap_cargo::Workspace,

    #[structopt(flatten)]
    pub(crate) color: concolor_clap::Color,

    #[structopt(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,
}
