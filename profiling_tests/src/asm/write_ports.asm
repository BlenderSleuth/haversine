global Write_x1
global Write_x2
global Write_x3
global Write_x4

section .text

Write_x1:
    mov rax, 0xFF
    align 64
.loop:
    mov [rdx], rax
    sub rcx, 1
    jnle .loop
    ret
    
Write_x2:
    mov rax, 0xFF
    align 64
.loop:
    mov [rdx], rax
    mov [rdx], rax
    sub rcx, 2
    jnle .loop
    ret
    
Write_x3:
    mov rax, 0xFF
    align 64
.loop:
    mov [rdx], rax
    mov [rdx], rax
    mov [rdx], rax
    sub rcx, 3
    jnle .loop
    ret
    
Write_x4:
    mov rax, 0xFF
    align 64
.loop:
    mov [rdx], rax
    mov [rdx], rax
    mov [rdx], rax
    mov [rdx], rax
    sub rcx, 4
    jnle .loop
    ret
    