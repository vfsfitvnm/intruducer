use crate::{proc::ProcSym, VirtAddr};

pub(crate) fn gen_first(second_payload_path: &str) -> Vec<u8> {
    use tiny_asm::arm::{Reg::*, TinyAsm};

    TinyAsm::new()
        // Push every general purpose register, plus the link register (r14).
        .push([R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, LR])
        // Open second payload file.
        .movw(R7, 5)
        .adrl(R0, "second_payload_path")
        .movw(R1, 0)
        .movw(R2, 0)
        .svc(0)
        // Second payload file descriptor.
        .movr(R11, R0)
        // Map the Second payload file to memory.
        .movw(R7, 192)
        .movw(R0, 0)
        .movw(R1, 512)
        .movw(R2, 1 | 4)
        .movw(R3, 2)
        .movr(R4, R11)
        .movw(R5, 0)
        .svc(0)
        // Second payload code virtual address.
        .movr(R12, R0)
        // Close Second payload file.
        .movw(R7, 6)
        .movr(R0, R11)
        .svc(0)
        // Execute second payload code.
        .movr(PC, R12)
        // Data
        .label("second_payload_path")
        .asciiz(second_payload_path)
        .align::<4>()
        .build()
}

pub(crate) fn gen_second(
    original_code: &[u8],
    original_ip: VirtAddr,
    lib_path: &str,
    dlopen: &ProcSym,
) -> Vec<u8> {
    use tiny_asm::arm::{Reg::*, TinyAsm};

    TinyAsm::new()
        // Open memory file (/proc/self/mem).
        .movw(R7, 5)
        .adrl(R0, "mem_path")
        .movw(R1, 2)
        .movw(R2, 0)
        .svc(0)
        // Memory file descriptor.
        .movr(R12, R0)
        // Restore the original code.
        .movw(R7, 181)
        .movr(R0, R12)
        .adrl(R1, "original_code")
        .movw(R2, original_code.len() as u16)
        .ldrl(R3, "original_ip")
        .ldrl(R4, "original_ip")
        .svc(0)
        // Close memory file.
        .movw(R7, 6)
        .movr(R0, R12)
        .svc(0)
        // Call dlopen.
        .adrl(R0, "lib_path")
        .movw(R1, 1)
        .movr(LR, PC)
        .ldrl(PC, "dlopen_addr")
        // Pop every previously pushed register
        .pop([R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, LR])
        // Restore the original execution flow
        .ldrl(PC, "original_ip")
        // Data
        .label("mem_path")
        .asciiz("/proc/self/mem")
        .align::<4>()
        .label("original_code")
        .bytes(original_code)
        .align::<4>()
        .label("original_ip")
        .dword(original_ip.try_into().unwrap())
        .align::<4>()
        .label("lib_path")
        .asciiz(lib_path)
        .align::<4>()
        .label("dlopen_addr")
        .dword(dlopen.addr.try_into().unwrap())
        .align::<4>()
        .build()
}
