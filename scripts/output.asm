section .data
	L0: db `%d: %c\n`, 0
	L1: db `Hello, world!`, 0
section .text

extern puts
extern printf
extern putchar
extern exit
global myputchar
myputchar:
	push rbp
	mov rbp, rsp
	sub rsp, 1

	mov byte [rbp-1], dil
	movsx edi, byte [rbp-1]
	call putchar

	leave
	ret

global main
main:
	push rbp
	mov rbp, rsp
	sub rsp, 15

	mov dword [rbp-4], 12/3
	xor rax, rax
	mov rdi, L0
	mov esi, dword [rbp-4]
	mov edx, 67+3
	call printf

	mov byte [rbp-5], 65
	mov byte [rbp-6], 10
	movsx edi, byte [rbp-5]
	call putchar

	movsx edi, byte [rbp-6]
	call putchar

	movsx edi, byte [rbp-5]
	call myputchar

	movsx edi, byte [rbp-6]
	call myputchar

	mov al, byte [rbp-5]
	mov byte [rbp-7], al
	movsx edi, byte [rbp-7]
	call putchar

	movsx edi, byte [rbp-6]
	call putchar

	mov edi, 66+5
	call putchar

	mov edi, 10
	call putchar

	mov qword [rbp-15], L1
	mov rdi, qword [rbp-15]
	call puts

	mov rdi, 0
	call exit

	leave
	ret

