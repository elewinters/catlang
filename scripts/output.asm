section .data
	L0: db `%d\n`, 0
	L1: db `hello`, 0
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

	push r11
	call putchar
	pop r11


	mov dil, 10

	push r11
	call putchar
	pop r11


	leave
	ret

global abc
abc:
	push rbp
	mov rbp, rsp

	mov dil, 65

	push r11
	call putchar
	pop r11


	mov dil, 66

	push r11
	call putchar
	pop r11


	mov dil, 67

	push r11
	call putchar
	pop r11


	mov dil, 10

	push r11
	call putchar
	pop r11


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

	mov r11d, 1
	add r11d, 0
	mov edi, r11d
	mov esi, 1

	push r11
	call sum
	pop r11

	mov r11d, 1
	add r11d, 0
	mov edi, 1
	mov esi, r11d

	push r11
	call sum
	pop r11

	mov edi, eax
	mov esi, eax

	push r11
	call sum
	pop r11

	mov edi, eax

	push r11
	call sqr
	pop r11

	mov r11d, eax
	mov r11d, 1
	add r11d, 1
	mov edi, r11d

	push r11
	call sqr
	pop r11

	add r11d, eax
	mov edi, 2

	push r11
	call sqr
	pop r11

	add r11d, eax
	mov edi, 2

	push r11
	call sqr
	pop r11

	add r11d, eax
	mov edi, 2

	push r11
	call sqr
	pop r11

	add r11d, eax
	mov edi, 2

	push r11
	call sqr
	pop r11

	add r11d, eax
	mov dword [rbp-4], r11d

	xor rax, rax
	mov rdi, L0
	mov esi, [rbp-4]

	push r11
	call printf
	pop r11


	mov r11, L1
	add r11, 1
	mov rdi, r11

	push r11
	call puts
	pop r11


	mov eax, 0
	leave
	ret

