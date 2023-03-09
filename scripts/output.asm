section .data
	L0: db `%d\n`, 0
section .text

extern printf
extern exit
extern strlen
global sum
sum:
	push rbp
	mov rbp, rsp

	mov dword [rbp-4], edi
	mov dword [rbp-8], esi
	mov edx, [rbp-4]
	add edx, [rbp-8]
	mov eax, dword edx
	pop rbp
	ret

global main
main:
	push rbp
	mov rbp, rsp
	sub rsp, 20

	mov rdx, 5
	mov qword [rbp-8], rdx

	mov rdx, 10
	mov qword [rbp-16], rdx

	mov edi, 5
	mov esi, 5
	call sum
	mov edx, eax
	mov dword [rbp-20], edx

	xor rax, rax
	mov rdi, L0
	mov esi, dword [rbp-20]
	call printf

	mov edx, 0
	mov eax, dword edx
	leave
	ret

