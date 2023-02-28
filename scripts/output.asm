section .data
	L0: db `meow\n`, 0
	L1: db `Hello, world!\n`, 0
section .text

global meow
meow:
	push rbp
	mov rbp, rsp

	mov rax, 1
	mov rdi, 1
	mov rsi, L0
	mov rdx, 5
	syscall

	pop rbp
	ret

global _start
_start:
	push rbp
	mov rbp, rsp
	sub rsp, 16

	mov qword [rbp-8], 1
	mov qword [rbp-16], 1
	mov rax, [rbp-8]
	mov rdi, [rbp-16]
	mov rsi, L1
	mov rdx, 14
	syscall

	call meow

	mov rax, 60
	mov rdi, 0
	syscall

	leave
	ret

