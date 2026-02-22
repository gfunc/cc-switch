# CC Switch Web UI

A web-based interface for CC Switch that provides the same functionality as the desktop application but accessible through a browser.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Frontend (React + TS)                    │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐    │
│  │ Components  │  │    Hooks     │  │  TanStack Query  │    │
│  │   (UI)      │──│ (Bus. Logic) │──│   (Cache/Sync)   │    │
│  └─────────────┘  └──────────────┘  └──────────────────┘    │
│           │                                                 │
│           │ HTTP API / WebSocket                            │
└───────────┼─────────────────────────────────────────────────┘
            │
┌───────────▼─────────────────────────────────────────────────┐
│                  Web Server (Rust + Axum)                   │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────┐    │
│  │   HTTP API  │  │  WebSocket   │  │  Shared Services │    │
│  │  (RESTful)  │──│  (Events)    │──│  (from Tauri)    │    │
│  └─────────────┘  └──────────────┘  └──────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

## Features

- All provider management features
- MCP server management
- Skills management
- Prompts management
- Session management
- Settings management
- Real-time updates via WebSocket

## Getting Started

### Prerequisites

- Rust 1.85+
- Node.js 18+
- pnpm 8+

### Development

```bash
# Start the web server
cd web-server
cargo run

# Start the web UI (in another terminal)
pnpm dev:web
```

### Production Build

```bash
# Build web server
cd web-server
cargo build --release

# Build web UI
pnpm build:web
```

## API Documentation

See `web-server/API.md` for detailed API documentation.

## Authentication

The web UI uses token-based authentication. Set the `CC_SWITCH_TOKEN` environment variable to configure the access token.

## License

MIT © Jason Young
