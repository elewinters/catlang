NAME=catlang
OUT_DIR=~/bin

build:
	cargo build
	install target/debug/$(NAME) $(OUT_DIR)/$(NAME) 
run: build
	$(NAME) scripts/code.cat -o scripts/output.asm -V

runb: build
	$(NAME) scripts/code.cat -V -b -o scripts/output && scripts/output

runc: build
	$(NAME) scripts/code.cat -V -lc -b -o scripts/output && scripts/output