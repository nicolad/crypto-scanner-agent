# Makefile ─────────────────────────────────────────────────────────────────────
# 1. CONFIGURABLE KNOBS -------------------------------------------------------
SECRETS     ?= Secrets.toml             # override:  make run SECRETS=foo.toml
SHUTTLE_BIN ?= cargo shuttle            # override:  make shuttle-run SHUTTLE_BIN=shuttle
ENV         ?= dev                      # override:  make run ENV=prod

ifeq ($(ENV),prod)
LOG_LEVEL=info
else
LOG_LEVEL=debug
endif

# Helper: export TOML keys as env vars (requires `tomlq` or compatible `yq`)
define export_secrets
	@if command -v tomlq >/dev/null; then \
		echo "⏩  exporting secrets from $(SECRETS)"; \
		eval "$$(tomlq --raw-output 'to_entries|.[]|"\(.key)=\(.value)"' $(SECRETS) | sed 's/^/export /')"; \
	fi
endef

# 2. LOW-LEVEL COMMANDS -------------------------------------------------------
RUN_RELEASE       = env RUST_LOG=$(LOG_LEVEL) cargo run --release
SENTIMENT         = env RUST_LOG=$(LOG_LEVEL) cargo run --bin sentiment --release
CALCULATOR        = env RUST_LOG=$(LOG_LEVEL) cargo run --bin calculator --release
TOKEN_CHECKER     = env RUST_LOG=$(LOG_LEVEL) cargo run --bin token_checker -- BTC ETH
NAUTILUS          = env RUST_LOG=$(LOG_LEVEL) cargo run --bin nautilus_example --features nautilus --release
RAY_BALANCES      = env RUST_LOG=$(LOG_LEVEL) cargo run --bin raydium_cli --release -- balances $(OWNER)
RAY_TOP_COINS     = env RUST_LOG=$(LOG_LEVEL) cargo run --bin raydium_top_coins --release

SHUTTLE_RUN       = env RUST_LOG=$(LOG_LEVEL) $(SHUTTLE_BIN) run    --secrets $(SECRETS)
DEPLOY            = env RUST_LOG=$(LOG_LEVEL) $(SHUTTLE_BIN) deploy --secrets $(SECRETS)

FMT               = cargo fmt --all
LINT              = cargo clippy --all-targets --all-features -- -D warnings
CHECK             = cargo check
TEST              = cargo test

# 3. PUBLIC TARGETS -----------------------------------------------------------
.PHONY: run sentiment calculator token-checker \
        raydium-balances raydium-top-coins nautilus \
        shuttle-run deploy fmt lint check test

# 4. TARGET IMPLEMENTATIONS ---------------------------------------------------
run:
	$(call export_secrets)
	$(RUN_RELEASE)

sentiment:
	$(call export_secrets)
	$(SENTIMENT)

calculator:
	$(call export_secrets)
	$(CALCULATOR)

token-checker:
	$(call export_secrets)
	$(TOKEN_CHECKER)

raydium-balances:
	$(call export_secrets)
	$(RAY_BALANCES)

raydium-top-coins:
	$(call export_secrets)
	$(RAY_TOP_COINS)

nautilus:
	$(call export_secrets)
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
