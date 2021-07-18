use futures::Stream;
use shiplift::{tty, ContainerFilter, ContainerListOptions, Docker, Error, LogsOptions};

static POJDE_PREFIX: &str = "pojde-";

pub struct Instances {
    pub docker: Docker,
}

pub struct Instance {
    pub name: String,
    pub start_port: Option<u64>,
    pub end_port: Option<u64>,
    pub status: String,
}

impl Instances {
    fn get_container(self: &Self, name: &str) -> shiplift::Container<'_> {
        self.docker.containers().get(POJDE_PREFIX.to_owned() + name)
    }

    pub async fn start(self: &Self, name: &str) -> Result<(), shiplift::Error> {
        self.get_container(name).start().await
    }

    pub async fn stop(self: &Self, name: &str) -> Result<(), shiplift::Error> {
        self.get_container(name).stop(None).await
    }

    pub async fn restart(self: &Self, name: &str) -> Result<(), shiplift::Error> {
        self.get_container(name).restart(None).await
    }

    pub async fn get_logs(
        self: &Self,
        name: &str,
    ) -> impl Stream<Item = Result<tty::TtyChunk, Error>> + '_ {
        self.get_container(name)
            .logs(&LogsOptions::builder().stdout(true).stderr(true).build())
    }

    pub async fn get_instances(self: &Self) -> Result<Vec<Instance>, shiplift::Error> {
        let instances = self
            .docker
            .containers()
            .list(
                &ContainerListOptions::builder()
                    .all()
                    .filter(vec![ContainerFilter::Name("/".to_owned() + POJDE_PREFIX)])
                    .build(),
            )
            .await;

        match instances {
            Ok(i) => Ok(i
                .iter()
                .map(|c| {
                    let mut ports = c
                        .ports
                        .iter()
                        .map(|p| p.public_port.unwrap())
                        .collect::<Vec<_>>();
                    ports.sort();

                    Instance {
                        name: c.names[0]
                            .strip_prefix(&("/".to_owned() + POJDE_PREFIX))
                            .unwrap()
                            .to_string(),
                        start_port: ports.first().copied(),
                        end_port: ports.last().copied(),
                        status: c.state.to_owned(),
                    }
                })
                .collect::<Vec<_>>()),
            Err(e) => Err(e),
        }
    }
}
