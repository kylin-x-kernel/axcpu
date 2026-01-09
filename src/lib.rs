#![cfg_attr(not(test), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![feature(cold_path)]
#![feature(if_let_guard)]
#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

#[macro_use]
extern crate log;

#[macro_use]
extern crate memory_addr;

#[macro_use]
pub mod trap;

#[cfg(feature = "uspace")]
mod uspace_common;

cfg_if::cfg_if! {
    if #[cfg(target_arch = "x86_64")] {
        mod x86_64;
        pub use self::x86_64::*;
    } else if #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))] {
        mod riscv;
        pub use self::riscv::*;
    } else if #[cfg(target_arch = "aarch64")]{
        mod aarch64;
        pub use self::aarch64::*;
    } else if #[cfg(any(target_arch = "loongarch64"))] {
        mod loongarch64;
        pub use self::loongarch64::*;
    }
}

/// Control the interrupt state of the current CPU.
#[crate_interface::def_interface]
pub trait IrqCtlIf {
    /// Disable all interrupts on the current CPU.
    fn disable_irqs();

    /// Enable all interrupts on the current CPU.
    fn enable_irqs();

    /// Check if interrupts are currently enabled on the current CPU.
    fn irqs_enabled() -> bool;
}

use crate_interface::call_interface;

/// Disable all interrupts on the current CPU.
pub fn disable_irqs() {
    call_interface!(IrqCtlIf::disable_irqs);
}

/// Enable all interrupts on the current CPU.
pub fn enable_irqs() {
    call_interface!(IrqCtlIf::enable_irqs);
}

/// Check if interrupts are currently enabled on the current CPU.
pub fn irqs_enabled() -> bool {
    call_interface!(IrqCtlIf::irqs_enabled)
}