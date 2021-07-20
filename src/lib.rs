pub mod instances;
pub mod update;

#[cfg(not(any(target_arch = "riscv64")))]
pub mod widgets;
