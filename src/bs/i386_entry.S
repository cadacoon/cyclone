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
    # save multiboot paramters
    mov     %eax,       %edi
    mov     %ebx,       %esi

    # initialize stack (use physical addresses)
    mov     $stack_top - 0xF0000000, %eax
    sub     $16,        %eax
    add     $8,         %eax
    mov     %eax,       %esp
    mov     %eax,       %ebp

    # call main with multiboot parameters
    push    %ebp
    mov     %esp,       %ebp
    push    %esi
    push    %edi
    call    main_bootstrap
    ud2

.section .bss

stack_bottom:
    .zero 16384
stack_top:
