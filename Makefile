# Makefile ─────────────────────────────────────────────────────────────────────
# 0. Prerequisites
#    • python-dotenv CLI:  brew install python-dotenv      # already on your Mac
#    • .env in project root with KEY=value lines (e.g. OWNER=abc123)
#
# 1. CONFIGURABLE KNOBS -------------------------------------------------------
ENV_FILE   ?= .env                    # override: make ENV_FILE=prod.env run
DOTENV_BIN ?= $(shell command -v dotenv)
SHUTTLE_BIN?= cargo shuttle

ifeq ($(DOTENV_BIN),)
$(error 🚫  'dotenv' CLI not found. Install with: brew install python-dotenv)
endif

# Single macro that injects secrets for any command
DO = $(DOTENV_BIN) -f $(ENV_FILE) run --   # ← NOTE the “run --”

# 2. LOW-LEVEL COMMANDS -------------------------------------------------------
CARGO           = cargo
LOCAL_RUN       = $(CARGO) run --release
SENTIMENT       = $(CARGO) run --bin sentiment --release
CALCULATOR      = $(CARGO) run --bin calculator --release
TOKEN_CHECKER   = $(CARGO) run --bin token_checker -- BTC ETH
NAUTILUS        = $(CARGO) run --bin nautilus_example --features nautilus --release
RAY_BALANCES    = $(CARGO) run --bin raydium_cli --release -- balances "$${OWNER}"
RAY_TOP_COINS   = $(CARGO) run --bin raydium_top_coins --release

SHUTTLE_RUN     = $(SHUTTLE_BIN) run    --secrets $(ENV_FILE)
DEPLOY          = $(SHUTTLE_BIN) deploy --secrets $(ENV_FILE)

FMT             = $(CARGO) fmt --all
LINT            = $(CARGO) clippy --all-targets --all-features -- -D warnings
CHECK           = $(CARGO) check
TEST            = $(CARGO) test

# 3. PUBLIC TARGETS -----------------------------------------------------------
.PHONY: run local-run sentiment calculator token-checker \
        raydium-balances raydium-top-coins nautilus \
        shuttle-run deploy fmt lint check test

# Preferred entrypoint: run via Shuttle so secrets are always present
run: shuttle-run

# ---- Service targets --------------------------------------------------------
local-run:
	$(DO) $(LOCAL_RUN)

sentiment:
	$(DO) $(SENTIMENT)

calculator:
	$(DO) $(CALCULATOR)

token-checker:
	$(DO) $(TOKEN_CHECKER)

raydium-balances:
	$(DO) $(RAY_BALANCES)

raydium-top-coins:
	$(DO) $(RAY_TOP_COINS)

nautilus:
	$(DO) $(NAUTILUS)

# ---- Shuttle wrappers -------------------------------------------------------
shuttle-run:
	$(DO) $(SHUTTLE_RUN)

deploy:
	$(DO) $(DEPLOY)

# ---- Code-quality helpers (no secrets needed) -------------------------------
fmt:
	$(FMT)

lint:
	$(LINT)

check:
	$(CHECK)

test:
	$(TEST)
