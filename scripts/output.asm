section .data
	L0: db `meow\n`, 0
	L1: db `hi :D\n`, 0
	L2: db `Hello, world!\n`, 0
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

global args
args:
	push rbp
	mov rbp, rsp

	mov qword [rbp-8], rdi
	mov qword [rbp-16], rsi
	mov rax, [rbp-16]
	mov rdi, [rbp-8]
	mov rsi, L1
	mov rdx, 6
	syscall

	pop rbp
	ret

global _start
_start:
	push rbp
	mov rbp, rsp
	sub rsp, 21

	mov qword [rbp-8], 1
	mov dword [rbp-12], 50
	mov qword [rbp-20], 1
	mov rax, [rbp-8]
	mov rdi, [rbp-20]
	mov rsi, L2
	mov rdx, 14
	syscall

	call meow

	mov byte [rbp-21], 10
	mov rdi, [rbp-8]
	mov rsi, [rbp-20]
	call args

	mov rax, 60
	mov rdi, 0
	syscall

	leave
	ret

