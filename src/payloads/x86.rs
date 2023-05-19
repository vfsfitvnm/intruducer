use crate::{proc::ProcSym, VirtAddr};

pub(crate) fn gen_first(second_payload_path: &str) -> Vec<u8> {
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
        // Open second payload file.
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
        // add ebx, second_payload_path
        .instr_with_ref([0x81, 0xc3], "second_payload_path")
        // mov ecx, 0
        .instr([0xb9, 0x00, 0x00, 0x00, 0x00])
        // mov edx, 0
        .instr([0xba, 0x00, 0x00, 0x00, 0x00])
        // int 0x80
        .instr([0xcd, 0x80])
        //
        // Second payload file descriptor.
        //
        // mov edi, eax
        .instr([0x89, 0xc7])
        //
        // Map the Second payload file to memory.
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
        // Second payload code virtual address.
        //
        // mov ebp, eax
        .instr([0x89, 0xc5])
        //
        // Close Second payload file.
        //
        // mov eax, 6
        .instr([0xb8, 0x06, 0x00, 0x00, 0x00])
        // mov ebx, edi
        .instr([0x89, 0xfb])
        // int 0x80
        .instr([0xcd, 0x80])
        //
        // Delete Second payload file.
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
        // add ebx, second_payload_path
        .instr_with_ref([0x81, 0xc3], "second_payload_path")
        // int 0x80
        .instr([0xcd, 0x80])
        //
        // Execute second payload code.
        //
        // jmp ebp
        .instr([0xff, 0xe5])
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
