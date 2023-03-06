section .data
	L0: db `gonna return a number!`, 0
	L1: db `Hello, world!`, 0
	L2: db `/bin/sh`, 0
section .text

extern puts
extern strcpy
extern malloc
extern free
extern exit
extern system
global number
number:
	push rbp
	mov rbp, rsp

	mov rax, 1
	mov rdi, 1
	mov rsi, L0
	mov rdx, 22
	syscall

	mov eax, dword 5
	pop rbp
	ret

global main
main:
	push rbp
	mov rbp, rsp
	sub rsp, 12

	mov rdi, 50
	call malloc
	mov qword [rbp-8], rax

	mov rdi, qword [rbp-8]
	mov rsi, L1
	call strcpy

	mov rdi, qword [rbp-8]
	call puts

	mov dword [rbp-12], L2

	mov edi, dword [rbp-12]
	call system

	mov edi, 0
	call exit

	leave
	ret

