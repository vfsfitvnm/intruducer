use crate::{proc::ProcSym, VirtAddr};

pub(crate) fn gen_first(second_payload_path: &str) -> Vec<u8> {
    use tiny_asm::arm::{Reg::*, TinyAsm};

    TinyAsm::new()
        // Push every general purpose register, plus the link register (r14).
        .push([r0, r1, r2, r3, r4, r5, r6, r7, r8, r9, r10, r11, r12, lr])
        // Open second payload file.
        .movw(r7, 5)
        .adrl(r0, "second_payload_path")
        .movw(r1, 0)
        .movw(r2, 0)
        .svc(0)
        // Second payload file descriptor.
        .movr(r11, r0)
        // Map the Second payload file to memory.
        .movw(r7, 192)
        .movw(r0, 0)
        .movw(r1, 512)
        .movw(r2, 1 | 4)
        .movw(r3, 2)
        .movr(r4, r11)
        .movw(r5, 0)
        .svc(0)
        // Second payload code virtual address.
        .movr(r12, r0)
        // Close Second payload file.
        .movw(r7, 6)
        .movr(r0, r11)
        .svc(0)
        // Execute second payload code.
        .movr(pc, r12)
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
        .movw(r7, 5)
        .adrl(r0, "mem_path")
        .movw(r1, 2)
        .movw(r2, 0)
        .svc(0)
        // Memory file descriptor.
        .movr(r12, r0)
        // Restore the original code.
        .movw(r7, 181)
        .movr(r0, r12)
        .adrl(r1, "original_code")
        .movw(r2, original_code.len() as u16)
        .ldrl(r3, "original_ip")
        .ldrl(r4, "original_ip")
        .svc(0)
        // Close memory file.
        .movw(r7, 6)
        .movr(r0, r12)
        .svc(0)
        // Call dlopen.
        .adrl(r0, "lib_path")
        .movw(r1, 1)
        .movr(lr, pc)
        .ldrl(pc, "dlopen_addr")
        // Pop every previously pushed register
        .pop([r0, r1, r2, r3, r4, r5, r6, r7, r8, r9, r10, r11, r12, lr])
        // Restore the original execution flow
        .ldrl(pc, "original_ip")
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
