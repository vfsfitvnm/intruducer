use crate::{proc::ProcSym, VirtAddr};

pub(crate) fn gen_first(second_payload_path: &str) -> Vec<u8> {
    use tiny_asm::arm64::{AddrMode2::PreIndexed, Reg::*, TinyAsm};

    TinyAsm::new()
        // Push every general purpose register; are other registers necessary to push?
        .stp(PreIndexed, x0, x1, sp, -16)
        .stp(PreIndexed, x2, x3, sp, -16)
        .stp(PreIndexed, x4, x5, sp, -16)
        .stp(PreIndexed, x6, x7, sp, -16)
        .stp(PreIndexed, x8, x9, sp, -16)
        .stp(PreIndexed, x10, x11, sp, -16)
        .stp(PreIndexed, x12, x13, sp, -16)
        .stp(PreIndexed, x14, x15, sp, -16)
        .stp(PreIndexed, x16, x17, sp, -16)
        .stp(PreIndexed, x18, x19, sp, -16)
        .stp(PreIndexed, x20, x21, sp, -16)
        .stp(PreIndexed, x22, x23, sp, -16)
        .stp(PreIndexed, x24, x25, sp, -16)
        .stp(PreIndexed, x26, x27, sp, -16)
        .stp(PreIndexed, x28, x29, sp, -16)
        .stri(PreIndexed, x30, sp, -16)
        // Open second payload file
        .movi(x8, 56)
        .movi(x0, 0)
        .adr(x1, "second_payload_path")
        .movi(x2, 0)
        .movi(x3, 0)
        .svc(0)
        // Second payload file descriptor
        .movr(x14, x0)
        // Map the Second payload file to memory
        .movi(x8, 222)
        .movi(x0, 0)
        .movi(x1, 256)
        .movi(x2, 1 | 4)
        .movi(x3, 2)
        .movr(x4, x14)
        .movi(x5, 0)
        .svc(0)
        // Second payload code virtual address
        .movr(x15, x0)
        // Close Second payload file.
        .movi(x8, 57)
        .movr(x0, x14)
        .svc(0)
        // Execute second payload code
        .br(x15)
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
    use tiny_asm::arm64::{AddrMode2::PostIndexed, Reg::*, TinyAsm};

    TinyAsm::new()
        // Open memory file (/proc/self/mem).
        .movi(x8, 56)
        .movi(x0, 0)
        .adr(x1, "mem_path")
        .movi(x2, 2)
        .movi(x3, 0)
        .svc(0)
        // Memory file descriptor.
        .movr(x15, x0)
        // Restore the original code.
        .movi(x8, 68)
        .movr(x0, x15)
        .adr(x1, "original_code")
        .movi(x2, original_code.len().try_into().unwrap())
        .ldrl(x3, "original_ip")
        .svc(0)
        // Close memory file.
        .movi(x8, 57)
        .movr(x0, x15)
        .svc(0)
        // Call dlopen
        .adr(x0, "lib_path")
        .movi(x1, 1)
        .ldrl(x28, "dlopen_addr")
        .blr(x28)
        // Pop every previously pushed register
        .ldri(PostIndexed, x30, sp, 16)
        .ldp(PostIndexed, x28, x29, sp, 16)
        .ldp(PostIndexed, x26, x27, sp, 16)
        .ldp(PostIndexed, x24, x25, sp, 16)
        .ldp(PostIndexed, x22, x23, sp, 16)
        .ldp(PostIndexed, x20, x21, sp, 16)
        .ldp(PostIndexed, x18, x19, sp, 16)
        .ldp(PostIndexed, x16, x17, sp, 16)
        .ldp(PostIndexed, x14, x15, sp, 16)
        .ldp(PostIndexed, x12, x13, sp, 16)
        .ldp(PostIndexed, x10, x11, sp, 16)
        .ldp(PostIndexed, x8, x9, sp, 16)
        .ldp(PostIndexed, x6, x7, sp, 16)
        .ldp(PostIndexed, x4, x5, sp, 16)
        .ldp(PostIndexed, x2, x3, sp, 16)
        .ldp(PostIndexed, x0, x1, sp, 16)
        // Restore the original execution flow
        .ldrl(x28, "original_ip")
        .br(x28)
        // Data
        .label("mem_path")
        .asciiz("/proc/self/mem")
        .align::<4>()
        .label("original_code")
        .bytes(original_code)
        .align::<4>()
        .label("original_ip")
        .qword(original_ip)
        .align::<4>()
        .label("lib_path")
        .asciiz(lib_path)
        .align::<4>()
        .label("dlopen_addr")
        .qword(dlopen.addr)
        .align::<4>()
        .build()
}
