use crate::{
    proc::{ProcClass, ProcSym},
    VirtAddr,
};

pub(crate) fn shell_code(class: &ProcClass, stage_path: &str) -> Vec<u8> {
    match class {
        ProcClass::ThirtyTwo => shell_code_32(stage_path),
        ProcClass::SixtyFour => shell_code_64(stage_path),
    }
}

pub(crate) fn stage_code(
    class: &ProcClass,
    original_code: &[u8],
    original_ip: VirtAddr,
    lib_path: &str,
    dlopen: &ProcSym,
) -> Vec<u8> {
    match class {
        ProcClass::ThirtyTwo => stage_code_32(original_code, original_ip, lib_path, dlopen),
        ProcClass::SixtyFour => stage_code_64(original_code, original_ip, lib_path, dlopen),
    }
}

#[cfg(target_arch = "x86_64")]
fn shell_code_64(stage_path: &str) -> Vec<u8> {
    use tiny_asm::x86_64::TinyAsm;

    TinyAsm::new()
        //
        // Push every general purpose register; are other registers necessary to push?
        //
        // push rax
        .instr([0x50])
        // push rbx
        .instr([0x53])
        // push rcx
        .instr([0x51])
        // push rdx
        .instr([0x52])
        // push rbp
        .instr([0x55])
        // push rsi
        .instr([0x56])
        // push rdi
        .instr([0x57])
        // push r8
        .instr([0x41, 0x50])
        // push r9
        .instr([0x41, 0x51])
        // push r10
        .instr([0x41, 0x52])
        // push r11
        .instr([0x41, 0x53])
        // push r12
        .instr([0x41, 0x54])
        // push r13
        .instr([0x41, 0x55])
        // push r14
        .instr([0x41, 0x56])
        // push r15
        .instr([0x41, 0x57])
        //
        // Open stage file
        //
        // mov rax, 2
        .instr([0x48, 0xc7, 0xc0, 0x02, 0x00, 0x00, 0x00])
        // lea rdi, [rip + stage_path]
        .instr_with_ref([0x48, 0x8d, 0x3d], "stage_path")
        // mov rsi, 0
        .instr([0x48, 0xc7, 0xc6, 0x00, 0x00, 0x00, 0x00])
        // mov rdx, 0
        .instr([0x48, 0xc7, 0xc2, 0x00, 0x00, 0x00, 0x00])
        // syscall
        .instr([0x0f, 0x05])
        //
        // Stage file descriptor
        //
        // mov r14, rax
        .instr([0x49, 0x89, 0xc6])
        //
        // Map the stage file to memory
        //
        // mov rax, 9
        .instr([0x48, 0xc7, 0xc0, 0x09, 0x00, 0x00, 0x00])
        // mov rdi, 0
        .instr([0x48, 0xc7, 0xc7, 0x00, 0x00, 0x00, 0x00])
        // mov rsi, 512
        .instr([0x48, 0xc7, 0xc6, 0x00, 0x02, 0x00, 0x00])
        // mov rdx, 1 | 4
        .instr([0x48, 0xc7, 0xc2, 0x05, 0x00, 0x00, 0x00])
        // mov r10, 2
        .instr([0x49, 0xc7, 0xc2, 0x02, 0x00, 0x00, 0x00])
        // mov r8, r14
        .instr([0x4d, 0x89, 0xf0])
        // mov r9, 0
        .instr([0x49, 0xc7, 0xc1, 0x00, 0x00, 0x00, 0x00])
        // syscall
        .instr([0x0f, 0x05])
        //
        // Stage code virtual address
        //
        // mov r15, rax
        .instr([0x49, 0x89, 0xc7])
        //
        // Close stage file
        //
        // mov rax, 3
        .instr([0x48, 0xc7, 0xc0, 0x03, 0x00, 0x00, 0x00])
        // mov rdi, r14
        .instr([0x4c, 0x89, 0xf7])
        // syscall
        .instr([0x0f, 0x05])
        //
        // Delete stage file.
        // Will fail on Android apps.
        //
        // mov rax, 87
        .instr([0x48, 0xc7, 0xc0, 0x57, 0x00, 0x00, 0x00])
        // lea rdi, [rip + stage_path]
        .instr_with_ref([0x48, 0x8d, 0x3d], "stage_path")
        // syscall
        .instr([0x0f, 0x05])
        //
        // Execute stage code.
        //
        // jmp r15
        .instr([0x41, 0xff, 0xe7])
        //
        // Data
        //
        .label("stage_path")
        .asciiz(stage_path)
        .build()
}

#[cfg(target_arch = "x86_64")]
fn stage_code_64(
    original_code: &[u8],
    original_ip: VirtAddr,
    lib_path: &str,
    dlopen: &ProcSym,
) -> Vec<u8> {
    use tiny_asm::x86_64::TinyAsm;

    TinyAsm::new()
        //
        // Open memory file
        //
        // mov rax, 2
        .instr([0x48, 0xc7, 0xc0, 0x02, 0x00, 0x00, 0x00])
        // lea rdi, [rip + proc_self_mem]
        .instr_with_ref([0x48, 0x8d, 0x3d], "mem_path")
        // mov rsi, 2
        .instr([0x48, 0xc7, 0xc6, 0x02, 0x00, 0x00, 0x00])
        // xor rdx, rdx
        .instr([0x48, 0x31, 0xd2])
        // syscall
        .instr([0x0f, 0x05])
        //
        // Memory file descriptor
        //
        // mov r15, rax
        .instr([0x49, 0x89, 0xc7])
        //
        // Restore the original code
        //
        // mov rax, 18
        .instr([0x48, 0xc7, 0xc0, 0x12, 0x00, 0x00, 0x00])
        // mov rdi, r15
        .instr([0x4c, 0x89, 0xff])
        // lea rsi, [rip + original_code]
        .instr_with_ref([0x48, 0x8d, 0x35], "original_code")
        // mov rdx, [rip + original_code_len]
        .instr_with_ref([0x48, 0x8b, 0x15], "original_code_len")
        // mov r10, [rip + original_ip]
        .instr_with_ref([0x4c, 0x8b, 0x15], "original_ip")
        // syscall
        .instr([0x0f, 0x05])
        //
        // Close memory file.
        //
        // mov rax, 3
        .instr([0x48, 0xc7, 0xc0, 0x03, 0x00, 0x00, 0x00])
        // mov rdi, r15
        .instr([0x4c, 0x89, 0xff])
        // syscall
        .instr([0x0f, 0x05])
        //
        // Align the stack to a 16 byte boundary
        //
        // mov rbp, rsp
        .instr([0x48, 0x89, 0xe5])
        // and rsp, -16
        .instr([0x48, 0x83, 0xe4, 0xf0])
        //
        // Call dlopen
        //
        // mov rax, [rip + dlopen_addr]
        .instr_with_ref([0x48, 0x8b, 0x05], "dlopen_addr")
        // lea rdi, [rip + lib_path])
        .instr_with_ref([0x48, 0x8d, 0x3d], "lib_path")
        // mov rsi, 1
        .instr([0x48, 0xc7, 0xc6, 0x01, 0x00, 0x00, 0x00])
        // call rax
        .instr([0xff, 0xd0])
        //
        // Restore the stack
        //
        // mov rsp, rbp
        .instr([0x48, 0x89, 0xec])
        //
        // Pop every previously pushed register
        //
        // pop r15
        .instr([0x41, 0x5f])
        // pop r14
        .instr([0x41, 0x5e])
        // pop r13
        .instr([0x41, 0x5d])
        // pop r12
        .instr([0x41, 0x5c])
        // pop r11
        .instr([0x41, 0x5b])
        // pop r10
        .instr([0x41, 0x5a])
        // pop r9
        .instr([0x41, 0x59])
        // pop r8
        .instr([0x41, 0x58])
        // pop rdi
        .instr([0x5f])
        // pop rsi
        .instr([0x5e])
        // pop rbp
        .instr([0x5d])
        // pop rdx
        .instr([0x5a])
        // pop rcx
        .instr([0x59])
        // pop rbx
        .instr([0x5b])
        // pop rax
        .instr([0x58])
        //
        // Restore the original execution flow
        //
        // jmp [rip + original_ip]
        .instr_with_ref([0xff, 0x25], "original_ip")
        //
        // Data
        //
        .label("mem_path")
        .asciiz("/proc/self/mem")
        .label("original_code")
        .bytes(original_code)
        .label("original_code_len")
        .qword(original_code.len().try_into().unwrap())
        .label("original_ip")
        .qword(original_ip)
        .label("lib_path")
        .asciiz(lib_path)
        .label("dlopen_addr")
        .qword(dlopen.addr)
        .build()
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
fn shell_code_32(stage_path: &str) -> Vec<u8> {
    use tiny_asm::x86::TinyAsm;

    TinyAsm::new()
        //
        // Push every general purpose register; are other registers necessary to push?
        //
        // push eax
        .instr([0x50])
        // push ebx
        .instr([0x53])
        // push ecx
        .instr([0x51])
        // push edx
        .instr([0x52])
        // push ebp
        .instr([0x55])
        // push esi
        .instr([0x56])
        // push edi
        .instr([0x57])
        //
        // Open stage file.
        //
        // mov eax, 5
        .instr([0xb8, 0x05, 0x00, 0x00, 0x00])
        // call 5
        .instr([0xe8, 0x00, 0x00, 0x00, 0x00])
        // next1: pop ebx
        .label("next1")
        .instr([0x5b])
        // sub ebx, next1
        .instr_with_ref([0x81, 0xeb], "next1")
        // add ebx, stage_path
        .instr_with_ref([0x81, 0xc3], "stage_path")
        // mov ecx, 0
        .instr([0xb9, 0x00, 0x00, 0x00, 0x00])
        // mov edx, 0
        .instr([0xba, 0x00, 0x00, 0x00, 0x00])
        // int 0x80
        .instr([0xcd, 0x80])
        //
        // Stage file descriptor.
        //
        // mov edi, eax
        .instr([0x89, 0xc7])
        //
        // Map the stage file to memory.
        //
        // mov eax, 192
        .instr([0xb8, 0xc0, 0x00, 0x00, 0x00])
        // mov ebx, 0
        .instr([0xbb, 0x00, 0x00, 0x00, 0x00])
        // mov ecx, 512
        .instr([0xb9, 0x00, 0x02, 0x00, 0x00])
        // mov edx, 1 | 4
        .instr([0xba, 0x05, 0x00, 0x00, 0x00])
        // mov esi, 2
        .instr([0xbe, 0x02, 0x00, 0x00, 0x00])
        // mov ebp, 0
        .instr([0xbd, 0x00, 0x00, 0x00, 0x00])
        // int 0x80
        .instr([0xcd, 0x80])
        //
        // Stage code virtual address.
        //
        // mov ebp, eax
        .instr([0x89, 0xc5])
        //
        // Close stage file.
        //
        // mov eax, 6
        .instr([0xb8, 0x06, 0x00, 0x00, 0x00])
        // mov ebx, edi
        .instr([0x89, 0xfb])
        // int 0x80
        .instr([0xcd, 0x80])
        //
        // Delete stage file.
        // Will fail on Android apps.
        //
        // mov eax, 10
        .instr([0xb8, 0x0a, 0x00, 0x00, 0x00])
        // call 5
        .instr([0xe8, 0x00, 0x00, 0x00, 0x00])
        // next2: pop ebx
        .label("next2")
        .instr([0x5b])
        // sub ebx, next2
        .instr_with_ref([0x81, 0xeb], "next2")
        // add ebx, stage_path
        .instr_with_ref([0x81, 0xc3], "stage_path")
        // int 0x80
        .instr([0xcd, 0x80])
        //
        // Execute stage code.
        //
        // jmp ebp
        .instr([0xff, 0xe5])
        //
        // Data
        //
        .label("stage_path")
        .asciiz(stage_path)
        .build()
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
fn stage_code_32(
    original_code: &[u8],
    original_ip: VirtAddr,
    lib_path: &str,
    dlopen: &ProcSym,
) -> Vec<u8> {
    use tiny_asm::x86::TinyAsm;

    TinyAsm::new()
        //
        // Open memory file (/proc/self/mem).
        //
        // mov eax, 5
        .instr([0xb8, 0x05, 0x00, 0x00, 0x00])
        // call 5
        .instr([0xe8, 0x00, 0x00, 0x00, 0x00])
        // next0: pop ebx
        .label("next0")
        .instr([0x5b])
        // sub ebx, next0
        .instr_with_ref([0x81, 0xeb], "next0")
        // add ebx, proc_self_mem
        .instr_with_ref([0x81, 0xc3], "mem_path")
        // mov ecx, 2
        .instr([0xb9, 0x02, 0x00, 0x00, 0x00])
        // mov edx, 0
        .instr([0xba, 0x00, 0x00, 0x00, 0x00])
        // int 0x80
        .instr([0xcd, 0x80])
        //
        // Memory file descriptor.
        //
        // mov ebx, eax
        .instr([0x89, 0xc3])
        //
        // Restore the original code.
        //
        // mov eax, 181
        .instr([0xb8, 0xb5, 0x00, 0x00, 0x00])
        // call 5
        .instr([0xe8, 0x00, 0x00, 0x00, 0x00])
        // next1: pop ecx
        .label("next1")
        .instr([0x59])
        // sub ecx, next1
        .instr_with_ref([0x81, 0xe9], "next1")
        // add ecx, original_code
        .instr_with_ref([0x81, 0xc1], "original_code")
        // mov edx, original_code_len
        .instr([0xba])
        .instr((original_code.len() as u32).to_le_bytes())
        // mov esi, instruction_pointer
        .instr([0xbe])
        .instr((original_ip as u32).to_le_bytes())
        // mov edi, 0
        .instr([0xbf, 0x00, 0x00, 0x00, 0x00])
        // int 0x80
        .instr([0xcd, 0x80])
        //
        // Close memory file.
        //
        // mov eax, 6
        .instr([0xb8, 0x06, 0x00, 0x00, 0x00])
        // int 0x80
        .instr([0xcd, 0x80])
        //
        // Make a new call frame.
        //
        // push ebp
        .instr([0x55])
        // mov ebp, esp
        .instr([0x89, 0xe5])
        //
        // Call dlopen.
        //
        // mov eax, dlopen_addr
        .instr([0xb8])
        .instr((dlopen.addr as u32).to_le_bytes())
        // push 1
        .instr([0x6a, 0x01])
        // call 5
        .instr([0xe8, 0x00, 0x00, 0x00, 0x00])
        // next2: pop ebx
        .label("next2")
        .instr([0x5b])
        // sub ebx, next2
        .instr_with_ref([0x81, 0xeb], "next2")
        // add ebx, lib_path
        .instr_with_ref([0x81, 0xc3], "lib_path")
        // push ebx
        .instr([0x53])
        // call eax
        .instr([0xff, 0xd0])
        //
        // Restore the old call frame
        //
        // mov esp, ebp
        .instr([0x89, 0xec])
        // pop ebp
        .instr([0x5d])
        //
        // Pop every previously pushed register.
        //
        // pop edi
        .instr([0x5f])
        // pop esi
        .instr([0x5e])
        // pop ebp
        .instr([0x5d])
        // pop edx
        .instr([0x5a])
        // pop ecx
        .instr([0x59])
        // pop ebx
        .instr([0x5b])
        // pop eax
        .instr([0x58])
        //
        // Restore the original execution flow.
        //
        // push original_ip
        .instr([0x68])
        .instr((original_ip as u32).to_le_bytes())
        // ret
        .instr([0xc3])
        //
        // Data
        //
        .label("mem_path")
        .asciiz("/proc/self/mem")
        .label("original_code")
        .bytes(original_code)
        .label("lib_path")
        .asciiz(lib_path)
        .build()
}

#[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
fn shell_code_32(stage_path: &str) -> Vec<u8> {
    use tiny_asm::arm::{Reg::*, TinyAsm};

    TinyAsm::new()
        // Push every general purpose register, plus the link register (r14).
        .push([R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, LR])
        // Open stage file.
        .movw(R7, 5)
        .adrl(R0, "stage_path")
        .movw(R1, 0)
        .movw(R2, 0)
        .svc(0)
        // Stage file descriptor.
        .movr(R11, R0)
        // Map the stage file to memory.
        .movw(R7, 192)
        .movw(R0, 0)
        .movw(R1, 512)
        .movw(R2, 1 | 4)
        .movw(R3, 2)
        .movr(R4, R11)
        .movw(R5, 0)
        .svc(0)
        // Stage code virtual address.
        .movr(R12, R0)
        // Close stage file.
        .movw(R7, 6)
        .movr(R0, R11)
        .svc(0)
        // Execute stage code.
        .movr(PC, R12)
        // Data
        .label("stage_path")
        .asciiz(stage_path)
        .align::<4>()
        .build()
}

#[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
fn stage_code_32(
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

#[cfg(target_arch = "aarch64")]
fn shell_code_64(stage_path: &str) -> Vec<u8> {
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
        // Open stage file
        .movi(X8, 56)
        .movi(X0, 0)
        .adr(X1, "stage_path")
        .movi(X2, 0)
        .movi(X3, 0)
        .svc(0)
        // Stage file descriptor
        .movr(X14, X0)
        // Map the stage file to memory
        .movi(X8, 222)
        .movi(X0, 0)
        .movi(X1, 256)
        .movi(X2, 1 | 4)
        .movi(X3, 2)
        .movr(X4, X14)
        .movi(X5, 0)
        .svc(0)
        // Stage code virtual address
        .movr(X15, X0)
        // Close stage file.
        .movi(X8, 57)
        .movr(X0, X14)
        .svc(0)
        // Execute stage code
        .br(X15)
        // Data
        .label("stage_path")
        .asciiz(stage_path)
        .align::<4>()
        .build()
}

#[cfg(target_arch = "aarch64")]
fn stage_code_64(
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
