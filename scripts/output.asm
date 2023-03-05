section .data
	L0: db `10`, 0
	L1: db `%d\n`, 0
section .text

extern puts
extern printf
extern putchar
extern exit
extern atoi
global main
main:
	push rbp
	mov rbp, rsp
	sub rsp, 8

	mov edi, 65
	call putchar

	mov edi, 10
	call putchar

	mov rdi, L0
	call atoi
	mov dword [rbp-4], eax

	mov eax, dword [rbp-4]
	mov dword [rbp-8], eax

	xor rax, rax
	mov rdi, L1
	mov esi, dword [rbp-8]
	call printf

	mov rdi, 0
	call exit

	leave
	ret

