// Copyright 2024 Kevin Ludwig
//
// Licensed under the Apache License, Version 2.0 (the "License")//
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

    // Fill page table 1
    mov eax, offset ptl1 - 0xC0000000
    mov ecx, 0x3                      // P R/W
1:
    mov [eax], ecx
    add eax  , 4
    add ecx  , 0x00001000
    cmp ecx  , 0x00400003
    jnz 1b

    // Fill page table 2
    mov eax                   , offset ptl2 - 0xC0000000
    mov long ptr [eax + 0xFFC], offset ptl2 - 0xC0000000 + 0x3 // P R/W 3FF: 2
    mov long ptr [eax + 0x000], offset ptl1 - 0xC0000000 + 0x3 // P R/W 000: 1
    mov long ptr [eax + 0xC00], offset ptl1 - 0xC0000000 + 0x3 // P R/W 300: 1
    mov cr3                   , eax

    // Enable paging
    mov eax, cr0
    or  eax, 0x80000000 // CR0.PG
    mov cr0, eax

    // Load GDT
    lgdt gdtr

    // Reload CS
    //jmp 8, main
    .byte 0xEA
    .long 2f
    .short 0x0008
2:
    // Reload DS
    mov eax, 16
    mov ds, eax
    mov es, eax
    mov fs, eax
    mov gs, eax
    mov ss, eax

    // Call main with multiboot parameters
    mov eax, offset stack_top
    sub eax, 16
    add eax, 8
    mov esp, eax
    mov ebp, eax

    push ebp
    mov  ebp, esp
    push esi
    push edi
    call main

gdtr:
    .short (8 * 6) - 1
    .long DESCRIPTOR_TABLE

    .section .bss

    .align 16384
stack_bottom:
    .zero 16384
stack_top:

    .align 4096
ptl2:
    .zero 4096
ptl1:
    .zero 4096
