RFLAGS="-C link-arg=-s"

# main contracts

build: account-bindings

account-bindings: contract
	$(call compile_release,account-bindings)
	@mkdir -p res
	cp target/wasm32-unknown-unknown/release/account_bindings.wasm ./res

dev-deploy: build
	near dev-deploy --wasmFile ./res/account_bindings.wasm

clean:
	rm res/*.wasm
	rm tests/compiled-contracts/*.wasm

lint:
	cargo fmt -- --check
	cargo clippy --tests -- -D clippy::all

define compile_release
	@rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p $(1) --target wasm32-unknown-unknown --release
endef

define compile_test
	@rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p $(1) --target wasm32-unknown-unknown --features "test"
	@mkdir -p res
endef

test: test-unit test-integration

test-unit:
	cargo test 

test-integration: test-account-bindings

monkey-patch:
	cp ./tests/web.js node_modules/near-workspaces/node_modules/near-api-js/lib/utils/

TEST_FILE ?= **
LOGS ?=
TEST_CONCURRENCY ?= 4

test-account-bindings: monkey-patch build
	@mkdir -p ./tests/compiled-contracts/
	@cp ./res/account_bindings.wasm ./tests/compiled-contracts/
	NEAR_PRINT_LOGS=$(LOGS) npx ava --timeout=5m tests/__tests__/$(TEST_FILE).ava.ts --verbose
