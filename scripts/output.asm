section .data
	L0: db `gonna return a number :D`, 0
	L1: db `Hello, world!`, 0
	L2: db `%d\n`, 0
section .text

extern puts
extern printf
extern strcpy
extern malloc
extern free
extern exit
extern system
global number
number:
	push rbp
	mov rbp, rsp

	mov rdi, L0
	call puts

	mov eax, dword 5+5
	pop rbp
	ret

global hello_str
hello_str:
	push rbp
	mov rbp, rsp
	sub rsp, 8

	mov rdi, 15
	call malloc
	mov qword [rbp-8], rax

	mov rdi, qword [rbp-8]
	mov rsi, L1
	call strcpy

	mov rax, qword [rbp-8]
	leave
	ret

global main
main:
	push rbp
	mov rbp, rsp
	sub rsp, 12

	call number
	mov dword [rbp-4], eax

	xor rax, rax
	mov rdi, L2
	mov esi, dword [rbp-4]
	call printf

	call hello_str
	mov qword [rbp-12], rax

	mov rdi, qword [rbp-12]
	call puts

	mov edi, 0
	call exit

	leave
	ret

