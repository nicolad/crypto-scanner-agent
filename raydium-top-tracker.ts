// ============================================================================
//  raydium-top-tracker.ts
//  Tiny helper-lib for finding the hottest Raydium pairs in real-time.
//  Vadim Nicolai – 2025-06-07
// ============================================================================

import axios, { AxiosInstance } from "axios";

// ---------------------------------------------------------------------------
// 1️⃣  Minimal run-time types for Raydium’s /v2/main/pairs endpoint.
//     (The real JSON has ~60 fields; we only declare what we use.)
export interface JsonPairItemInfo {
  ammId: string;            // Raydium AMM account
  lpMint: string;
  baseMint: string;
  quoteMint: string;
  baseSymbol: string;
  quoteSymbol: string;
  volume24h: number;        // ⚡  raw units (not USD)
  volume24hQuote: number;   // ⚡  quoted in quote token
  volume7d: number;
  volume7dQuote: number;
  apr24h: number;
  apr7d: number;
  price: number;            // pooled price in quote token
  liquidityUsd: number;     // TVL in USD (useful liquidity filter)
  updatedAt?: number;       // unix ms (when Raydium refreshed this row)
}

export type Comparator = (a: JsonPairItemInfo, b: JsonPairItemInfo) => number;

// ---------------------------------------------------------------------------
// 2️⃣  Library – handles polling, caching, filtering & sorting.

export interface TrackerOptions {
  /** How often to refresh (ms). Default 15 000 ms (≈ 1 Raydium block). */
  pollInterval?: number;
  /** Discard pairs with liquidity < x USD. Default $25 k. */
  minLiquidityUsd?: number;
  /** Custom ranking function. Default sort by volume24hQuote desc. */
  sort?: Comparator;
}

export class RaydiumTopTracker {
  private api: AxiosInstance;
  private opts: Required<TrackerOptions>;
  private pairs: JsonPairItemInfo[] = [];
  private timer?: NodeJS.Timeout;

  constructor(opts: TrackerOptions = {}) {
    this.opts = {
      pollInterval: opts.pollInterval ?? 15_000,
      minLiquidityUsd: opts.minLiquidityUsd ?? 25_000,
      sort:
        opts.sort ??
        ((a, b) => b.volume24hQuote - a.volume24hQuote), // default ranking
    };
    this.api = axios.create({
      baseURL: "https://api.raydium.io/v2",
      timeout: 15_000,
    });
  }

  /** Kick off polling. Returns an async iterator so you can `for await`. */
  async *run(): AsyncGenerator<JsonPairItemInfo[], void, void> {
    await this.refresh();   // prime cache synchronously
    yield this.pairs;
    this.timer = setInterval(async () => {
      try {
        await this.refresh();
      } catch (e) {
        console.warn("[RaydiumTopTracker] refresh failed:", e);
      }
    }, this.opts.pollInterval);

    try {
      // eslint-disable-next-line no-constant-condition
      while (true) {
        yield this.pairs;
        await new Promise((r) => setTimeout(r, this.opts.pollInterval));
      }
    } finally {
      clearInterval(this.timer);
    }
  }

  /** Synchronous getter – last computed ranking. */
  get topPairs(): JsonPairItemInfo[] {
    return this.pairs;
  }

  // -------------------------------------------------------------------------
  // 💡  Extend here: plug into TA libraries, Telegram alerts, etc.

  private async refresh(): Promise<void> {
    const { data } = await this.api.get<JsonPairItemInfo[]>("/main/pairs");
    // 1. Filter obvious dust & rugs
    const filtered = data.filter(
      (p) => (p.liquidityUsd ?? 0) >= this.opts.minLiquidityUsd
    );
    // 2. Rank with user comparator
    filtered.sort(this.opts.sort);
    this.pairs = filtered;
  }
}

// ---------------------------------------------------------------------------
// 3️⃣  Helper – quick CLI demo.
//     `npx ts-node raydium-top-tracker.ts` or build into your own tool.
if (require.main === module) {
  (async () => {
    const tracker = new RaydiumTopTracker({
      pollInterval: 60_000,          // update every minute
      minLiquidityUsd: 50_000,       // ignore illiquid stuff
      // Example: rank by (24 h volume ÷ liquidity) – a crude “turnover” metric
      sort: (a, b) =>
        b.volume24hQuote / b.liquidityUsd -
        a.volume24hQuote / a.liquidityUsd,
    });

    console.log("⏳  Fetching … (ctrl-c to quit)\n");
    for await (const list of tracker.run()) {
      console.clear();
      console.table(
        list.slice(0, 10).map((p) => ({
          pair: `${p.baseSymbol}/${p.quoteSymbol}`,
          vol24h: (p.volume24hQuote / 1e6).toFixed(2) + "M",
          apr24h: p.apr24h.toFixed(1) + " %",
          price: p.price.toPrecision(6),
          liqUsd: "$" + Math.round(p.liquidityUsd).toLocaleString(),
        }))
      );
    }
  })();
}

How it works & what to tweak

🔬  StepDetailWhy it matters for scalping

FetchA single GET /v2/main/pairs call delivers the full order-book snapshot once per poll. The feed already contains volume24h and volume24hQuote so you don’t have to crunch historic trades yourself.Fast, cheap, stateless; keeps your infra thin.
FilterminLiquidityUsd protects you from micro-caps where a few hundred USD swings distort the ranking. Tune per risk appetite.Avoiding “ghost” volume is crucial when you need to hit and exit quickly.
RankDefault is descending volume24hQuote, but you can inject any comparator: turnover, 7 d momentum, realized volatility, etc.Different scalping styles → different alpha signals.
StreamThe class exposes an async iterator, so your bot can for await and react the millisecond a new list is ready.No callback hell; no RxJS bloat.
DecoupleThe library itself has zero Solana or Web3 dependencies.  Pair it with Raydium SDK only where you actually submit swaps.Keeps cold-path code isolated from hot trading code, simplifies testing.


Production hardening (ideas you might not have asked for)

AreaSuggestion

Command-queuePut refresh() behind a simple mutex to survive network spikes without thrashing.
Back-pressureIf you stream to multiple consumer coroutines, hand them an immutable copy (structuredClone) to avoid race conditions.
Fallback RPCIf Raydium’s CDN glitches, stitch a temporary view from Solana logs (connection.getSignaturesForAddress + getParsedTransaction).  Overkill for most bots but eliminates single-point failure.
Risk / FundingAdd a quick check against Open Interest on Raydium Perps (endpoint /main/perps) before entering; high volume + high OI often signals a soon-to-mean-revert scalp.
LatencyRun your node in the same AZ as your Solana RPC (e.g., FTX‐Lands Frankfurt) to shave ~80 ms round-trip.


---

Usage example (ESM)

import { RaydiumTopTracker } from "./raydium-top-tracker.js";

const track = new RaydiumTopTracker({ pollInterval: 10_000 });

setInterval(() => {
  // Grab freshest list synchronously
  const [lead] = track.topPairs;
  console.log(
    `🔥  #1 right now: ${lead.baseSymbol}/${lead.quoteSymbol} – ` +
      `${(lead.volume24hQuote / 1e6).toFixed(2)} M quote vol / 24 h`
  );
}, 10_000);


---

Why trust the volume24h field?

Raydium’s indexer aggregates swap instructions from the on-chain AMM vault, so wash trading is less of an issue than on many CEX feeds. The same field is consumed by Raydium’s own analytics UI and their public SDK demo. 


---

Where to go next

Raydium Trade API – build the execution leg that actually fires your swaps.

@solana/web3.js – craft SIMD ‘priority fee’ TXs (Raydium exposes an endpoint that returns suggested micro-lamport fees).

Risk envelope – incorporate your PnL stop logic; Solana blocks close in ~400 ms, so your kill-switch must be local.


---

Raw resource links

https://api.raydium.io/v2/main/pairs

https://api.raydium.io/v2/main/price

https://docs.raydium.io​ (navigation → Developers → APIs)

https://github.com/raydium-io/raydium-sdk (archived but still the canonical type defs)


Feel free to fork & adapt ― and let me know if you need the execution side wired in. Happy scalping!
