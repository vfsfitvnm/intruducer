use crate::{proc::ProcSym, VirtAddr};

pub(crate) fn gen_first(second_payload_path: &str) -> Vec<u8> {
    use tiny_asm::arm64::{AddrMode2::PreIndexed, Reg::*, TinyAsm};

    TinyAsm::new()
        // Push every general purpose register; are other registers necessary to push?
        .stp(PreIndexed, X0, X1, SP, -16)
        .stp(PreIndexed, X2, X3, SP, -16)
        .stp(PreIndexed, X4, X5, SP, -16)
        .stp(PreIndexed, X6, X7, SP, -16)
        .stp(PreIndexed, X8, X9, SP, -16)
        .stp(PreIndexed, X10, X11, SP, -16)
        .stp(PreIndexed, X12, X13, SP, -16)
        .stp(PreIndexed, X14, X15, SP, -16)
        .stp(PreIndexed, X16, X17, SP, -16)
        .stp(PreIndexed, X18, X19, SP, -16)
        .stp(PreIndexed, X20, X21, SP, -16)
        .stp(PreIndexed, X22, X23, SP, -16)
        .stp(PreIndexed, X24, X25, SP, -16)
        .stp(PreIndexed, X26, X27, SP, -16)
        .stp(PreIndexed, X28, X29, SP, -16)
        .stri(PreIndexed, X30, SP, -16)
        // Open second payload file
        .movi(X8, 56)
        .movi(X0, 0)
        .adr(X1, "second_payload_path")
        .movi(X2, 0)
        .movi(X3, 0)
        .svc(0)
        // Second payload file descriptor
        .movr(X14, X0)
        // Map the Second payload file to memory
        .movi(X8, 222)
        .movi(X0, 0)
        .movi(X1, 256)
        .movi(X2, 1 | 4)
        .movi(X3, 2)
        .movr(X4, X14)
        .movi(X5, 0)
        .svc(0)
        // Second payload code virtual address
        .movr(X15, X0)
        // Close Second payload file.
        .movi(X8, 57)
        .movr(X0, X14)
        .svc(0)
        // Execute second payload code
        .br(X15)
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
        .movi(X8, 56)
        .movi(X0, 0)
        .adr(X1, "mem_path")
        .movi(X2, 2)
        .movi(X3, 0)
        .svc(0)
        // Memory file descriptor.
        .movr(X15, X0)
        // Restore the original code.
        .movi(X8, 68)
        .movr(X0, X15)
        .adr(X1, "original_code")
        .movi(X2, original_code.len().try_into().unwrap())
        .ldrl(X3, "original_ip")
        .svc(0)
        // Close memory file.
        .movi(X8, 57)
        .movr(X0, X15)
        .svc(0)
        // Call dlopen
        .adr(X0, "lib_path")
        .movi(X1, 1)
        .ldrl(X28, "dlopen_addr")
        .blr(X28)
        // Pop every previously pushed register
        .ldri(PostIndexed, X30, SP, 16)
        .ldp(PostIndexed, X28, X29, SP, 16)
        .ldp(PostIndexed, X26, X27, SP, 16)
        .ldp(PostIndexed, X24, X25, SP, 16)
        .ldp(PostIndexed, X22, X23, SP, 16)
        .ldp(PostIndexed, X20, X21, SP, 16)
        .ldp(PostIndexed, X18, X19, SP, 16)
        .ldp(PostIndexed, X16, X17, SP, 16)
        .ldp(PostIndexed, X14, X15, SP, 16)
        .ldp(PostIndexed, X12, X13, SP, 16)
        .ldp(PostIndexed, X10, X11, SP, 16)
        .ldp(PostIndexed, X8, X9, SP, 16)
        .ldp(PostIndexed, X6, X7, SP, 16)
        .ldp(PostIndexed, X4, X5, SP, 16)
        .ldp(PostIndexed, X2, X3, SP, 16)
        .ldp(PostIndexed, X0, X1, SP, 16)
        // Restore the original execution flow
        .ldrl(X28, "original_ip")
        .br(X28)
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
