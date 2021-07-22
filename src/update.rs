use std::env::consts;

use clap::{crate_name, crate_version};

static REPO_OWNER: &str = "pojntfx";

pub fn update() -> Result<self_update::Status, self_update::errors::Error> {
    let bin_suffix = match (consts::OS, consts::ARCH) {
        ("windows", arch) => format!("exe.windows-{}.exe", arch), // Windows is special
        (os, arch) => format!("{}-{}", os, arch),
    };

    self_update::backends::github::Update::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(crate_name!())
        .bin_name(&(crate_name!().to_owned() + &bin_suffix))
        .show_download_progress(true)
        .current_version(crate_version!())
        .build()?
        .update()
}
