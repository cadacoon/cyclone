// Copyright 2024 Kevin Ludwig
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

    .extern main

    .code32
    .align 8

    .section .multiboot.init, "a"

    .global __start
__start:
    // Save multiboot paramters
    mov edi, eax
    mov esi, ebx

    // Enable long mode
    mov   ecx, 0xC0000080 // EFER
    rdmsr
    or    eax, 0x00000100 // EFER.LME
    wrmsr

    // Enable PAE
    mov eax, cr4
    or  eax, 0x00000020 // CR4.PAE
    mov cr4, eax

    // Fill page table 1
    mov eax, offset ptl1 - 0xFFFFFFFF80000000
    mov ecx, 0x3                              // P R/W
1:
    mov [eax], ecx
    add eax  , 8
    add ecx  , 0x00001000
    cmp ecx  , 0x00200003
    jnz 1b

    // Fill page table 2
    mov eax                   , offset ptl2 - 0xFFFFFFFF80000000
    mov long ptr [eax + 0x000], offset ptl1 - 0xFFFFFFFF80000000 + 0x3 // P R/W 000: 1

    // Fill page table 3i
    mov eax                   , offset ptl3i - 0xFFFFFFFF80000000
    mov long ptr [eax + 0x000], offset ptl2 - 0xFFFFFFFF80000000 + 0x3 // P R/W 000: 2

    // Fill page table 3
    mov eax                   , offset ptl3 - 0xFFFFFFFF80000000
    mov long ptr [eax + 0xFF0], offset ptl2 - 0xFFFFFFFF80000000 + 0x3 // P R/W 776: 2

    // Fill page table 4
    mov eax                   , offset ptl4  - 0xFFFFFFFF80000000
    mov long ptr [eax + 0xFF0], offset ptl4  - 0xFFFFFFFF80000000 + 0x3 // P R/W 776: 4
    mov long ptr [eax + 0x000], offset ptl3i - 0xFFFFFFFF80000000 + 0x3 // P R/W 000: 3i
    mov long ptr [eax + 0xFF8], offset ptl3  - 0xFFFFFFFF80000000 + 0x3 // P R/W 777: 3
    mov cr3, eax

    // Enable paging
    mov eax, cr0
    or  eax, 0x80000000 // CR0.PG
    mov cr0, eax

    // Load GDT
    lgdt gdtr

    // Reload DS
    mov ax, (2 << 3)
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    // Reload CS
    //jmp (1 << 3), main
    .byte 0xEA
    .long 2f
    .short 0x0008
2:
    .code64
    // Call main with multiboot parameters
    mov rax, offset stack_top
    sub rax, 16
    add rax, 8
    mov rsp, rax
    mov rbp, rax

    push rbp
    mov  rbp, rsp
    push rsi
    push rdi
    call main

gdtr:
    .short (8 * 7) - 1
    .quad DESCRIPTOR_TABLE - 0xFFFFFFFF80000000

    .section .bss

    .align 16384
stack_bottom:
    .zero 16384
stack_top:

    .align 4096
ptl4:
    .zero 4096
ptl3i:
    .zero 4096
ptl3:
    .zero 4096
ptl2:
    .zero 4096
ptl1:
    .zero 4096
