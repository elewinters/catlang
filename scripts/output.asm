section .data
	L0: db `%hd\n`, 0
	L1: db `%hd\n`, 0
section .text

extern puts
extern printf
extern putchar
global myputchar
myputchar:
	push rbp
	mov rbp, rsp
	sub rsp, 2

	mov byte [rbp-1], dil
	mov byte [rbp-2], sil
	mov dil, [rbp-1]
	call putchar

	mov dil, [rbp-2]
	call putchar

	mov dil, 10
	call putchar

	leave
	ret

global abc
abc:
	push rbp
	mov rbp, rsp

	mov dil, 65
	call putchar

	mov dil, 66
	call putchar

	mov dil, 67
	call putchar

	mov dil, 10
	call putchar

	pop rbp
	ret

global main
main:
	push rbp
	mov rbp, rsp
	sub rsp, 2

	mov word [rbp-2], 50

	call abc

	xor rax, rax
	mov rdi, L0
	mov si, [rbp-2]
	call printf

	xor rax, rax
	mov rdi, L1
	mov r11w, [rbp-2]
	add r11w, 1
	mov si, r11w
	call printf

	mov eax, 0
	leave
	ret

