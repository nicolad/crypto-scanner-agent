# TODO

This list tracks upcoming tasks to enhance the crypto-scanner agent with scalping-oriented features. Each item adapts the original playbook to this project's focus on streaming cryptocurrency gainers.

## 1. Gainers Discovery
1. Enforce a 24h volume filter of at least $1M in the signal extractor.
2. Check real-time volume before broadcasting any signal.
3. Require both a 24h gain >= 5% and a 1h gain >= 1% when filtering pairs.
4. Ignore pairs listed for less than 30 days.
5. Skip tokens with low liquidity (e.g., less than 5% book depth in top 3 bids).
6. Exclude pairs with spreads above 0.05%.
7. Filter out meme coins to reduce unstable signals.
8. Prioritize top-cap coins like BTC and ETH for reliable liquidity.
9. Limit exposure to very recently listed coins to avoid pump-and-dumps.
10. Drop coins showing negative on-chain activity that might hint at manipulation.
11. Avoid processing flash pumps over 5% in under 10s without volume confirmation.
12. Include funding rates as a secondary indicator and avoid extremes.
13. Skip coins with re-entrancy vulnerabilities.
14. Add a top 5 gainers filter and focus on the one with the best volume.
15. Prefer coins trending upward rather than ranging sideways.

## 2. Risk Management
16. Set risk per trade in downstream strategies to 0.2% of account size.
17. Cap simultaneous open positions at 3 when trading using these signals.
18. Place stop losses at least 0.2% below entry or half the spread.
19. Close a position if the coin moves against you by 0.25%.
20. Force an exit on a 0.5% loss.
21. Enable trailing stops only once a trade is up 0.3%.
22. Add a time stop of 60 seconds per position for scalping.
23. Close trades at the first sign of high volume drop or reversal.
24. Avoid pyramiding winners; keep sizes conservative.
25. Protect account balance with a max daily drawdown (e.g., 5%).
26. Track win/loss ratio and adjust risk as needed.
27. Exit trades when funding rates turn extreme or change direction.
28. Close trades based on market conditions rather than arbitrary targets.
29. Backtest to find the best risk/reward thresholds.
30. Reduce trade size when volatility spikes around major news.

## 3. Entry Logic
31. Enter on breakout after the price makes a new high within the current 30s bar.
32. Wait for a pullback confirmation before entering.
33. Confirm breakouts using volume spikes (5x-10x normal volume).
34. Use VWAP to refine entry timing.
35. Enter after a bounce from the 10-period moving average.
36. Trade only during low spread periods.
37. Favor limit orders for entry; use market orders only for extreme breakouts.
38. Place limit orders slightly inside the spread.
39. Require price action confirmation (e.g., candlestick patterns) before entry.
40. Watch depth shrinking as a sign of imminent movement.
41. Beware of fake breakouts and validate with volume.
42. Skip entries during major news unless momentum is already present.
43. Avoid chasing moves; only enter if momentum persists.
44. Use multi-timeframe analysis (5m and 15m charts) for context.
45. Ensure sufficient liquidity (5x-10x position size in book depth) before entry.
46. Enter only on new highs during bullish trends; do not fade tops.
47. Confirm key support or resistance breaks with candles before entering.
48. Check momentum indicators (RSI/ADX) for strength before taking a position.

## 4. Exit Logic
49. Use trailing stops once price moves more than 0.3% in your favor.
50. Exit on weakening momentum, such as RSI divergence.
51. Close if a reversal candle appears.
52. Take profit near the upper Bollinger Band or similar indicator.
53. Exit after 60 seconds regardless of gain or loss.
54. Scale out half the position when up by 0.4%.
55. Exit all positions if the overall market turns sharply.
56. Close immediately on a wick rejection against the trade.
57. Adjust stops if strong momentum continues in your favor.
58. Exit if funding flips negative.
59. Use aggressive exits when the market reverses within the time stop.
60. Set automated profit-taking limits for scaling out.
61. Watch the 1-minute chart for quick reversals after entry.

## 5. Trading Strategy
62. Combine momentum indicators like RSI and MACD when generating signals.
63. Use multi-timeframe confirmation for higher probability setups.
64. Size orders according to the liquidity of each coin.
65. Hedge altcoin exposure with BTC or stablecoins when appropriate.
66. Focus on long-only strategies during bull markets.
67. Maintain a daily list of 24h high gainers.
68. Build a dynamic scanner that checks both volatility and volume criteria.
69. Backtest strategies thoroughly before live deployment.
70. Configure alerts for potential breakouts.

## 6. Additional Risk Controls
71. Place hard stop losses 0.5% below entry.
72. Always use take-profit orders on high-volume coins.
73. Avoid market orders except in emergencies; prefer limit orders.
74. Limit trading frequency to 5–10 trades per hour to reduce overtrading.
75. Pause trading after three consecutive losses.
76. Review daily PnL and adjust sizing if needed.
77. Diversify positions across multiple coins.
78. Use position sizing tools that account for volatility.
79. Never risk more than 2% of total equity on one trade.
80. Do not increase size after losses.
81. Maintain at least a 1:2 risk/reward ratio per trade.
82. Minimize slippage on both entry and exit.
83. Avoid holding scalp positions too long.
84. Enforce a cooldown period after losing sessions.
85. Steer clear of trades during major economic releases.
86. Automate periodic stop-loss placement for longer holds.
87. Rebalance the portfolio weekly to avoid heavy exposure.
88. Scale in and out using small incremental orders.

## 7. Infrastructure and Optimization
89. Deploy on a low-latency server close to the exchange.
90. Use async tasks to handle multiple pairs concurrently.
91. Optimize code paths to reduce latency.
92. Manage memory carefully to avoid GC delays.
93. Cache computations and network calls when possible.
94. Optionally place Nginx in front to proxy WebSockets.
95. Keep WebSocket connections minimal—one per exchange.
96. Host on a VPS with less than 20ms latency to Binance.
97. Use message queues like Kafka for logging events.
98. Provide real-time alerts when a coin hits trade triggers.
99. Batch database writes for efficiency.
100. Compress WebSocket messages with gzip.

## 8. Back-testing and Analysis
101. Back-test with historical tick data for realism.
102. Experiment with different strategies and time frames.
103. Simulate slippage by removing order book depth in tests.
104. Compare performance against simple strategies like buy and hold.
105. Use Monte Carlo simulations to model random outcomes.
106. Evaluate entry and exit filters for their impact.
107. Track drawdowns to gauge worst-case scenarios.
108. Add real-time metrics for PnL and Sharpe ratio.
109. Analyze failed trades to identify recurring issues.

## 9. Automation and Deployment
110. Use Shuttle to deploy the service securely.
111. Containerize the bot with Docker for consistent runs.
112. Ensure automatic restarts on failure.
113. Set up CI/CD for seamless updates.
114. Automate regular strategy performance reviews.
115. Store API keys using Shuttle secrets.
116. Add thorough logging for debugging.
117. Use environment variables to switch between dev and prod.
118. Trigger alerts on any connection loss or errors.
119. Back up logs periodically for audits.
120. Apply custom timeouts based on volatility spikes.

## 10. Advanced Tips
121. Experiment with sentiment analysis to predict gains.
122. Incorporate on-chain metrics like active addresses and gas fees.
123. Monitor social media for price-moving news.
124. Research machine learning models for high-probability gainers.
125. Allow multiple strategies (scalping, swing, trend following).
126. Consider market-making when liquidity is high but volatility is low.
127. Detect liquidation cascades and react in real time.
128. Explore grid trading for consistent profits.
129. Adjust risk models dynamically as volatility changes.
130. Test multi-exchange arbitrage during divergence.
131. Use low-latency data sources, possibly co-located with the exchange.
132. Experiment with ML models to predict short-term volatility.
133. Combine technical and fundamental data with ensemble models.
134. Hedge against large market drawdowns when appropriate.
