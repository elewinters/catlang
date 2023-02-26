NAME = catlang
OUT_DIR = ~/bin
PROFILE ?= "debug"

.SILENT: build

build:
	echo '-----------------------------------'
	echo "   building with $(PROFILE) profile"
	echo '-----------------------------------'

	if [ $(PROFILE) = "debug" ]; then \
		cargo build; \
		install target/debug/$(NAME) $(OUT_DIR)/$(NAME); \
	else \
		cargo build --release; \
		install target/release/$(NAME) $(OUT_DIR)/$(NAME); \
	fi
run: build
	$(OUT_DIR)/$(NAME) scripts/code.cat -o scripts/output.asm -V

runb: build
	$(OUT_DIR)/$(NAME) scripts/code.cat -V -b -o scripts/output && scripts/output

runc: build
	$(OUT_DIR)/$(NAME) scripts/code.cat -V -lc -b -o scripts/output && scripts/output