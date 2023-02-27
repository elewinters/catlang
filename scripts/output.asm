section .data
	L0: db `pussy :33\n`, 0
	L1: db `Hello, world!\n`, 0
section .text

global meow
meow:
	mov rax, 1
	mov rdi, 1
	mov rsi, L0
	mov rdx, 10
	syscall

	ret

global _start
_start:
	mov rax, 1
	mov rdi, 1
	mov rsi, L1
	mov rdx, 14
	syscall

	call meow

	mov rax, 60
	mov rdi, 0
	syscall

	ret

