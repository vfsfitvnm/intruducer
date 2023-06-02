use crate::{os::VirtAddr, proc::ProcSym};

pub(crate) fn gen_first(second_payload_path: &str) -> Vec<u8> {
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
        // Open second payload file
        //
        // mov rax, 2
        .instr([0x48, 0xc7, 0xc0, 0x02, 0x00, 0x00, 0x00])
        // lea rdi, [rip + second_payload_path]
        .instr_with_ref([0x48, 0x8d, 0x3d], "second_payload_path")
        // mov rsi, 0
        .instr([0x48, 0xc7, 0xc6, 0x00, 0x00, 0x00, 0x00])
        // mov rdx, 0
        .instr([0x48, 0xc7, 0xc2, 0x00, 0x00, 0x00, 0x00])
        // syscall
        .instr([0x0f, 0x05])
        //
        // Second payload file descriptor
        //
        // mov r14, rax
        .instr([0x49, 0x89, 0xc6])
        //
        // Map the Second payload file to memory
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
        // Second payload code virtual address
        //
        // mov r15, rax
        .instr([0x49, 0x89, 0xc7])
        //
        // Close Second payload file
        //
        // mov rax, 3
        .instr([0x48, 0xc7, 0xc0, 0x03, 0x00, 0x00, 0x00])
        // mov rdi, r14
        .instr([0x4c, 0x89, 0xf7])
        // syscall
        .instr([0x0f, 0x05])
        //
        // Delete Second payload file.
        // Will fail on Android apps.
        //
        // mov rax, 87
        .instr([0x48, 0xc7, 0xc0, 0x57, 0x00, 0x00, 0x00])
        // lea rdi, [rip + second_payload_path]
        .instr_with_ref([0x48, 0x8d, 0x3d], "second_payload_path")
        // syscall
        .instr([0x0f, 0x05])
        //
        // Execute second payload code.
        //
        // jmp r15
        .instr([0x41, 0xff, 0xe7])
        //
        // Data
        //
        .label("second_payload_path")
        .asciiz(second_payload_path)
        .build()
}

pub(crate) fn gen_second(
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
        // mov rdx, 0
        .instr([0x48, 0xc7, 0xc2, 0x00, 0x00, 0x00, 0x00])
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
        // lea rdi, [rip + lib_path])
        .instr_with_ref([0x48, 0x8d, 0x3d], "lib_path")
        // mov rsi, 1
        .instr([0x48, 0xc7, 0xc6, 0x01, 0x00, 0x00, 0x00])
        // call [rip + dlopen_addr]
        .instr_with_ref([0xff, 0x15], "dlopen_addr")
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
