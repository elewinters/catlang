section .data
section .text
global sum
sum:
global _start
_start:
	mov rax, 1
	mov rdi, 1
	mov rsi, 65
	mov rdx, 1
	syscall
