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
    mov     $ptl1 - 0xC0000000, %eax
    mov     $0x3,           %ecx # flags: present, write
1:
    mov     %ecx,           (%eax)
    add     $4,             %eax
    add     $0x00001000,    %ecx
    cmp     $0x00400003,    %ecx
    jnz     1b

    # Fill page table 2
    mov     $ptl2 - 0xC0000000, %eax
    movl    $ptl2 - 0xC0000000 + 0x3, 0xFFC(%eax) # self-reference
    movl    $ptl1 - 0xC0000000 + 0x3, 0x000(%eax) # identity
    movl    $ptl1 - 0xC0000000 + 0x3, 0xC00(%eax) # system & kernel

    # Enable paging
    mov     %eax,           %cr3
    mov     %cr0,           %eax
    or      $0x80010000,    %eax
    mov     %eax,           %cr0

    # Call main with multiboot parameters
    mov     $stack_top,     %eax
    sub     $16,            %eax
    add     $8,             %eax
    mov     %eax,           %esp
    mov     %eax,           %ebp

    push    %ebp
    mov     %esp,           %ebp
    push    %esi
    push    %edi
    call    main

.section .bss

stack_bottom:
    .zero 16384
stack_top:

.align 4096
ptl2:
    .zero 4096
ptl1:
    .zero 4096
