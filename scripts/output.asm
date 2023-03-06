section .data
	L0: db `gonna return a number :D`, 0
	L1: db `Hello, world!`, 0
	L2: db `void...`, 0
	L3: db `%ld\n`, 0
	L4: db `test`, 0
	L5: db `lenght of 'test': %ld\n`, 0
section .text

extern puts
extern printf
extern strlen
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

	mov rax, qword 5+5
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

global mystrlen
mystrlen:
	push rbp
	mov rbp, rsp
	sub rsp, 8

	mov qword [rbp-8], rdi
	mov rdi, qword [rbp-8]
	call strlen
	leave
	ret

global void
void:
	push rbp
	mov rbp, rsp
	sub rsp, 4

	mov dword [rbp-4], edi
	mov rdi, L2
	call puts

	leave
	ret

global sum
sum:
	push rbp
	mov rbp, rsp

	mov qword [rbp-8], rdi
	mov qword [rbp-16], rsi
	mov rax, [rdi+rsi]
	pop rbp
	ret

global main
main:
	push rbp
	mov rbp, rsp
	sub rsp, 24

	mov edi, 1
	call void

	mov rdi, 50
	mov rsi, 50
	call sum
	mov qword [rbp-8], rax

	xor rax, rax
	mov rdi, L3
	mov rsi, qword [rbp-8]
	call printf

	mov rdi, L4
	call mystrlen
	mov qword [rbp-16], rax

	xor rax, rax
	mov rdi, L5
	mov rsi, qword [rbp-16]
	call printf

	call hello_str
	mov qword [rbp-24], rax

	mov rdi, qword [rbp-24]
	call puts

	mov edi, 0
	call exit

	leave
	ret

