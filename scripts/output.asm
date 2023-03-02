section .data
	L0: db `Hello, world!\n`, 0
	L1: db `yass girl\n`, 0
	L2: db `meow :3\n`, 0
section .text

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

global exit
exit:
	push rbp
	mov rbp, rsp

	mov qword [rbp-8], rdi
	mov rax, 60
	mov rdi, [rbp-8]
	syscall

	pop rbp
	ret

global _start
_start:
	push rbp
	mov rbp, rsp

	mov rdi, L0
	mov rsi, 14
	call print

	mov rdi, L1
	mov rsi, 10
	call print

	mov rdi, L2
	mov rsi, 8
	call print

	mov rdi, 0
	call exit

	pop rbp
	ret

