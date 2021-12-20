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
    pub api: bool,

    #[structopt(short, long, group = "mode")]
    pub diff: bool,

    #[structopt(short, long, possible_values(&Format::variants()), default_value = "pretty")]
    pub format: Format,

    #[structopt(flatten)]
    pub manifest: clap_cargo::Manifest,

    #[structopt(flatten)]
    pub workspace: clap_cargo::Workspace,

    #[structopt(flatten)]
    pub(crate) color: concolor_clap::Color,

    #[structopt(flatten)]
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
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Mode {
    DumpRaw,
    Api,
    Diff,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Format {
    Silent,
    Pretty,
    Md,
    Json,
}

impl Format {
    pub fn variants() -> [&'static str; 4] {
        ["silent", "pretty", "md", "json"]
    }
}

impl std::str::FromStr for Format {
    type Err = String;
    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        match s {
            "silent" => Ok(Format::Silent),
            "pretty" => Ok(Format::Pretty),
            "md" | "markdown" => Ok(Format::Md),
            "json" => Ok(Format::Json),
            _ => Err(format!("valid values: {}", Self::variants().join(", "))),
        }
    }
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Format::Silent => "silent".fmt(f),
            Format::Pretty => "pretty".fmt(f),
            Format::Md => "md".fmt(f),
            Format::Json => "json".fmt(f),
        }
    }
}

impl Default for Format {
    fn default() -> Self {
        Format::Pretty
    }
}
