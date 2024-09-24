# Copyright 2024 Kevin Ludwig
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

.extern main

.code32
.align 8

.section .multiboot.init, "a"

.global __start
__start:
    # Save multiboot paramters
    mov     %eax,           %edi
    mov     %ebx,           %esi

    # Fill page table 1
    mov     $ptl1 - 0xFFFFFFFF80000000, %eax
    mov     $0x3,           %ecx # flags: present, write
1:
    mov     %ecx,           (%eax)
    add     $8,             %eax
    add     $0x00001000,    %ecx
    cmp     $0x00400003,    %ecx
    jnz     1b

    # Fill page table 2
    mov     $ptl2 - 0xFFFFFFFF80000000, %eax
    movl    $ptl1 - 0xFFFFFFFF80000000, 0x000(%eax) # identity

    # Fill page table 3
    mov     $ptl3_identity - 0xFFFFFFFF80000000, %eax
    movl    $ptl2 - 0xFFFFFFFF80000000, 0x000(%eax) # identity
    mov     $ptl3 - 0xFFFFFFFF80000000, %eax
    movl    $ptl2 - 0xFFFFFFFF80000000, 0xFF0(%eax) # identity

    # Fill page table 4
    mov     $ptl4 - 0xFFFFFFFF80000000, %eax
    movl    $ptl4 - 0xFFFFFFFF80000000 + 0x3, 0xFF4(%eax) # self-reference
    movl    $ptl3_identity - 0xFFFFFFFF80000000, 0x000(%eax) # identity
    movl    $ptl3 - 0xFFFFFFFF80000000, 0xFF8(%eax) # system & kernel

    # Enable long mode
    mov     $0xC0000080,    %ecx
    rdmsr
    or      $0x00000100,    %ecx
    wrmsr

    # Enable paging
    mov     %eax,           %cr3
    mov     %cr0,           %eax
    or      $0x80010000,    %eax
    mov     %eax,           %cr0

.code64
    # Call main with multiboot parameters
    mov     $stack_top,     %rax
    sub     $16,            %rax
    add     $8,             %rax
    mov     %rax,           %rsp
    mov     %rax,           %rbp

    push    %rbp
    mov     %rsp,           %rbp
    push    %rsi
    push    %rdi
    call    main

.section .bss

stack_bottom:
    .zero 16384
stack_top:

.align 4096
ptl4:
    .zero 4096
ptl3_identity:
    .zero 4096
ptl3:
    .zero 4096
ptl2:
    .zero 4096
ptl1:
    .zero 4096
