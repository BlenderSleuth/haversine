global CacheTest

section .text

; rcx: size
; rdx: mem ptr
; r8: cache size mask
CacheTest:
    xor r9, r9
    mov r10, rdx
    align 64
.loop:
    vmovdqu ymm0, [r10]
    vmovdqu ymm1, [r10 + 32]
    vmovdqu ymm2, [r10 + 64]
    vmovdqu ymm3, [r10 + 96]
    vmovdqu ymm0, [r10 + 128]
    vmovdqu ymm1, [r10 + 160]
    vmovdqu ymm2, [r10 + 192]
    vmovdqu ymm3, [r10 + 224]
    
    add r9, 256
    and r9, r8
    mov r10, rdx
    add r10, r9
    
    sub rcx, 256
    jnz .loop
    ret