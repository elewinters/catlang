section .data
	L0: db `%d\n`, 0
section .text

extern puts
extern printf
extern putchar
extern strlen
global myputchar
myputchar:
	push rbp
	mov rbp, rsp
	sub rsp, 1

	mov byte [rbp-1], dil
	mov dil, [rbp-1]
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

global sum
sum:
	push rbp
	mov rbp, rsp

	mov dword [rbp-4], edi
	mov dword [rbp-8], esi
	mov r11d, [rbp-4]
	add r11d, [rbp-8]
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
	sub rsp, 4

	mov r11d, 5
	xor rax, rax
	xor rbx, rbx
	xor rdx, rdx
	mov eax, dword r11d
	mov ebx, 2
	idiv ebx
	mov r11d, eax
	mov dword [rbp-4], r11d

	xor rax, rax
	mov rdi, L0
	mov esi, [rbp-4]
	call printf

	mov eax, 0
	leave
	ret

