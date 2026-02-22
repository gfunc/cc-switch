# CC Switch Web UI

This is the web-based UI for CC Switch, allowing you to manage AI API providers through a browser interface.

## Architecture

The web UI consists of two parts:

1. **Web Server** (`web-server/`): A Rust Axum-based HTTP server that exposes the same functionality as the Tauri desktop app
2. **Web Frontend** (`src/`): The React frontend adapted to work with the HTTP API instead of Tauri IPC

## Features

- All provider management features from the desktop app
- MCP server management
- Skills management
- Prompts management
- Real-time updates via WebSocket
- JWT-based authentication

## Quick Start

### Prerequisites

- Rust 1.85+
- Node.js 18+
- pnpm 8+

### Running the Web Server

```bash
cd web-server
cargo run
```

The server will start on `http://localhost:3000` by default.

Environment variables:

- `PORT`: Server port (default: 3000)
- `CC_SWITCH_DB_PATH`: Path to the SQLite database (default: `~/.cc-switch/cc-switch.db`)
- `JWT_SECRET`: Secret key for JWT tokens (default: `default-secret-key`)
- `CC_SWITCH_PASSWORD`: Password for web login (default: `admin`)

### Running the Web UI

```bash
# Install dependencies
pnpm install

# Start the web UI in development mode
pnpm dev:web
```

This will start the Vite dev server configured for web mode.

### Building for Production

```bash
# Build the web UI
pnpm build:web

# The static files will be in `dist/` and served by the web server
```

## API Documentation

### Authentication

All API endpoints (except `/api/v1/auth/login`) require authentication via Bearer token:

```
Authorization: Bearer <token>
```

### Endpoints

#### Auth

- `POST /api/v1/auth/login` - Login with username/password, returns JWT token

#### Providers

- `GET /api/v1/providers?app={app}` - List all providers for an app
- `GET /api/v1/providers/:id` - Get a specific provider
- `POST /api/v1/providers` - Create a new provider
- `PUT /api/v1/providers/:id` - Update a provider
- `DELETE /api/v1/providers/:id` - Delete a provider
- `POST /api/v1/providers/:id/switch?app={app}` - Switch to a provider
- `GET /api/v1/providers/current?app={app}` - Get current provider ID

#### Settings

- `GET /api/v1/settings` - Get settings
- `PUT /api/v1/settings` - Update settings

#### MCP

- `GET /api/v1/mcp` - List MCP servers
- `POST /api/v1/mcp` - Create MCP server
- `GET /api/v1/mcp/:id` - Get MCP server
- `PUT /api/v1/mcp/:id` - Update MCP server
- `DELETE /api/v1/mcp/:id` - Delete MCP server
- `POST /api/v1/mcp/:id/toggle` - Toggle MCP server

#### Prompts

- `GET /api/v1/prompts` - List prompts
- `POST /api/v1/prompts` - Create prompt
- `GET /api/v1/prompts/:id` - Get prompt
- `PUT /api/v1/prompts/:id` - Update prompt
- `DELETE /api/v1/prompts/:id` - Delete prompt
- `POST /api/v1/prompts/:id/activate` - Activate prompt

#### Skills

- `GET /api/v1/skills` - List installed skills
- `GET /api/v1/skills/discover` - Discover available skills
- `POST /api/v1/skills/:id/install` - Install a skill
- `DELETE /api/v1/skills/:id/uninstall` - Uninstall a skill

#### Sessions

- `GET /api/v1/sessions` - List sessions
- `GET /api/v1/sessions/:id` - Get session
- `POST /api/v1/sessions/:id/resume` - Resume session
- `DELETE /api/v1/sessions/:id` - Delete session

#### Proxy

- `GET /api/v1/proxy/status` - Get proxy status
- `POST /api/v1/proxy/start` - Start proxy
- `POST /api/v1/proxy/stop` - Stop proxy
- `POST /api/v1/proxy/restart` - Restart proxy

### WebSocket

Connect to `ws://localhost:3000/api/v1/ws` for real-time events.

Events:

- `provider.created` - Provider created
- `provider.updated` - Provider updated
- `provider.deleted` - Provider deleted
- `provider.switched` - Provider switched

## Development

### Project Structure

```
web-server/
├── src/
│   ├── main.rs              # Server entry point
│   ├── handlers/
│   │   └── ws.rs            # WebSocket handler
│   ├── middleware/
│   │   └── auth.rs          # JWT authentication
│   ├── models/
│   │   ├── app_state.rs     # App state with database
│   │   └── mod.rs           # Data models
│   └── routes/
│       ├── auth.rs          # Auth routes
│       ├── mcp.rs           # MCP routes
│       ├── prompts.rs       # Prompts routes
│       ├── providers.rs     # Providers routes
│       ├── proxy.rs         # Proxy routes
│       ├── sessions.rs      # Sessions routes
│       ├── settings.rs      # Settings routes
│       └── skills.rs        # Skills routes
├── Cargo.toml
└── README.md

src/lib/api/
├── web-client.ts            # HTTP client for web API
└── web/
    ├── index.ts             # Web API exports
    └── providers.ts         # Provider web API
```

## Security Considerations

1. **Authentication**: The web UI uses JWT tokens for authentication. Set a strong `JWT_SECRET` in production.

2. **Password**: Change the default `CC_SWITCH_PASSWORD` in production.

3. **HTTPS**: Use a reverse proxy (nginx, Caddy) with HTTPS in production.

4. **CORS**: The server allows all origins in development. Configure CORS properly for production.

5. **Database**: The web server accesses the same SQLite database as the desktop app. Ensure proper file permissions.

## License

MIT © Jason Young
