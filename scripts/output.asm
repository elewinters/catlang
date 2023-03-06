section .data
	L0: db `Hello, world!`, 0
section .text

extern puts
extern strcpy
extern malloc
extern free
extern exit
global main
main:
	push rbp
	mov rbp, rsp
	sub rsp, 8

	mov rdi, 50
	call malloc
	mov qword [rbp-8], rax

	mov rdi, qword [rbp-8]
	mov rsi, L0
	call strcpy

	mov rdi, qword [rbp-8]
	call puts

	mov rdi, qword [rbp-8]
	call free

	mov edi, 0
	call exit

	leave
	ret

