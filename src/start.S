.extern main

.code32
.align 8

.section .text

.global __start
__start:
    # save multiboot paramters
    mov     %eax,       %edi
    mov     %ebx,       %esi

    # prepare stack
    mov     $stack_top, %eax
    sub     $16,        %eax
    add     $8,         %eax

    # set stack
    mov     %eax,       %esp
    mov     %eax,       %ebp

    push    %ebp
    mov     %esp,   %ebp

    # push multiboot parameters in reverse order to stack
    push    %esi
    push    %edi
    call    main

    # should never return
    ud2

.section .bss

.align 16384
stack_bottom:
    .zero 16384
stack_top:
