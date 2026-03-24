NanoClaw WebSocket Protocol

  Connection

  ws://localhost:{port}?token={WEB_CHANNEL_TOKEN}

  Authentication is required. Two methods supported:

- Query parameter: ?token=<token>
- Header: Authorization: Bearer <token>

  Failure returns WebSocket close code 4001 with reason "Unauthorized".

  On successful connection the server immediately sends a connected message.

  ---
  Client → Server Messages

  All messages are JSON with a type field.

  message — Send a chat message

  {
    "type": "message",
    "content": "Hello, can you help me launch a token?",
    "sender_name": "Alice"
  }
  ┌─────────────┬───────────┬──────────┬──────────────────────────────────┐
  │    Field    │   Type    │ Required │              Notes               │
  ├─────────────┼───────────┼──────────┼──────────────────────────────────┤
  │ type        │ "message" │ yes      │                                  │
  ├─────────────┼───────────┼──────────┼──────────────────────────────────┤
  │ content     │ string    │ yes      │ Must be non-empty after trimming │
  ├─────────────┼───────────┼──────────┼──────────────────────────────────┤
  │ sender_name │ string    │ no       │ Defaults to "Web User"           │
  └─────────────┴───────────┴──────────┴──────────────────────────────────┘
  wallet_connect — Attach a Solana wallet

  {
    "type": "wallet_connect",
    "address": "BJjUoux3xacYcRZV31Ytsi4haJb3HgyzmweVDHutiLWU"
  }
  ┌─────────┬──────────────────┬──────────┬──────────────────────────────────┐
  │  Field  │       Type       │ Required │              Notes               │
  ├─────────┼──────────────────┼──────────┼──────────────────────────────────┤
  │ type    │ "wallet_connect" │ yes      │                                  │
  ├─────────┼──────────────────┼──────────┼──────────────────────────────────┤
  │ address │ string           │ yes      │ Must be non-empty after trimming │
  └─────────┴──────────────────┴──────────┴──────────────────────────────────┘
  wallet_disconnect — Detach wallet

{
    "type": "wallet_disconnect"
  }
  ┌───────┬─────────────────────┬──────────┬───────┐
  │ Field │        Type         │ Required │ Notes │
  ├───────┼─────────────────────┼──────────┼───────┤
  │ type  │ "wallet_disconnect" │ yes      │       │
  └───────┴─────────────────────┴──────────┴───────┘
  ---

  Server → Client Messages

  All messages are JSON with a type field. Messages are broadcast to all connected clients unless noted.

  connected — Connection acknowledged

  Sent only to the connecting client (not broadcast) immediately after the WebSocket handshake succeeds.

  {
    "type": "connected",
    "jid": "web:default"
  }

  message — Agent response

  {
    "type": "message",
    "content": "Here's how to launch a token...",
    "sender": "NanoClaw"
  }
  ┌─────────┬────────┬──────────────────────────────────────┐
  │  Field  │  Type  │                Notes                 │
  ├─────────┼────────┼──────────────────────────────────────┤
  │ content │ string │ The agent's response text (markdown) │
  ├─────────┼────────┼──────────────────────────────────────┤
  │ sender  │ string │ The assistant's configured name      │
  └─────────┴────────┴──────────────────────────────────────┘
  typing — Agent is thinking

  {
    "type": "typing",
    "isTyping": true
  }
  ┌──────────┬─────────┬────────────────────────────────────────────────────────┐
  │  Field   │  Type   │                         Notes                          │
  ├──────────┼─────────┼────────────────────────────────────────────────────────┤
  │ isTyping │ boolean │ true when the agent starts processing, false when done │
  └──────────┴─────────┴────────────────────────────────────────────────────────┘
  transaction — Transaction for wallet signing

  Sent when the agent (via genesis MCP) needs the user to sign a Solana transaction.

  {
    "type": "transaction",
    "transaction": "<base64-encoded serialized transaction>",
    "message": "Sign to launch your token"
  }
  ┌─────────────┬─────────┬──────────────────────────────────────────────┐
  │    Field    │  Type   │                    Notes                     │
  ├─────────────┼─────────┼──────────────────────────────────────────────┤
  │ transaction │ string  │ Base64-encoded serialized Solana transaction │
  ├─────────────┼─────────┼──────────────────────────────────────────────┤
  │ message     │ string? │ Optional human-readable description          │
  └─────────────┴─────────┴──────────────────────────────────────────────┘
  wallet_connected — Wallet address acknowledged

  Broadcast to all clients after a successful wallet_connect.

  {
    "type": "wallet_connected",
    "address": "BJjUoux3xacYcRZV31Ytsi4haJb3HgyzmweVDHutiLWU"
  }

  wallet_disconnected — Wallet detached

  Broadcast to all clients after a wallet_disconnect.

  {
    "type": "wallet_disconnected"
  }

  error — Error response

  Sent only to the client that caused the error (not broadcast).

  {
    "type": "error",
    "error": "Description of what went wrong"
  }

  Known error strings:

- "Invalid JSON" — message wasn't parseable JSON
- "wallet_connect requires a non-empty address string" — missing/empty address
- "Expected { type: \"message\", content: \"...\" }" — missing content field
- "Unknown message type: <type>" — unrecognized type field

  ---
  Typical Flow

  Client                          Server
    |                                |
    |--- ws connect ?token=xxx ----->|
    |<---- { type: "connected" } ----|
    |                                |
    |--- { type: "wallet_connect",   |
    |      address: "BJ..." } ----->|
    |<-- { type: "wallet_connected", |
    |      address: "BJ..." } ------|
    |                                |
    |--- { type: "message",         |
    |      content: "launch a       |
    |      token called FOO" } ---->|
    |<-- { type: "typing",          |
    |      isTyping: true } --------|
    |                                |  (agent processes...)
    |<-- { type: "transaction",      |
    |      transaction: "base64...", |
    |      message: "Sign to        |
    |      launch FOO" } -----------|
    |                                |
    |  (user signs in wallet UI)     |
    |                                |
    |<-- { type: "message",         |
    |      content: "Token launched!"|
    |      sender: "NanoClaw" } ----|
    |<-- { type: "typing",          |
    |      isTyping: false } -------|

  ---
  Notes

- The default JID is web:default — the server only services this single chat room.
- Wallet state is global per server instance, not per-client. If one client connects a wallet, it applies to all
  clients.
- The sender field in server message payloads reflects the configured ASSISTANT_NAME (default: "NanoClaw").
- The server port defaults to 3002 and is configurable via WEB_CHANNEL_PORT.
