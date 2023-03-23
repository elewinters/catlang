section .data
	L0: db `not enough arguments`, 0
	L1: db `hi`, 0
	L2: db `hi`, 0
	L3: db `condition is true`, 0
	L4: db `condition is false`, 0
	L5: db `%d\n`, 0
	L6: db `Hello, world!\n`, 0
	L7: db `exiting...`, 0
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

	pop rbx
	pop rbp
	ret

global main
main:
	push rbp
	mov rbp, rsp
	push rbx
	sub rsp, 40

	mov dword [rbp-12], edi
	mov eax, dword [rbp-12]
	cmp eax, 1
	jne .L1
	mov rdi, L0
	call puts

	mov edi, 1
	call exit

.L1:
	mov rdi, L1
	mov rsi, L2
	call strcmp

	mov dword [rbp-16], eax
	mov eax, dword [rbp-16]
	cmp eax, 0
	jne .L2
	mov rdi, L3
	call puts

.L2:
	mov eax, dword [rbp-16]
	cmp eax, 0
	je .L3
	mov rdi, L4
	call puts

.L3:
	mov ebx, 5
	imul ebx, 5
	imul ebx, 5

	cdq
	mov r11d, 2
	mov eax, ebx
	idiv r11d
	mov ebx, eax

	add ebx, 10
	imul ebx, 2

	cdq
	mov r11d, 9
	mov eax, ebx
	idiv r11d
	mov ebx, eax

	mov dword [rbp-20], ebx
	xor rax, rax
	mov rdi, L5
	mov esi, [rbp-20]
	call printf

	mov qword [rbp-28], 1
	mov rbx, [rbp-28]
	sub rbx, 1
	add rbx, 1
	mov rax, rbx
	mov rdi, [rbp-28]
	mov rsi, L6
	mov rbx, 10
	add rbx, 4
	mov rdx, rbx
	syscall

	mov rdi, L7
	call puts

	mov eax, 0

	pop rbx
	leave
	ret

