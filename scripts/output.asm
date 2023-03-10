section .data
	L0: db `gonna return a number :D`, 0
	L1: db `Hello, world!`, 0
	L2: db `void...`, 0
	L3: db `ah`, 0
	L4: db `%ld\n`, 0
	L5: db `test`, 0
	L6: db `hi`, 0
	L7: db `lenght of 'test': %ld\n`, 0
	L8: db `does the syscall still work?\n`, 0
	L9: db `%s`, 0
section .text

extern puts
extern printf
extern strlen
extern strcpy
extern malloc
extern free
extern exit
extern system
extern scanf
global number
number:
	push rbp
	mov rbp, rsp

	mov rdi, L0
	call puts

	mov r11, 5
	add r11, 5
	mov rax, qword r11
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

	mov rdi, [rbp-8]
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
	mov rdi, [rbp-8]
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
	mov r11, [rbp-8]
	add r11, [rbp-16]
	mov rax, qword r11
	pop rbp
	ret

global square
square:
	push rbp
	mov rbp, rsp

	mov dword [rbp-4], edi
	mov r11d, [rbp-4]
	imul r11d, [rbp-4]
	mov eax, dword r11d
	pop rbp
	ret

global main
main:
	push rbp
	mov rbp, rsp
	sub rsp, 32

	mov edi, 1+1
	call void

	mov r11, 50
	add r11, L3
	add r11, 20
	imul r11, 2
	mov qword [rbp-8], r11

	xor rax, rax
	mov rdi, L4
	mov rsi, [rbp-8]
	call printf

	mov r11, L5
	add r11, 1
	mov rdi, r11
	call mystrlen
	mov r11, rax
	mov rdi, L6
	call mystrlen
	add r11, rax
	mov qword [rbp-16], r11

	xor rax, rax
	mov rdi, L7
	mov rsi, [rbp-16]
	call printf

	call hello_str
	mov qword [rbp-24], rax

	mov rdi, [rbp-24]
	call puts

	mov rdi, [rbp-24]
	call free

	mov rax, 1
	mov rdi, 1
	mov rsi, L8
	mov rdx, 29
	syscall

	mov r11, 128
	add r11, 5
	mov rdi, r11
	call malloc
	mov qword [rbp-32], rax

	mov rdi, L9
	mov rsi, [rbp-32]
	call scanf

	mov rdi, [rbp-32]
	call puts

	mov edi, 0
	call exit

	leave
	ret

