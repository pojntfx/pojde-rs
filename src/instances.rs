use shiplift::Docker;

static POJDE_PREFIX: &str = "pojde-";

pub struct Instances {
    pub docker: Docker,
}

impl Instances {
    pub async fn restart(self: &Self, name: &str) -> Result<(), shiplift::Error> {
        self.docker
            .containers()
            .get(POJDE_PREFIX.to_owned() + name)
            .restart(None)
            .await
    }
}
