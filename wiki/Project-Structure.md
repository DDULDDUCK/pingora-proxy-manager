# Project Structure

A guide to the directories and files in this repository.

## Root Directory
- `backend/`: Rust backend source code.
- `frontend/`: React frontend source code.
- `data/`: (Local only) SQLite database and certificates.
- `logs/`: (Local only) Access and error logs.
- `docker-compose.yml`: Production Docker configuration.
- `Dockerfile`: Multi-stage build for the full application.

## Backend Structure (`/backend/src`)
- `main.rs`: Entry point. Initializes DB, API, and Proxy.
- `api/`: Axum API implementation.
    - `handlers/`: Logic for each API endpoint.
- `proxy/`: Pingora proxy logic.
    - `filters/`: Custom middleware for the proxy.
- `db/`: Database models and SQL queries using SQLx.
- `acme.rs`: SSL certificate automation logic.
- `state.rs`: Shared application state (ArcSwap).
- `tls_manager.rs`: SNI-based dynamic TLS certificate selection.
- `stream_manager.rs`: L4 TCP/UDP stream management.

## Frontend Structure (`/frontend/src`)
- `components/`: Reusable UI components (shadcn/ui).
- `pages/`: Main application views (Dashboard, Hosts, SSL, etc.).
- `store/`: State management (Redux/Zustand).
- `lib/`: Utility functions and API client.
- `hooks/`: Custom React hooks.

## Key Files
- `backend/Cargo.toml`: Backend dependencies.
- `frontend/package.json`: Frontend dependencies.
- `data/data.db`: SQLite database schema and data.
- `backend/src/constants.rs`: Application-wide constants (ports, timeouts).
