RUN_RELEASE = cargo run --release
SENTIMENT = cargo run --bin sentiment --release
CALCULATOR = cargo run --bin calculator --release
TOKEN_CHECKER = cargo run --bin token_checker -- BTC ETH
NAUTILUS = cargo run --bin nautilus_example --features nautilus --release
RAY_BALANCES = cargo run --bin raydium_cli --release -- balances $(OWNER)
SHUTTLE_RUN = shuttle run --secrets Secrets.toml
DEPLOY = shuttle deploy --secrets backend/Secrets.toml
FMT = cargo fmt --all
LINT = cargo clippy --all-targets --all-features -- -D warnings
CHECK = cargo check
TEST = cargo test

.PHONY: run sentiment calculator token-checker raydium-balances nautilus shuttle-run deploy fmt lint check test

run:
	$(RUN_RELEASE)

sentiment:
	$(SENTIMENT)

calculator:
	$(CALCULATOR)

token-checker:
	$(TOKEN_CHECKER)

raydium-balances:
	$(RAY_BALANCES)

nautilus:
	$(NAUTILUS)

shuttle-run:
	$(SHUTTLE_RUN)

deploy:
	$(DEPLOY)

fmt:
	$(FMT)

lint:
	$(LINT)

check:
	$(CHECK)

test:
	$(TEST)

