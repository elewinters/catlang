section .data
	L0: db `Hello, 
	world!`, 0
section .text

global _start
_start:
	mov rax, 1
	mov rdi, 1
	mov rsi, L0
	mov rdx, 14
	syscall

	mov rax, 60
	mov rdi, 0
	syscall

