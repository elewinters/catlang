section .data
	L0: db `exiting...`, 0
section .text

extern puts
extern exit
extern printf
extern putchar
extern strlen
extern strcmp
global myputchar
myputchar:
	push rbp
	mov rbp, rsp
	push rbx
	sub rsp, 8

	mov byte [rbp-1], dil
	mov dil, [rbp-1]
	call putchar

	mov dil, 10
	call putchar

	pop rbx
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
	mov ebx, [rbp-4]
	add ebx, [rbp-8]
	mov eax, ebx
	pop rbp
	ret

global sqr
sqr:
	push rbp
	mov rbp, rsp

	mov dword [rbp-4], edi
	mov ebx, [rbp-4]
	imul ebx, [rbp-4]
	mov eax, ebx
	pop rbp
	ret

global main
main:
	push rbp
	mov rbp, rsp
	push rbx
	sub rsp, 40

	mov dword [rbp-4], 5

	mov dword [rbp-8], 5

	mov dword [rbp-12], 5

	mov dword [rbp-16], 5

	mov dword [rbp-20], 5

	mov dword [rbp-24], 5

	mov dword [rbp-28], 5

	mov rdi, L0
	call puts

	mov eax, 0
	pop rbx
	leave
	ret

