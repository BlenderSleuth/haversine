global Read_4x2
global Read_8x2
global Read_16x2
global Read_32x2
global Read_64x2

section .text

Read_4x2:
    xor rax, rax
    align 64
.loop:
    mov r8d, [rdx]
    mov r9d, [rdx + 4]
    add rax, 4
    cmp rax, rcx
    jb .loop
    ret
    
Read_8x2:
    xor rax, rax
    align 64
.loop:
    mov r8, [rdx]
    mov r9, [rdx + 8]
    add rax, 8
    cmp rax, rcx
    jb .loop
    ret
    
Read_16x2:
    xor rax, rax
    align 64
.loop:
    vmovdqu xmm0, [rdx]
    vmovdqu xmm1, [rdx + 16]
    add rax, 16
    cmp rax, rcx
    jb .loop
    ret

Read_32x2:
    xor rax, rax
    align 64
.loop:
    vmovdqu ymm0, [rdx]
    vmovdqu ymm1, [rdx + 32]
    add rax, 32
    cmp rax, rcx
    jb .loop
    ret
    
Read_64x2:
    xor rax, rax
    align 64
.loop:
    vmovdqu64 zmm0, [rdx]
    vmovdqu64 zmm1, [rdx + 64]
    add rax, 64
    cmp rax, rcx
    jb .loop
    ret