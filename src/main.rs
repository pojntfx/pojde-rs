use openssh::{KnownHosts, Session};
use std::str::FromStr;

use bollard::{
    container::{InspectContainerOptions, ListContainersOptions, StartContainerOptions},
    Docker,
};
use clap::{crate_authors, crate_description, crate_version, AppSettings, Clap};
use maplit::hashmap;
use owo_colors::OwoColorize;
use tabled::{Column, Format, Modify, Style, Table, Tabled};

const POJDE_DOCKER_PREFIX: &str = "pojde-";
const SSH_PORT: &str = "8005/tcp";
const CONTAINER_RUNNING_STATE: &str = "running";
const CONTAINER_EXITED_STATE: &str = "exited";

#[derive(Clap)]
#[clap(
    version = crate_version!(),
    author = crate_authors!(),
    about = crate_description!(),
    setting = AppSettings::ColoredHelp,
)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    List(List),
    Start(Start),
    Stop(Stop),
    Restart(Restart),
    Forward(Forward),
}

#[derive(Clap)]
#[clap(
    about = "List all instances.",
    setting = AppSettings::ColoredHelp,
)]
struct List {}

#[derive(Clap)]
#[clap(
    about = "Start instance(s).",
    setting = AppSettings::ColoredHelp,
)]
struct Start {
    names: Vec<String>,
}

#[derive(Clap)]
#[clap(
    about = "Stop instance(s).",
    setting = AppSettings::ColoredHelp,
)]
struct Stop {
    names: Vec<String>,
}

#[derive(Clap)]
#[clap(
    about = "Restart instance(s).",
    setting = AppSettings::ColoredHelp,
)]
struct Restart {
    names: Vec<String>,
}

#[derive(Clap)]
#[clap(
    about = "Forward port(s) to or from an instance.",
    setting = AppSettings::ColoredHelp,
)]
struct Forward {
    #[clap(about = "Name of the instance to forward from or to.")]
    name: String,
    #[clap(about = "Local address:remote address to forward, i.e. localhost:5000:localhost:5000")]
    address: Vec<String>,
    #[clap(short, long, about = "Peer to forward to.", possible_values = &["local","remote"], default_value = "local")]
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

#[derive(Tabled)]
struct InstanceColumn {
    #[header("NAME")]
    name: String,
    #[header("STATUS")]
    status: String,
    #[header("PORTS")]
    ports: String,
}

#[tokio::main]
async fn main() {
    let opts = Opts::parse();

    let docker = Docker::connect_with_local_defaults().unwrap();

    match opts.subcmd {
        SubCommand::List(_) => {
            let containers = docker
                .list_containers(Some(ListContainersOptions {
                    all: true,
                    filters: hashmap! {
                        "name" => vec![POJDE_DOCKER_PREFIX],
                    },
                    ..Default::default()
                }))
                .await
                .unwrap();

            let containers_in_table = containers.iter().map(|c| {
                let mut ports: Vec<i64> = c
                    .ports
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|p| p.public_port.unwrap())
                    .collect();
                ports.sort_by(|a, b| a.cmp(b));

                let name = c.names.as_ref().unwrap()[0]
                    .trim_start_matches("/")
                    .trim_start_matches(POJDE_DOCKER_PREFIX)
                    .to_string();
                let status = c.state.as_ref().unwrap().to_string();

                let ports = match (ports.first(), ports.last()) {
                    (Some(first_port), Some(last_port)) => {
                        first_port.to_string() + "-" + &last_port.to_string()
                    }
                    _ => String::new(),
                };

                InstanceColumn {
                    name,
                    status,
                    ports,
                }
            });

            print!(
                "{}",
                Table::new(containers_in_table)
                    .with(Style::pseudo_clean())
                    .with(Modify::new(Column(..1)).with(Format(|s| s.yellow().to_string())))
                    .with(Modify::new(Column(1..2)).with(Format(|s| {
                        if s == CONTAINER_EXITED_STATE {
                            return s.red().to_string();
                        }

                        s.green().to_string()
                    })))
                    .with(Modify::new(Column(2..)).with(Format(|s| s.cyan().to_string())))
            );
        }
        SubCommand::Start(c) => {
            for name in c
                .names
                .iter()
                .map(|name| POJDE_DOCKER_PREFIX.to_owned() + name)
            {
                let res = docker
                    .start_container(&name, None::<StartContainerOptions<String>>)
                    .await;

                match res {
                    Ok(_) => println!("Instance started."),
                    Err(error) => match error {
                        bollard::errors::Error::DockerResponseNotModifiedError { message: _ } => {
                            println!("Instance already running.");
                        }
                        other_error => {
                            panic!("Unexpected error during instance start: {:?}", other_error)
                        }
                    },
                }
            }
        }
        SubCommand::Stop(c) => {
            for name in c
                .names
                .iter()
                .map(|name| POJDE_DOCKER_PREFIX.to_owned() + name)
            {
                let res = docker.stop_container(&name, None).await;

                match res {
                    Ok(_) => println!("Instance stopped."),
                    Err(error) => match error {
                        bollard::errors::Error::DockerResponseNotModifiedError { message: _ } => {
                            println!("Instance already stopped.");
                        }
                        other_error => {
                            panic!("Unexpected error during instance stop: {:?}", other_error)
                        }
                    },
                }
            }
        }
        SubCommand::Restart(c) => {
            for name in c
                .names
                .iter()
                .map(|name| POJDE_DOCKER_PREFIX.to_owned() + name)
            {
                let res = docker.restart_container(&name, None).await;

                match res {
                    Ok(_) => println!("Instance restarted."),
                    Err(error) => panic!("Unexpected error during instance restart: {:?}", error),
                }
            }
        }
        SubCommand::Forward(c) => {
            let container = docker
                .inspect_container(&(POJDE_DOCKER_PREFIX.to_owned() + &c.name), None)
                .await;

            let ports = container.unwrap().network_settings.unwrap().ports.unwrap();

            let shared_port = ports.get("8005/tcp").unwrap().as_ref().unwrap()[0]
                .host_port
                .as_ref()
                .unwrap();

            println!("{}", shared_port);
        }
    }
}
