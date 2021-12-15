#[derive(structopt::StructOpt)]
#[structopt(name = "cargo")]
#[structopt(bin_name = "cargo")]
#[structopt(
        global_setting = structopt::clap::AppSettings::UnifiedHelpMessage,
        global_setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        global_setting = structopt::clap::AppSettings::DontCollapseArgsInUsage,
        global_setting = structopt::clap::AppSettings::ColoredHelp,
        global_setting = concolor_clap::color_choice(),
)]
pub enum Command {
    Api(Api),
}

#[derive(structopt::StructOpt)]
#[structopt(about)]
#[structopt(group = structopt::clap::ArgGroup::with_name("mode").multiple(false))]
pub struct Api {
    #[structopt(long, group = "mode")]
    pub dump_raw: bool,

    #[structopt(long, group = "mode")]
    pub dump_api: bool,

    #[structopt(flatten)]
    pub manifest: clap_cargo::Manifest,

    #[structopt(flatten)]
    pub workspace: clap_cargo::Workspace,

    #[structopt(flatten)]
    pub(crate) color: concolor_clap::Color,

    #[structopt(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,
}
