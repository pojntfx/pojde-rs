use std::str::from_utf8;
use std::str::FromStr;

use clap::{crate_authors, crate_description, crate_version, AppSettings, Clap};
use futures::future::try_join_all;
use futures::StreamExt;
use pojdectl_rs::instances::Instances;
use pojdectl_rs::update::update;
use shiplift::Docker;
use spinners::{Spinner, Spinners};
use tabled::Style;
use tokio::task::spawn_blocking;

use tabled::{Table, Tabled};

#[derive(Clap)]
#[clap(
    version = crate_version!(),
    author = crate_authors!(),
    about = crate_description!(),
    setting = AppSettings::ColoredHelp,
)]
struct Opts {
    #[clap(subcommand)]
    subcmd: Topics,

    #[clap(
        short,
        long,
        about = "Remote host to execute on, in format user@host:port",
        global = true
    )]
    node: Option<String>,
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
    #[clap(about = "Name(s) of the instance(s) to start", required = true)]
    names: Vec<String>,
}

#[derive(Clap)]
#[clap(
    about = "Stop instance(s)",
    setting = AppSettings::ColoredHelp,
)]
struct Stop {
    #[clap(about = "Name(s) of the instance(s) to stop", required = true)]
    names: Vec<String>,
}

#[derive(Clap)]
#[clap(
    about = "Restart instance(s)",
    setting = AppSettings::ColoredHelp,
)]
struct Restart {
    #[clap(about = "Name(s) of the instance(s) to restart", required = true)]
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

#[derive(Tabled)]
struct Instance {
    #[header("NAME")]
    name: String,
    #[header("STATUS")]
    status: String,
    #[header("PORTS")]
    ports: String,
}

#[tokio::main]
pub async fn main() {
    let opts = Opts::parse();

    match opts.subcmd {
        Topics::Modify(t) => {
            let instances = Instances {
                docker: Docker::new(),
            };

            match t.subcmd {
                ModificationCommands::Apply(_) => todo!(),
                ModificationCommands::Remove(_) => todo!(),
                ModificationCommands::List(_) => match instances.get_instances().await {
                    Ok(containers) => print!(
                        "{}",
                        Table::new(containers.iter().map(|c| {
                            if let (Some(start_port), Some(end_port)) = (c.start_port, c.end_port) {
                                return Instance {
                                    name: c.name.to_owned(),
                                    status: c.status.to_owned(),
                                    ports: start_port.to_string() + "-" + &end_port.to_string(),
                                };
                            }

                            Instance {
                                name: c.name.to_owned(),
                                status: c.status.to_owned(),
                                ports: String::new(),
                            }
                        }))
                        .with(Style::pseudo())
                        .to_string()
                    ),
                    Err(e) => eprintln!("Could not list instances: {}", e),
                },
            }
        }
        Topics::Cycle(t) => {
            let instances = Instances {
                docker: Docker::new(),
            };

            match t.subcmd {
                LifecycleCommands::Start(c) => {
                    let sp =
                        Spinner::new(Spinners::Dots, format!("Starting {:?} ...", c.names).into());

                    let res = try_join_all(
                        c.names
                            .iter()
                            .map(|name| instances.start(name))
                            .collect::<Vec<_>>(),
                    )
                    .await;

                    sp.stop();
                    print!("{}{}", ansi_escapes::CursorLeft, ansi_escapes::EraseLine);

                    match res {
                        Ok(_) => println!("Started {:?}.", c.names),
                        Err(e) => eprintln!("Could not start {:?}: {}", c.names, e),
                    }
                }
                LifecycleCommands::Stop(c) => {
                    let sp =
                        Spinner::new(Spinners::Dots, format!("Stopping {:?} ...", c.names).into());

                    let res = try_join_all(
                        c.names
                            .iter()
                            .map(|name| instances.stop(name))
                            .collect::<Vec<_>>(),
                    )
                    .await;

                    sp.stop();
                    print!("{}{}", ansi_escapes::CursorLeft, ansi_escapes::EraseLine);

                    match res {
                        Ok(_) => println!("Stopped {:?}.", c.names),
                        Err(e) => eprintln!("Could not stop {:?}: {}", c.names, e),
                    }
                }
                LifecycleCommands::Restart(c) => {
                    let sp = Spinner::new(
                        Spinners::Dots,
                        format!("Restarting {:?} ...", c.names).into(),
                    );

                    let res = try_join_all(
                        c.names
                            .iter()
                            .map(|name| instances.restart(name))
                            .collect::<Vec<_>>(),
                    )
                    .await;

                    sp.stop();
                    print!("{}{}", ansi_escapes::CursorLeft, ansi_escapes::EraseLine);

                    match res {
                        Ok(_) => println!("Restarted {:?}.", c.names),
                        Err(e) => eprintln!("Could not restart {:?}: {}", c.names, e),
                    }
                }
            }
        }
        Topics::Util(t) => {
            let instances = Instances {
                docker: Docker::new(),
            };

            match t.subcmd {
                UtilityCommands::Logs(c) => {
                    let mut logs = instances.get_logs(&c.name).await;

                    while let Some(log) = logs.next().await {
                        match log {
                            Ok(chunk) => match chunk {
                                shiplift::tty::TtyChunk::StdOut(b) => {
                                    print!("{}", from_utf8(&b).unwrap())
                                }
                                shiplift::tty::TtyChunk::StdErr(b) => {
                                    print!("{}", from_utf8(&b).unwrap())
                                }
                                shiplift::tty::TtyChunk::StdIn(_) => unreachable!(),
                            },
                            Err(e) => eprintln!("Could not get logs: {}", e),
                        }
                    }
                }
                UtilityCommands::Enter(c) => {
                    let mut console = instances.enter(&c.name).await;

                    while let Some(log) = console.next().await {
                        match log {
                            Ok(chunk) => match chunk {
                                shiplift::tty::TtyChunk::StdOut(b) => {
                                    print!("{}", from_utf8(&b).unwrap())
                                }
                                shiplift::tty::TtyChunk::StdErr(b) => {
                                    print!("{}", from_utf8(&b).unwrap())
                                }
                                shiplift::tty::TtyChunk::StdIn(_) => unreachable!(),
                            },
                            Err(e) => eprintln!("Could not enter instance: {}", e),
                        }
                    }
                }
                UtilityCommands::Forward(_) => todo!(),
            }
        }
        Topics::Misc(t) => match t.subcmd {
            MiscellaneousCommands::UpgradePojdectl(_) => {
                let res = spawn_blocking(|| update()).await;

                match res {
                    Ok(s) => println!("Upgrade status: `{}`", s.unwrap().version()),
                    Err(e) => eprintln!("Could not update: {}", e),
                }
            }
            MiscellaneousCommands::GetCACert(_) => todo!(),
            MiscellaneousCommands::ResetCA(_) => todo!(),
        },
    }
}
