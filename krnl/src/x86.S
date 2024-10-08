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

    .section .multiboot.init, "a"

    .global __start
__start:
    .code32
    mov edi, eax
    mov esi, ebx

    // setup rudimentary page table
    // 00000000 - 003FFFFF: id
    // C0000000 - C03FFFFF: id
    // FFC00000 - FFFFFFFF: self ref
    mov eax, offset ptl1 - 0xC0000000
    mov ecx, 0x3                      // P R/W
1:
    mov [eax], ecx
    add eax  , 4
    add ecx  , 0x00001000
    cmp ecx  , 0x00400003
    jnz 1b

    mov eax                   , offset ptl2 - 0xC0000000
    mov long ptr [eax + 0xFFC], offset ptl2 - 0xC0000000 + 0x3 // P R/W 3FF: 2
    mov long ptr [eax + 0x000], offset ptl1 - 0xC0000000 + 0x3 // P R/W 000: 1
    mov long ptr [eax + 0xC00], offset ptl1 - 0xC0000000 + 0x3 // P R/W 300: 1
    mov cr3                   , eax

    // enable paging
    mov eax, cr0
    or  eax, 0x80000000 // CR0.PG
    mov cr0, eax

    // setup flat segmentation
    lgdt gdtr
    mov  ax, (2 << 3) // KDATA
    mov  ds, ax
    mov  es, ax
    mov  fs, ax
    mov  gs, ax
    mov  ss, ax
    //jmp  (1 << 3), 2f
    .byte 0xEA
    .long 2f
    .short (1 << 3) // KCODE
2:
    // setup stack and call main with multiboot arguments
    mov eax, offset stack_top
    mov esp, eax
    mov ebp, eax
    push esi
    push edi
    call main

gdtr:
    .short (8 * 7) - 1
    .long DESCRIPTOR_TABLE



    .section .bss

    .align 4096
stack_bottom:
    .zero 4096
stack_top:
ptl2:
    .zero 4096
ptl1:
    .zero 4096
