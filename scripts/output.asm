section .data
	L0: db `\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`, 0
	L1: db `Hello, world!\n`, 0
	L2: db `yass girl\n`, 0
	L3: db `meow :3\n`, 0
	L4: db `scripts/hello.txt`, 0
	L5: db `hello world!`, 0
	L6: db `scripts/hello.txt`, 0
section .text

global print
print:
	push rbp
	mov rbp, rsp

	mov qword [rbp-8], rdi
	mov qword [rbp-16], rsi
	mov rax, 1
	mov rdi, 1
	mov rsi, [rbp-8]
	mov rdx, [rbp-16]
	syscall

	pop rbp
	ret

global exit
exit:
	push rbp
	mov rbp, rsp

	mov qword [rbp-8], rdi
	mov rax, 60
	mov rdi, [rbp-8]
	syscall

	pop rbp
	ret

global fwrite
fwrite:
	push rbp
	mov rbp, rsp

	mov qword [rbp-8], rdi
	mov qword [rbp-16], rsi
	mov qword [rbp-24], rdx
	mov rax, 2
	mov rdi, [rbp-8]
	mov rsi, 65
	mov rdx, 0644o
	syscall

	mov qword [rbp-32], 0
	mov qword [rbp-32], rax
	mov rax, 1
	mov rdi, [rbp-32]
	mov rsi, [rbp-16]
	mov rdx, [rbp-24]
	syscall

	mov rax, 3
	mov rdi, [rbp-32]
	syscall

	pop rbp
	ret

global fprint
fprint:
	push rbp
	mov rbp, rsp

	mov qword [rbp-8], rdi
	mov rax, 2
	mov rdi, [rbp-8]
	mov rsi, 0
	mov rdx, 0644o
	syscall

	mov qword [rbp-16], 0
	mov qword [rbp-16], rax
	mov rax, 0
	mov rdi, [rbp-16]
	mov rsi, L0
	mov rdx, 16
	syscall

	mov rdi, L0
	mov rsi, 16
	call print
	mov rax, 3
	mov rdi, [rbp-16]
	syscall

	pop rbp
	ret

global _start
_start:
	push rbp
	mov rbp, rsp

	mov rdi, L1
	mov rsi, 14
	call print

	mov rdi, L2
	mov rsi, 10
	call print

	mov rdi, L3
	mov rsi, 8
	call print

	mov rdi, L4
	mov rsi, L5
	mov rdx, 12
	call fwrite

	mov rdi, L6
	call fprint

	mov rdi, 0
	call exit

	pop rbp
	ret

