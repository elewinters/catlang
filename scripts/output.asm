section .data
	L0: db `%d\n`, 0
	L1: db `%d\n`, 0
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

global div
div:
	push rbp
	mov rbp, rsp

	mov dword [rbp-4], edi
	mov dword [rbp-8], esi
	mov eax, edi
	idiv esi
	pop rbp
	ret

global div_real
div_real:
	push rbp
	mov rbp, rsp

	mov dword [rbp-4], edi
	mov dword [rbp-8], esi
	mov r11d, [rbp-4]
	xor rdx, rdx
	mov eax, r11d
	mov ebx, [rbp-8]
	idiv ebx
	mov r11d, eax
	mov eax, r11d
	pop rbp
	ret

global sqr
sqr:
	push rbp
	mov rbp, rsp

	mov dword [rbp-4], edi
	mov r11d, [rbp-4]
	imul r11d, [rbp-4]
	mov eax, r11d
	pop rbp
	ret

global main
main:
	push rbp
	mov rbp, rsp
	sub rsp, 8

	mov edi, 50
	call sqr
	mov r11d, eax
	mov edi, 50
	call sqr
	add r11d, eax
	xor rdx, rdx
	mov eax, r11d
	mov ebx, 2
	idiv ebx
	mov r11d, eax
	mov dword [rbp-4], r11d

	call abc

	xor rax, rax
	mov rdi, L0
	mov esi, [rbp-4]
	call printf

	mov edi, 200
	mov esi, 2
	call div_real
	mov dword [rbp-8], eax

	xor rax, rax
	mov rdi, L1
	mov esi, [rbp-8]
	call printf

	mov eax, 0
	leave
	ret

