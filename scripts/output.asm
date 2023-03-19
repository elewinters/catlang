section .data
	L0: db `not enough arguments`, 0
	L1: db `hi`, 0
	L2: db `hi`, 0
	L3: db `condition is true`, 0
	L4: db `condition is false`, 0
	L5: db `%d\n`, 0
	L6: db `exiting...`, 0
section .text

extern puts
extern exit
extern printf
extern putchar
extern strlen
extern strcmp
global myputchar
myputchar:
	push rbx
	push rbp
	mov rbp, rsp
	sub rsp, 1

	mov byte [rbp-1], dil
	mov dil, [rbp-1]
	call putchar

	mov dil, 10
	call putchar

	leave
	pop rbx
	ret

global abc
abc:
	push rbx
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
	pop rbx
	ret

global div
div:
	push rbx
	push rbp
	mov rbp, rsp

	mov dword [rbp-4], edi
	mov dword [rbp-8], esi
	mov eax, edi
	idiv esi
	pop rbp
	pop rbx
	ret

global sum
sum:
	push rbx
	push rbp
	mov rbp, rsp

	mov dword [rbp-4], edi
	mov dword [rbp-8], esi
	mov ebx, [rbp-4]
	add ebx, [rbp-8]
	mov eax, ebx
	pop rbp
	pop rbx
	ret

global sqr
sqr:
	push rbx
	push rbp
	mov rbp, rsp

	mov dword [rbp-4], edi
	mov ebx, [rbp-4]
	imul ebx, [rbp-4]
	mov eax, ebx
	pop rbp
	pop rbx
	ret

global main
main:
	push rbx
	push rbp
	mov rbp, rsp
	sub rsp, 12

	mov dword [rbp-4], edi
	mov eax, dword [rbp-4]
	cmp eax, 1
	jne .L0
	mov rdi, L0
	call puts

	mov edi, 1
	call exit

.L0:
	mov rdi, L1
	mov rsi, L2
	call strcmp

	mov dword [rbp-8], eax

	mov eax, dword [rbp-8]
	cmp eax, 0
	jne .L1
	mov rdi, L3
	call puts

.L1:
	mov eax, dword [rbp-8]
	cmp eax, 0
	je .L2
	mov rdi, L4
	call puts

.L2:
	mov edi, 5
	mov esi, 5
	call sum

	mov ebx, eax
	mov edi, 5
	mov esi, 5
	call sum

	add ebx, eax
	mov edi, 5
	mov esi, 5
	call sum

	add ebx, eax
	mov edi, 5
	mov esi, 5
	call sum

	add ebx, eax
	xor rax, rax
	cdq
	mov eax, dword ebx
	mov r11d, 2
	idiv r11d
	mov ebx, eax
	mov dword [rbp-12], ebx

	xor rax, rax
	mov rdi, L5
	mov esi, [rbp-12]
	call printf

	mov rdi, L6
	call puts

	mov eax, 0
	leave
	pop rbx
	ret

