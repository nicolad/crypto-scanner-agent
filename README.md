# crypto-scanner-agent

This project is a minimal Axum application that streams cryptocurrency gainers over WebSocket.
Incoming data is pulled from the Binance `!ticker@arr` feed and filtered server side before being broadcast
to any connected clients.  A small example client is served from the `static` directory.
