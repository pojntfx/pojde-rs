#[cfg(not(target_arch = "riscv64"))]
use pojdectl_rs::widgets;

#[cfg(not(target_arch = "riscv64"))]
#[tokio::main]
pub async fn main() {
    let app = widgets::Window::default();
    let native_options = eframe::NativeOptions::default();

    eframe::run_native(Box::new(app), native_options);
}

#[cfg(target_arch = "riscv64")]
#[tokio::main]
pub async fn main() {
    eprintln!("pojdegui is not supported on RISC-V yet.")
}
