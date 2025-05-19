# Initial Task List

This directory contains high-level tasks selected from [todo.md](../todo.md). These are simple enhancements that can be implemented early.

1. **Enforce 24h volume filter** - Ensure signals only include pairs with at least `$1M` in 24h volume.
2. **Check real-time volume** - Verify current volume before broadcasting any signal.
3. **24h and 1h gain checks** - Require `>= 5%` gain over 24h and `>= 1%` gain over 1h when filtering pairs.
4. **Ignore new pairs** - Skip pairs that have been listed for less than `30` days.
5. **Skip low liquidity tokens** - Exclude tokens with less than `5%` book depth in the top three bids.
