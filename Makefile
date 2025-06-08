# Makefile ─────────────────────────────────────────────────────────────────────
# 1. CONFIGURABLE KNOBS -------------------------------------------------------
SECRETS     ?= Secrets.toml             # override e.g.  make run SECRETS=foo.toml
SHUTTLE_BIN ?= cargo shuttle            # or simply `shuttle` if you prefer

# 2. SHELL SETUP --------------------------------------------------------------
.ONESHELL:                              # each recipe runs in a *single* shell
SHELL := /bin/bash                      # predictable Bashism support

# 3. SECRET HELPER ------------------------------------------------------------
# Export every K = V pair from $(SECRETS) (quotes optional, ignores comments)
define export_secrets
	@if [ -f "$(SECRETS)" ]; then \
		echo "⏩  exporting secrets from $(SECRETS)"; \
		grep -E '^[[:space:]]*[A-Za-z0-9_.-]+[[:space:]]*=' "$(SECRETS)" \
		| sed -E 's/^[[:space:]]*//;s/[[:space:]]*=[[:space:]]*/=/' \
		| while IFS='=' read -r k v; do \
				v="$${v%\"}"; v="$${v#\"}"; v="$${v%\'}"; v="$${v#\'}"; \
				export "$$k=$$v"; \
		  done; \
	else \
		echo "⚠️  $(SECRETS) not found – no secrets exported"; \
	fi
endef

# Helper: OWNER value straight from the secrets file (no runtime deps)
OWNER := $(shell awk -F= '/^[[:space:]]*owner[[:space:]]*=/{gsub(/["'\'']/,"");gsub(/^[[:space:]]+|[[:space:]]+$$/,"",$$2);print $$2;exit}' $(SECRETS))

# 4. LOW-LEVEL COMMANDS -------------------------------------------------------
LOCAL_RUN         = cargo run --release
SENTIMENT         = cargo run --bin sentiment --release
CALCULATOR        = cargo run --bin calculator --release
TOKEN_CHECKER     = cargo run --bin token_checker -- BTC ETH
NAUTILUS          = cargo run --bin nautilus_example --features nautilus --release
RAY_BALANCES      = cargo run --bin raydium_cli --release -- balances $(OWNER)
RAY_TOP_COINS     = cargo run --bin raydium_top_coins --release

SHUTTLE_RUN       = $(SHUTTLE_BIN) run --secrets $(SECRETS)
DEPLOY            = $(SHUTTLE_BIN) deploy --secrets $(SECRETS)

FMT               = cargo fmt --all
LINT              = cargo clippy --all-targets --all-features -- -D warnings
CHECK             = cargo check
TEST              = cargo test

# 5. PUBLIC TARGETS -----------------------------------------------------------
.PHONY: run local-run sentiment calculator token-checker \
        raydium-balances raydium-top-coins nautilus \
        shuttle-run deploy fmt lint check test

# 6. TARGET IMPLEMENTATIONS ---------------------------------------------------
# Preferred: run through Shuttle so secrets are also available to the service
run: shuttle-run

# Optional direct cargo run (still exports secrets first)
local-run:
	$(call export_secrets)
	$(LOCAL_RUN)

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
	@if [ -z "$(OWNER)" ]; then \
		echo "❌  OWNER not found in $(SECRETS)"; exit 1; \
	fi
	$(RAY_BALANCES)

raydium-top-coins:
	$(call export_secrets)
	$(RAY_TOP_COINS)

nautilus:
	$(call export_secrets)
	$(NAUTILUS)

# Explicit Shuttle wrappers ---------------------------------------------------
shuttle-run:
	$(SHUTTLE_RUN)

deploy:
	$(DEPLOY)

# Code-quality helpers --------------------------------------------------------
fmt:   ; $(FMT)
lint:  ; $(LINT)
check: ; $(CHECK)
test:  ; $(TEST)
