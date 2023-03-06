section .data
	L0: db `Hello, world!`, 0
	L1: db `/bin/sh`, 0
section .text

extern puts
extern strcpy
extern malloc
extern free
extern exit
extern system
global main
main:
	push rbp
	mov rbp, rsp
	sub rsp, 12

	mov rdi, 50
	call malloc
	mov qword [rbp-8], rax

	mov rdi, qword [rbp-8]
	mov rsi, L0
	call strcpy

	mov rdi, qword [rbp-8]
	call puts

	mov dword [rbp-12], L1

	mov edi, dword [rbp-12]
	call system

	mov edi, 0
	call exit

	leave
	ret

