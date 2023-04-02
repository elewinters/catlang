section .data
	L0: db `hi`, 0
	L1: db `hi`, 0
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
	sub rsp, 24

	mov byte [rbp-9], dil
	mov dil, [rbp-9]
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
	push rbx
	sub rsp, 8

	mov dil, 65
	call putchar

	mov dil, 66
	call putchar

	mov dil, 67
	call putchar

	mov dil, 10
	call putchar


	pop rbx
	leave
	ret

global div
div:
	push rbp
	mov rbp, rsp
	push rbx

	mov dword [rbp-12], edi
	mov dword [rbp-16], esi
	mov ebx, [rbp-12]

	cdq
	mov r11d, [rbp-16]
	mov eax, ebx
	idiv r11d
	mov ebx, eax

	mov eax, ebx
	jmp .ret_div

.ret_div:
	pop rbx
	pop rbp
	ret

global sum
sum:
	push rbp
	mov rbp, rsp
	push rbx

	mov dword [rbp-12], edi
	mov dword [rbp-16], esi
	mov ebx, [rbp-12]
	add ebx, [rbp-16]
	mov eax, ebx
	jmp .ret_sum

.ret_sum:
	pop rbx
	pop rbp
	ret

global sqr
sqr:
	push rbp
	mov rbp, rsp
	push rbx

	mov dword [rbp-12], edi
	mov ebx, [rbp-12]
	imul ebx, [rbp-12]
	mov eax, ebx
	jmp .ret_sqr

.ret_sqr:
	pop rbx
	pop rbp
	ret

global main
main:
	push rbp
	mov rbp, rsp
	push rbx
	sub rsp, 24

	mov dword [rbp-12], edi
	mov rdi, L0
	mov rsi, L1
	call strcmp

	mov dword [rbp-16], eax
	mov eax, dword [rbp-16]
	cmp eax, 0
	jne .L1
	mov eax, 0
	jmp .ret_main
.L1:
	mov eax, dword [rbp-16]
	cmp eax, 0
	je .L2
	mov eax, 1
	jmp .ret_main
.L2:
	mov eax, 0
	jmp .ret_main

.ret_main:
	pop rbx
	leave
	ret

