use std::env::consts;

use clap::{crate_name, crate_version};

static REPO_OWNER: &str = "pojntfx";

pub fn update() -> Result<self_update::Status, self_update::errors::Error> {
    let bin_suffix = match (consts::OS, consts::ARCH) {
        ("linux", "x86_64") => "linux-x86_64",
        ("linux", "aarch64") => "linux-aarch64",
        ("linux", "arm") => "linux-armv7l",
        ("linux", "riscv64") => "linux-riscv64",
        ("windows", "x86_64") => "exe.windows-x86_64",
        ("macos", "x86_64") => "macos-x86_64",
        ("macos", "aarch64") => "macos-aarch64",
        (os, arch) => panic!(
            "Can't upgrade, unsupported OS `{}` or architecture `{}`",
            os, arch
        ),
    };

    self_update::backends::github::Update::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(crate_name!())
        .bin_name(&(crate_name!().to_owned() + bin_suffix))
        .show_download_progress(true)
        .current_version(crate_version!())
        .build()?
        .update()
}
