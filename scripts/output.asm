section .data
	L0: db `%d\n`, 0
section .text

extern puts
extern exit
extern printf
global main
main:
	push rbp
	mov rbp, rsp
	sub rsp, 4

	mov dword [rbp-4], 5
	xor rax, rax
	mov rdi, L0
	mov rsi, [rbp-4]
	call printf

	mov rdi, 0
	call exit

	leave
	ret

