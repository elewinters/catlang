section .data
	L0: db `%d\n`, 0
	L1: db `h`, 0
	L2: db `ey`, 0
	L3: db `world!`, 0
	L4: db `hello`, 0
section .text

extern puts
extern exit
extern printf
extern putchar
extern strlen
extern strcmp
extern many_args
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

global print_many
print_many:
	push rbp
	mov rbp, rsp
	push rbx
	sub rsp, 72

	mov dword [rbp-12], edi
	mov dword [rbp-16], esi
	mov dword [rbp-20], edx
	mov dword [rbp-24], ecx
	mov dword [rbp-28], r8d
	mov dword [rbp-32], r9d
	mov rax, qword [rbp+16]
	mov qword [rbp-40], rax
	mov rax, qword [rbp+24]
	mov qword [rbp-48], rax
	mov eax, dword [rbp+32]
	mov dword [rbp-52], eax
	mov eax, dword [rbp+40]
	mov dword [rbp-56], eax
	mov al, byte [rbp+48]
	mov byte [rbp-57], al
	mov al, byte [rbp+56]
	mov byte [rbp-58], al
	mov rdi, [rbp-40]
	call puts

	mov rdi, [rbp-48]
	call puts

	mov esi, [rbp-56]
	mov edi, [rbp-52]
	call sum

	mov dword [rbp-62], eax
	mov esi, [rbp-62]
	mov rdi, L0
	call printf

	mov dil, [rbp-57]
	call putchar

	mov dil, [rbp-58]
	call putchar


	pop rbx
	leave
	ret

global main
main:
	push rbp
	mov rbp, rsp
	push rbx
	sub rsp, 24

	mov dword [rbp-12], edi
	mov al, 10
	push rax
	mov al, 65
	push rax
	mov eax, 5
	push rax
	mov rdi, L1
	call strlen

	mov ebx, eax
	mov rdi, L2
	call strlen

	add ebx, eax
	mov eax, ebx
	push rax
	mov rax, L3
	push rax
	mov rax, L4
	push rax
	mov r9d, 6
	mov r8d, 5
	mov ecx, 4
	mov edx, 3
	mov esi, 2
	mov edi, 1
	call print_many

	add rsp, 48
	mov eax, 0
	jmp .ret_main

.ret_main:
	pop rbx
	leave
	ret

