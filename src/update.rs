use std::{env::consts, io::Error, io::ErrorKind};

use clap::{crate_name, crate_version};

static REPO_OWNER: &str = "pojntfx";

#[cfg(not(target_arch = "riscv64"))]
pub fn update() -> Result<String, Error> {
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

    let res = self_update::backends::github::Update::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(crate_name!())
        .bin_name(&(crate_name!().to_owned() + bin_suffix))
        .show_download_progress(true)
        .current_version(crate_version!())
        .build()
        .unwrap()
        .update();

    match res {
        Ok(s) => Ok(s.version().to_string()),
        Err(e) => Err(Error::new(ErrorKind::Other, e)),
    }
}

#[cfg(target_arch = "riscv64")]
pub fn update() -> Result<String, Error> {
    Err(Error::new(
        ErrorKind::Other,
        "Self-updates are not supported on riscv64 yet.",
    ))
}
