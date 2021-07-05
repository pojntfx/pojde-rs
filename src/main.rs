use bollard::{
    container::{ListContainersOptions, StartContainerOptions},
    Docker,
};
use clap::{crate_authors, crate_description, crate_version, Clap};
use maplit::hashmap;

const POJDE_DOCKER_PREFIX: &str = "pojde-";

#[derive(Clap)]
#[clap(
    version = crate_version!(),
    author = crate_authors!(),
    about = crate_description!(),
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
}

#[derive(Clap)]
#[clap(
    version = crate_version!(),
    author = crate_authors!(),
    about = "List all instances.",
)]
struct List {}

#[derive(Clap)]
#[clap(
    version = crate_version!(),
    author = crate_authors!(),
    about = "Start instance(s).",
)]
struct Start {
    names: Vec<String>,
}

#[derive(Clap)]
#[clap(
    version = crate_version!(),
    author = crate_authors!(),
    about = "Stop instance(s).",
)]
struct Stop {
    names: Vec<String>,
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

            for container in containers {
                let raw_name = &container.names.unwrap()[0];

                println!(
                    "{}",
                    raw_name
                        .trim_start_matches("/")
                        .trim_start_matches(POJDE_DOCKER_PREFIX)
                )
            }
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
                            panic!("Unexpected error during instance start: {:?}", other_error)
                        }
                    },
                }
            }
        }
    }
}
