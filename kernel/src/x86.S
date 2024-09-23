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

    # PTL1: identity
    mov     $ptl1 - 0xC0000000, %eax
    mov     $0x3,           %ecx # flags: present, write
1:
    mov     %ecx,           (%eax)
    add     $4,             %eax
    add     $0x00001000,    %ecx
    cmp     $0x00400003,    %ecx
    jnz     1b

    # PTL2
    mov     $ptl2 - 0xC0000000, %eax
    movl    $0x0010C003,    0x000(%eax) # identity
    movl    $0x0010C003,    0xC00(%eax) # system & kernel
    movl    $0x0010B003,    0xFFC(%eax) # recursive

    # Enable paging
    mov     %eax,           %cr3
    mov     %cr0,           %eax
    or      $0x80010000,    %eax
    mov     %eax,           %cr0

    # Init stack
    mov     $stack_top,     %eax
    sub     $16,            %eax
    add     $8,             %eax
    mov     %eax,           %esp
    mov     %eax,           %ebp

    # Call main with multiboot parameters
    push    %ebp
    mov     %esp,           %ebp
    push    %esi
    push    %edi
    call    main

.section .bss

stack_bottom:
    .zero 16384
stack_top:
ptl2:
    .zero 4096
ptl1:
    .zero 4096
