section .data
	L0: db `Hello, world!\n`, 0
section .text

global _start
_start:
	push rbp
	mov rbp, rsp

	mov qword [rbp-8], 1
	mov qword [rbp-16], 1
	mov qword [rbp-24], 60
	mov qword [rbp-32], 0
	mov rax, [rbp-8]
	mov rdi, [rbp-16]
	mov rsi, L0
	mov rdx, 14
	syscall

	mov rax, [rbp-24]
	mov rdi, [rbp-32]
	syscall

	pop rbp
	ret

