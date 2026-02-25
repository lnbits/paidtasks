all: format check

format: prettier

check: checkprettier

build-wasm:
	cd wasm/rust-example && \
		cargo build --release --target wasm32-unknown-unknown
	cp wasm/rust-example/target/wasm32-unknown-unknown/release/paidtasks.wasm wasm/module.wasm

prettier:
	npx prettier --write .

checkprettier:
	npx prettier --check .
pyright:
	@echo "skipping pyright for wasm extension"

mypy:
	@echo "skipping mypy for wasm extension"

black:
	@echo "skipping black for wasm extension"

ruff:
	@echo "skipping ruff for wasm extension"

checkruff:
	@echo "skipping ruff check for wasm extension"

checkblack:
	@echo "skipping black check for wasm extension"

checkeditorconfig:
	@echo "skipping editorconfig check for wasm extension"

test:
	@echo "no tests configured for wasm extension"

checkbundle:
	@echo "skipping checkbundle"
