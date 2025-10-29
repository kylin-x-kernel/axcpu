use log::{error, info};

/// 读取当前异常级别
#[inline]
pub fn current_el() -> u8 {
    let current_el: u64;
    unsafe {
        core::arch::asm!("mrs {}, CurrentEL", out(reg) current_el);
    }
    ((current_el >> 2) & 0b11) as u8
}

/// 读取 PAN 状态
#[inline]
pub fn read_pan() -> bool {
    let spsr: u64;
    unsafe {
        core::arch::asm!(
            "mrs {}, SPSR_EL1",
            out(reg) spsr,
            options(nomem, nostack, preserves_flags)
        );
    }
    // PAN 位在 bit 22
    (spsr & (1 << 22)) != 0
}

/// 读取 UAO 状态
#[inline]
pub fn read_uao() -> bool {
    let spsr: u64;
    unsafe {
        core::arch::asm!(
            "mrs {}, SPSR_EL1",
            out(reg) spsr,
            options(nomem, nostack, preserves_flags)
        );
    }

    (spsr & (1 << 23)) != 0
}

/// 启用 UAO
#[inline]
pub fn enable_uao() {
    unsafe {
        core::arch::asm!("msr UAO, #1", options(nomem, nostack));
    }
}

/// 读取 ESR_EL1 (Exception Syndrome Register)
#[inline]
pub fn read_esr_el1() -> u64 {
    let esr: u64;
    unsafe {
        core::arch::asm!("mrs {}, ESR_EL1", out(reg) esr);
    }
    esr
}

/// 读取 FAR_EL1 (Fault Address Register)
#[inline]
pub fn read_far_el1() -> u64 {
    let far: u64;
    unsafe {
        core::arch::asm!("mrs {}, FAR_EL1", out(reg) far);
    }
    far
}

/// 读取 SCTLR_EL1
#[inline]
pub fn read_sctlr_el1() -> u64 {
    let sctlr: u64;
    unsafe {
        core::arch::asm!("mrs {}, SCTLR_EL1", out(reg) sctlr);
    }
    sctlr
}

/// 读取 TCR_EL1 (Translation Control Register)
#[inline]
pub fn read_tcr_el1() -> u64 {
    let tcr: u64;
    unsafe {
        core::arch::asm!("mrs {}, TCR_EL1", out(reg) tcr);
    }
    tcr
}

/// 解析异常综合信息
pub fn parse_esr(esr: u64) {
    let ec = (esr >> 26) & 0x3f;
    let il = (esr >> 25) & 1;
    let iss = esr & 0x1ffffff;

    info!("ESR_EL1: 0x{:016x}", esr);
    info!(
        "  EC (Exception Class): 0x{:02x} - {}",
        ec,
        exception_class_name(ec)
    );
    info!("  IL (Instruction Length): {}", il);
    info!("  ISS (Instruction Specific Syndrome): 0x{:07x}", iss);

    // 如果是数据访问异常
    if ec == 0x20 || ec == 0x24 || ec == 0x25 {
        let dfsc = iss & 0x3f;
        let wnr = (iss >> 6) & 1;
        let s1ptw = (iss >> 7) & 1;
        let cm = (iss >> 8) & 1;
        let ea = (iss >> 9) & 1;
        let fnv = (iss >> 10) & 1;

        info!("  Data Abort Info:");
        info!(
            "    DFSC (Data Fault Status): 0x{:02x} - {}",
            dfsc,
            fault_status_name(dfsc)
        );
        info!(
            "    WnR (Write not Read): {} ({})",
            wnr,
            if wnr == 1 { "Write" } else { "Read" }
        );
        info!("    S1PTW (Stage 1 translation table walk): {}", s1ptw);
        info!("    CM (Cache maintenance): {}", cm);
        info!("    EA (External abort): {}", ea);
        info!("    FnV (FAR not Valid): {}", fnv);
    }
}

/// 获取异常类别名称
fn exception_class_name(ec: u64) -> &'static str {
    match ec {
        0x00 => "Unknown reason",
        0x01 => "Trapped WFI or WFE",
        0x03 => "Trapped MCR or MRC (CP15)",
        0x04 => "Trapped MCRR or MRRC (CP15)",
        0x05 => "Trapped MCR or MRC (CP14)",
        0x06 => "Trapped LDC or STC",
        0x07 => "Trapped SIMD or FP",
        0x0C => "Trapped MRRC (CP14)",
        0x0E => "Illegal Execution state",
        0x11 => "SVC instruction at EL0",
        0x12 => "HVC instruction at EL1",
        0x13 => "SMC instruction at EL1",
        0x15 => "SVC instruction at EL1",
        0x16 => "HVC instruction at EL2",
        0x17 => "SMC instruction at EL2",
        0x18 => "Trapped MSR, MRS or System instruction",
        0x1F => "Implementation defined exception",
        0x20 => "Instruction Abort from lower EL",
        0x21 => "Instruction Abort from same EL",
        0x22 => "PC alignment fault",
        0x24 => "Data Abort from lower EL",
        0x25 => "Data Abort from same EL",
        0x26 => "SP alignment fault",
        0x28 => "Trapped FP exception (AArch32)",
        0x2C => "Trapped FP exception (AArch64)",
        0x2F => "SError interrupt",
        0x30 => "Breakpoint from lower EL",
        0x31 => "Breakpoint from same EL",
        0x32 => "Software Step from lower EL",
        0x33 => "Software Step from same EL",
        0x34 => "Watchpoint from lower EL",
        0x35 => "Watchpoint from same EL",
        0x38 => "BKPT instruction",
        0x3C => "BRK instruction",
        _ => "Reserved or unknown",
    }
}

/// 获取错误状态名称
fn fault_status_name(dfsc: u64) -> &'static str {
    match dfsc {
        0b000000 => "Address size fault, level 0",
        0b000001 => "Address size fault, level 1",
        0b000010 => "Address size fault, level 2",
        0b000011 => "Address size fault, level 3",
        0b000100 => "Translation fault, level 0",
        0b000101 => "Translation fault, level 1",
        0b000110 => "Translation fault, level 2",
        0b000111 => "Translation fault, level 3",
        0b001001 => "Access flag fault, level 1",
        0b001010 => "Access flag fault, level 2",
        0b001011 => "Access flag fault, level 3",
        0b001101 => "Permission fault, level 1",
        0b001110 => "Permission fault, level 2",
        0b001111 => "Permission fault, level 3",
        0b010000 => "Synchronous External abort, not on translation table walk",
        0b010100 => "Synchronous External abort on translation table walk, level 0",
        0b010101 => "Synchronous External abort on translation table walk, level 1",
        0b010110 => "Synchronous External abort on translation table walk, level 2",
        0b010111 => "Synchronous External abort on translation table walk, level 3",
        0b011000 => "Synchronous parity error on memory access, not on translation table walk",
        0b011100 => "Synchronous parity error on memory access on translation table walk, level 0",
        0b011101 => "Synchronous parity error on memory access on translation table walk, level 1",
        0b011110 => "Synchronous parity error on memory access on translation table walk, level 2",
        0b011111 => "Synchronous parity error on memory access on translation table walk, level 3",
        0b100001 => "Alignment fault",
        0b110000 => "TLB conflict abort",
        0b110100 => "Implementation defined fault (Lockdown)",
        0b110101 => "Implementation defined fault (Unsupported Exclusive or Atomic access)",
        _ => "Reserved or unknown fault status",
    }
}

/// 解析 SCTLR_EL1
pub fn parse_sctlr(sctlr: u64) {
    info!("SCTLR_EL1: 0x{:016x}", sctlr);
    info!("  M   (MMU enable):            {}", sctlr & 1);
    info!("  A   (Alignment check):       {}", (sctlr >> 1) & 1);
    info!("  C   (Data cache):            {}", (sctlr >> 2) & 1);
    info!("  SA  (Stack Alignment EL1):   {}", (sctlr >> 3) & 1);
    info!("  SA0 (Stack Alignment EL0):   {}", (sctlr >> 4) & 1);
    info!("  CP15BEN:                     {}", (sctlr >> 5) & 1);
    info!("  nAA (non-Aligned Access):    {}", (sctlr >> 6) & 1);
    info!("  ITD (IT Disable):            {}", (sctlr >> 7) & 1);
    info!("  SED (SETEND Disable):        {}", (sctlr >> 8) & 1);
    info!("  UMA (User Mask Access):      {}", (sctlr >> 9) & 1);
    info!("  I   (Instruction cache):     {}", (sctlr >> 12) & 1);
    info!("  DZE (DC ZVA Enable):         {}", (sctlr >> 14) & 1);
    info!("  UCT (User Cache Type):       {}", (sctlr >> 15) & 1);
    info!("  nTWI (WFI trap):             {}", (sctlr >> 16) & 1);
    info!("  nTWE (WFE trap):             {}", (sctlr >> 18) & 1);
    info!("  WXN (Write implies XN):      {}", (sctlr >> 19) & 1);
    info!("  UWXN (User WXN):             {}", (sctlr >> 20) & 1);
    info!("  E0E (Endianness EL0):        {}", (sctlr >> 24) & 1);
    info!("  EE  (Endianness EL1):        {}", (sctlr >> 25) & 1);
}

/// 解析 TCR_EL1
pub fn parse_tcr(tcr: u64) {
    info!("TCR_EL1: 0x{:016x}", tcr);
    info!("  T0SZ: {}", tcr & 0x3f);
    info!("  EPD0 (disable TTBR0 walk): {}", (tcr >> 7) & 1);
    info!("  IRGN0 (Inner cacheability): {}", (tcr >> 8) & 0b11);
    info!("  ORGN0 (Outer cacheability): {}", (tcr >> 10) & 0b11);
    info!("  SH0 (Shareability): {}", (tcr >> 12) & 0b11);
    info!("  TG0 (Granule size): {}", (tcr >> 14) & 0b11);
    info!("  T1SZ: {}", (tcr >> 16) & 0x3f);
    info!("  A1 (ASID select): {}", (tcr >> 22) & 1);
    info!("  EPD1 (disable TTBR1 walk): {}", (tcr >> 23) & 1);
    info!("  TG1 (Granule size): {}", (tcr >> 30) & 0b11);
    info!("  IPS (Physical address size): {}", (tcr >> 32) & 0b111);
    info!("  TBI0 (Top Byte Ignore): {}", (tcr >> 37) & 1);
    info!("  TBI1 (Top Byte Ignore): {}", (tcr >> 38) & 1);
}

/// 完整的系统状态转储
pub fn dump_full_state(user_addr: usize, pte_value: u64) {
    info!("========================================");
    info!("User Memory Access Debug");
    info!("========================================");
    info!("Target User Address: 0x{:016x}", user_addr);
    info!("PTE Value: 0x{:016x}", pte_value);
    info!("");

    // 当前状态
    info!("Current State:");
    info!("  Exception Level: EL{}", current_el());
    info!("  PAN (Privileged Access Never): {}", read_pan());
    info!("  UAO (User Access Override): {}", read_uao());
    info!("");

    // 系统寄存器
    parse_sctlr(read_sctlr_el1());
    info!("");
    parse_tcr(read_tcr_el1());
    info!("");

    // 异常信息
    info!("Exception Information:");
    let esr = read_esr_el1();
    let far = read_far_el1();
    parse_esr(esr);
    info!("FAR_EL1 (Fault Address): 0x{:016x}", far);
    info!("========================================");
}
