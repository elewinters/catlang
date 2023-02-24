NAME=catlang
OUT_DIR=~/bin

build:
	cargo build
	install target/debug/$(NAME) $(OUT_DIR)/$(NAME) 
run: build
	$(NAME) scripts/test.meow -V