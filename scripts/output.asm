section .data
	L0: db `Hello, world!`, 0
section .text

extern puts
extern exit
global main
main:
	push rbp
	mov rbp, rsp

	mov rdi, L0
	call puts

	mov rdi, 0
	call exit

	pop rbp
	ret

