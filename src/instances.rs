use shiplift::Docker;

// TODO: Make private once listing logic is contained here
pub static POJDE_PREFIX: &str = "pojde-";

pub struct Instances {
    pub docker: Docker,
}

impl Instances {
    // TODO: Make private once logs are integrated here
    pub fn get_container(self: &Self, name: &str) -> shiplift::Container<'_> {
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
}
