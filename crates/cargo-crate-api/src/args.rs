use crate::report::Source;

#[derive(clap::Parser)]
#[clap(name = "cargo")]
#[clap(bin_name = "cargo")]
#[clap(
        global_setting = clap::AppSettings::DeriveDisplayOrder,
        global_setting = clap::AppSettings::DontCollapseArgsInUsage,
)]
#[clap(color =concolor_clap::color_choice())]
pub enum Command {
    CrateApi(Api),
}

#[derive(clap::Args)]
#[clap(about)]
#[clap(group = clap::ArgGroup::new("mode").multiple(false))]
#[clap(group = clap::ArgGroup::new("base").multiple(false).requires("diff"))]
pub struct Api {
    #[clap(long, group = "mode")]
    pub dump_raw: bool,

    #[clap(long, group = "mode")]
    pub api: bool,

    #[clap(short, long, group = "mode")]
    pub diff: bool,

    #[clap(long, value_name = "REF", group = "base")]
    pub git: Option<String>,

    #[clap(long, value_name = "TOML", group = "base")]
    pub path: Option<std::path::PathBuf>,

    #[clap(long, value_name = "PKG", group = "base")]
    pub registry: Option<String>,

    #[clap(
        short,
        long,
        arg_enum,
        default_value_t = Format::Pretty
    )]
    pub format: Format,

    #[clap(flatten)]
    pub manifest: clap_cargo::Manifest,

    #[clap(flatten)]
    pub workspace: clap_cargo::Workspace,

    #[clap(flatten)]
    pub(crate) color: concolor_clap::Color,

    #[clap(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,
}

impl Api {
    pub fn mode(&self) -> Mode {
        if self.dump_raw {
            Mode::DumpRaw
        } else if self.api {
            Mode::Api
        } else if self.diff {
            Mode::Diff
        } else {
            Mode::Api
        }
    }

    pub fn base(&self) -> Option<Source> {
        #[allow(clippy::manual_map)]
        if let Some(git) = self.git.as_ref() {
            Some(Source::Git(git.clone()))
        } else if let Some(path) = self.path.as_ref() {
            Some(Source::Path(path.clone()))
        } else if let Some(registry) = self.registry.as_ref() {
            Some(Source::Registry(registry.clone()))
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Mode {
    DumpRaw,
    Api,
    Diff,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, clap::ArgEnum)]
pub enum Format {
    Silent,
    Pretty,
    #[clap(alias = "markdown")]
    Md,
    Json,
}

impl Default for Format {
    fn default() -> Self {
        Format::Pretty
    }
}

#[test]
fn verify_app() {
    use clap::IntoApp;
    Command::into_app().debug_assert()
}
