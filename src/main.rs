use std::str::FromStr;

use clap::{crate_authors, crate_description, crate_version, AppSettings, Clap};

#[derive(Clap)]
#[clap(
    version = crate_version!(),
    author = crate_authors!(),
    about = crate_description!(),
    setting = AppSettings::ColoredHelp,
)]
struct Opts {
    #[clap(subcommand)]
    topics: Topics,
}

#[derive(Clap)]
enum Topics {
    // Modification commands
    Modify(Modify),

    // Lifecycle commands
    Cycle(Cycle),

    // Utility commands
    Util(Util),

    // Miscellaneous commands
    Misc(Misc),
}

// Modification commands
#[derive(Clap)]
#[clap(
    about = "Modification commands",
    setting = AppSettings::ColoredHelp,
    aliases = &["mo", "m", "mod", "modify"],
)]
struct Modify {
    #[clap(subcommand)]
    subcmd: ModificationCommands,
}

#[derive(Clap)]
enum ModificationCommands {
    Apply(Apply),
    Remove(Remove),
    List(List),
}

#[derive(Clap)]
#[clap(
    about = "Create or upgrade an instance",
    setting = AppSettings::ColoredHelp,
)]
struct Apply {
    #[clap(about = "Name of the instance to apply")]
    name: String,
    #[clap(about = "Starting port for the instance")]
    start_port: i32,
    #[clap(short, long, about = "Skip confirmation prompts")]
    force: bool,
    #[clap(short, long, about = "Pull latest image")]
    upgrade: bool,
    #[clap(short, long, about = "Re-create the container")]
    recreate: bool,
    #[clap(short, long, about = "Block Docker daemon access")]
    isolate: bool,
    #[clap(short, long, about = "Run in privileged mode")]
    privileged: bool,
}

#[derive(Clap)]
#[clap(
    about = "Remove instances(s)",
    setting = AppSettings::ColoredHelp,
)]
struct Remove {
    #[clap(about = "Name(s) of the instance(s) to remove")]
    names: Vec<String>,
    #[clap(short, long, about = "Skip confirmation prompts")]
    force: bool,
    #[clap(short, long, about = "Remove customizations")]
    customizations: bool,
    #[clap(short, long, about = "Remove preferences")]
    preferences: bool,
    #[clap(short, long, about = "Remove CA")]
    security: bool,
    #[clap(short, long, about = "Remove user data")]
    user_data: bool,
    #[clap(short, long, about = "Remove transfer data")]
    transfer: bool,
    #[clap(short, long, about = "Remove .deb cache")]
    deb_cache: bool,
    #[clap(short, long, about = "Remove everything")]
    all: bool,
}

#[derive(Clap)]
#[clap(
    about = "List all instances",
    setting = AppSettings::ColoredHelp,
)]
struct List {}

// Lifecycle commands
#[derive(Clap)]
#[clap(
    about = "Lifecycle commands",
    setting = AppSettings::ColoredHelp,
    aliases = &["cy", "c"],
)]
struct Cycle {
    #[clap(subcommand)]
    subcmd: LifecycleCommands,
}

#[derive(Clap)]
enum LifecycleCommands {
    Start(Start),
    Stop(Stop),
    Restart(Restart),
}

#[derive(Clap)]
#[clap(
    about = "Start instance(s)",
    setting = AppSettings::ColoredHelp,
)]
struct Start {
    #[clap(about = "Name(s) of the instance(s) to start")]
    names: Vec<String>,
}

#[derive(Clap)]
#[clap(
    about = "Stop instance(s)",
    setting = AppSettings::ColoredHelp,
)]
struct Stop {
    #[clap(about = "Name(s) of the instance(s) to stop")]
    names: Vec<String>,
}

#[derive(Clap)]
#[clap(
    about = "Restart instance(s)",
    setting = AppSettings::ColoredHelp,
)]
struct Restart {
    #[clap(about = "Name(s) of the instance(s) to restart")]
    names: Vec<String>,
}

// Utility commands
#[derive(Clap)]
#[clap(
    about = "Utility commands",
    setting = AppSettings::ColoredHelp,
    aliases = &["ut", "u"],
)]
struct Util {
    #[clap(subcommand)]
    subcmd: UtilityCommands,
}

#[derive(Clap)]
enum UtilityCommands {
    Logs(Logs),
    Enter(Enter),
    Forward(Forward),
}

#[derive(Clap)]
#[clap(
    about = "Get the logs of an instance",
    setting = AppSettings::ColoredHelp,
)]
struct Logs {
    #[clap(about = "Name of the instance to get logs for")]
    name: String,
}

#[derive(Clap)]
#[clap(
    about = "Get a shell in an instance",
    setting = AppSettings::ColoredHelp,
)]
struct Enter {
    #[clap(about = "Name of the instance to enter")]
    name: String,
}

#[derive(Clap)]
#[clap(
    about = "Forward port(s) to or from an instance",
    setting = AppSettings::ColoredHelp,
)]
struct Forward {
    #[clap(about = "Name of the instance to forward from or to")]
    name: String,
    #[clap(about = "Local address:remote address to forward, i.e. localhost:5000:localhost:5000")]
    address: Vec<String>,
    #[clap(short, long, about = "Peer to forward to", possible_values = &["local","remote"], default_value = "local")]
    direction: Direction,
}

enum Direction {
    Local,
    Remote,
}

impl FromStr for Direction {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "local" => Ok(Self::Local),
            "remote" => Ok(Self::Remote),
            _ => Err("no match"),
        }
    }
}

// Miscellaneous commands
#[derive(Clap)]
#[clap(
    about = "Miscellaneous commands",
    setting = AppSettings::ColoredHelp,
    aliases = &["mi", "i"],
)]
struct Misc {
    #[clap(subcommand)]
    subcmd: MiscellaneousCommands,
}

#[derive(Clap)]
enum MiscellaneousCommands {
    UpgradePojdectl(UpgradePojdectl),
    GetCACert(GetCACert),
    ResetCA(ResetCA),
}

#[derive(Clap)]
#[clap(
    about = "Upgrade this tool",
    setting = AppSettings::ColoredHelp,
)]
struct UpgradePojdectl {}

#[derive(Clap)]
#[clap(
    about = "Get the CA cert",
    setting = AppSettings::ColoredHelp,
)]
struct GetCACert {
    #[clap(short, long, about = "Print the certificate instead of downloading it")]
    print: bool,
}

#[derive(Clap)]
#[clap(
    about = "Reset the CA",
    setting = AppSettings::ColoredHelp,
)]
struct ResetCA {
    #[clap(short, long, about = "Skip confirmation prompts")]
    force: bool,
}

#[tokio::main]
async fn main() {
    Opts::parse();
}
