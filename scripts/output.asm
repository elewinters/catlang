section .data
	L0: db `meow\n`, 0
	L1: db `Hello, world!\n`, 0
	L2: db `hi :D\n`, 0
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

global print
print:
	push rbp
	mov rbp, rsp

	mov qword [rbp-8], rdi
	mov qword [rbp-16], rsi
	mov rax, 1
	mov rdi, 1
	mov rsi, [rbp-8]
	mov rdx, [rbp-16]
	syscall

	pop rbp
	ret

global _start
_start:
	push rbp
	mov rbp, rsp
	sub rsp, 32

	mov qword [rbp-8], 1
	mov qword [rbp-16], 1
	mov rax, [rbp-8]
	mov rdi, [rbp-16]
	mov rsi, L1
	mov rdx, 14
	syscall

	mov qword [rbp-24], 6
	mov rdi, L2
	mov rsi, [rbp-24]
	call print

	call meow

	mov qword [rbp-32], 0
	mov rax, 60
	mov rdi, [rbp-32]
	syscall

	leave
	ret

