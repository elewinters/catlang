section .data
	L0: db `hi`, 0
	L1: db `hi`, 0
	L2: db `condition is true`, 0
	L3: db `condition is false`, 0
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
	sub rsp, 8

	mov dword [rbp-4], edi
	mov rdi, L0
	mov rsi, L1

	push r11
	call strcmp
	pop r11

	mov dword [rbp-8], eax

	mov edi, 0
	mov esi, 0

	push r11
	call sum
	pop r11

	mov eax, dword [rbp-8]
	cmp eax, eax
	jne .L0
	mov rdi, L2

	push r11
	call puts
	pop r11


	mov edi, 0

	push r11
	call exit
	pop r11


.L0:
	mov rdi, L3

	push r11
	call puts
	pop r11


	mov eax, 0
	leave
	ret

