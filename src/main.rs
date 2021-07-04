use bollard::{container::ListContainersOptions, Docker};
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
}

#[derive(Clap)]
#[clap(
    version = crate_version!(),
    author = crate_authors!(),
    about = "List all instances.",
)]
struct List {}

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
    }
}
