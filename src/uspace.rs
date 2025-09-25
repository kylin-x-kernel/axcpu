use memory_addr::VirtAddr;

use crate::{trap::PageFaultFlags, TrapFrame};

pub use crate::uctx::{ExceptionInfo, UserContext};

#[derive(Debug, Clone, Copy)]
pub enum ReturnReason {
    Unknown,
    Interrupt,
    Syscall,
    PageFault(VirtAddr, PageFaultFlags),
    Exception(ExceptionInfo),
}

pub enum ExceptionKind {
    Other,
    Breakpoint,
    IllegalInstruction,
    Misaligned,
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ExceptionTableEntry {
    from: usize,
    to: usize,
}

unsafe extern "C" {
    static _ex_table_start: [ExceptionTableEntry; 0];
    static _ex_table_end: [ExceptionTableEntry; 0];
}

impl TrapFrame {
    pub(crate) fn fixup_exception(&mut self) -> bool {
        let entries = unsafe {
            core::slice::from_raw_parts(
                _ex_table_start.as_ptr(),
                _ex_table_end
                    .as_ptr()
                    .offset_from_unsigned(_ex_table_start.as_ptr()),
            )
        };
        match entries.binary_search_by(|e| e.from.cmp(&self.ip())) {
            Ok(entry) => {
                self.set_ip(entries[entry].to);
                true
            }
            Err(_) => false,
        }
    }
}

pub(crate) fn init_exception_table() {
    // Sort exception table
    let ex_table = unsafe {
        core::slice::from_raw_parts_mut(
            _ex_table_start.as_ptr().cast_mut(),
            _ex_table_end
                .as_ptr()
                .offset_from_unsigned(_ex_table_start.as_ptr()),
        )
    };
    ex_table.sort_unstable();
}
