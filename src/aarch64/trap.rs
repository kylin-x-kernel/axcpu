use aarch64_cpu::registers::{ESR_EL1, FAR_EL1};
use tock_registers::interfaces::Readable;

use super::TrapFrame;
use crate::trap::PageFaultFlags;

core::arch::global_asm!(
    include_str!("trap.S"),
    trapframe_size = const core::mem::size_of::<TrapFrame>(),
    kind_irq = const TrapKind::Irq as u8,
    kind_sync = const TrapKind::Synchronous as u8,
);

#[repr(u8)]
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) enum TrapKind {
    Synchronous = 0,
    Irq = 1,
    Fiq = 2,
    SError = 3,
}

#[repr(u8)]
#[derive(Debug)]
#[allow(dead_code)]
enum TrapSource {
    CurrentSpEl0 = 0,
    CurrentSpElx = 1,
    LowerAArch64 = 2,
    LowerAArch32 = 3,
}

#[unsafe(no_mangle)]
fn invalid_exception(tf: &TrapFrame, kind: TrapKind, source: TrapSource) {
    panic!(
        "Invalid exception {:?} from {:?}:\n{:#x?}",
        kind, source, tf
    );
}

#[unsafe(no_mangle)]
fn handle_irq_exception(_tf: &mut TrapFrame) {
    handle_trap!(IRQ, 0);
}

#[inline(always)]
pub(crate) fn is_valid_page_fault(iss: u64) -> bool {
    // Only handle Translation fault and Permission fault
    matches!(iss & 0b111100, 0b0100 | 0b1100) // IFSC or DFSC bits
}

fn handle_instruction_abort(tf: &mut TrapFrame, iss: u64) {
    let access_flags = PageFaultFlags::EXECUTE;
    let vaddr = va!(FAR_EL1.get() as usize);
    info!("Instruction Abort @ {:#x}, fault_vaddr={:#x}, ESR={:#x} ({:?})", tf.elr, vaddr, ESR_EL1.get(), access_flags);
    // Only handle Translation fault and Permission fault
    if !is_valid_page_fault(iss) {
        panic!(
            "Invalid Instruction Abort ISS {:#x} @ {:#x}, fault_vaddr={:#x}, ESR={:#x} ({:?}):\n{:#x?}\n{}",
            iss,
            tf.elr,
            vaddr,
            ESR_EL1.get(),
            access_flags,
            tf,
            tf.backtrace()
        );
    }

    if core::hint::likely(handle_trap!(PAGE_FAULT, vaddr, access_flags)) {
        return;
    }

    if !tf.fixup_exception() {
        panic!(
            "Unhandled EL1 Instruction Abort @ {:#x}, fault_vaddr={:#x}, ESR={:#x} ({:?}):\n{:#x?}\n{}",
            tf.elr,
            vaddr,
            ESR_EL1.get(),
            access_flags,
            tf,
            tf.backtrace()
        );
    }
}

pub(crate) fn data_abort_access_flags(iss: u64) -> PageFaultFlags {
    let wnr = (iss & (1 << 6)) != 0; // WnR: Write not Read
    let cm = (iss & (1 << 8)) != 0; // CM: Cache maintenance
    if wnr & !cm {
        PageFaultFlags::WRITE
    } else {
        PageFaultFlags::READ
    }
}

fn handle_data_abort(tf: &mut TrapFrame, iss: u64) {
    let access_flags = data_abort_access_flags(iss);
    let vaddr = va!(FAR_EL1.get() as usize);

    info!("Data Abort @ {:#x}, fault_vaddr={:#x}, ESR={:#x} ({:?})", tf.elr, vaddr, ESR_EL1.get(), access_flags);
    // Only handle Translation fault and Permission fault
    if !is_valid_page_fault(iss)
    {
        panic!(
            "Invalid Data Abort ISS {:#x} @ {:#x}, fault_vaddr={:#x}, ESR={:#x} ({:?}):\n{:#x?}\n{}",
            iss,
            tf.elr,
            vaddr,
            ESR_EL1.get(),
            access_flags,
            tf,
            tf.backtrace()
        );
    }

    if core::hint::likely(handle_trap!(PAGE_FAULT, vaddr, access_flags)) {
        return;
    }

    if !tf.fixup_exception() {
        panic!(
            "Unhandled EL1 Data Abort @ {:#x}, fault_vaddr={:#x}, ESR={:#x} ({:?}):\n{:#x?}\n{}",
            tf.elr,
            vaddr,
            ESR_EL1.get(),
            access_flags,
            tf,
            tf.backtrace()
        );
    }
}

#[unsafe(no_mangle)]
fn handle_sync_exception(tf: &mut TrapFrame) {
    let esr = ESR_EL1.extract();
    let iss = esr.read(ESR_EL1::ISS);
    match esr.read_as_enum(ESR_EL1::EC) {
        Some(ESR_EL1::EC::Value::InstrAbortCurrentEL) => handle_instruction_abort(tf, iss),
        Some(ESR_EL1::EC::Value::DataAbortCurrentEL) => handle_data_abort(tf, iss),
        Some(ESR_EL1::EC::Value::Brk64) => {
            debug!("BRK #{:#x} @ {:#x} ", iss, tf.elr);
            tf.elr += 4;
        }
        _ => {
            panic!(
                "Unhandled synchronous exception @ {:#x}: ESR={:#x} (EC {:#08b}, ISS {:#x})\n{}",
                tf.elr,
                esr.get(),
                esr.read(ESR_EL1::EC),
                iss,
                tf.backtrace()
            );
        }
    }
}
