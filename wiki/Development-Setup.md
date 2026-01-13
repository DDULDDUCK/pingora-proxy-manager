# Development Setup

Follow these instructions to set up a local development environment.

## Prerequisites
- **Rust**: Install via [rustup](https://rustup.rs/).
- **Node.js & npm**: For the frontend.
- **SQLite3**: For the database.
- **Certbot**: (Optional) Required for testing SSL generation locally.

## Repository Structure
- `/backend`: Rust source code.
- `/frontend`: React source code.
- `/data`: Default directory for database and certificates (ignored by git).

## Backend Setup
1. Navigate to the backend directory:
   ```bash
   cd backend
   ```
2. Create a `.env` file (optional, defaults are used):
   ```env
   JWT_SECRET=dev_secret
   RUST_LOG=debug
   ```
3. Run the backend:
   ```bash
   cargo run
   ```
   The backend will listen on:
   - `0.0.0.0:81` for API/Dashboard.
   - `0.0.0.0:8080` for HTTP Proxy.

## Frontend Setup
1. Navigate to the frontend directory:
   ```bash
   cd frontend
   ```
2. Install dependencies:
   ```bash
   npm install
   ```
3. Run the development server:
   ```bash
   npm run dev
   ```
4. Access the dashboard at `http://localhost:5173`. The Vite dev server is configured to proxy `/api` requests to `localhost:81`.

## Testing
### Backend Tests
```bash
cd backend
cargo test
```

### Docker Build Test
```bash
docker compose -f docker-compose.dev.yml up --build
```

## Tips
- **Database**: You can use any SQLite browser to inspect `data/data.db`.
- **Logs**: Backend logs are printed to stdout and also saved in `logs/`.
- **Pingora**: Note that Pingora might require specific permissions to bind to low ports (like 443). For local dev, we use 8080.

---
Next: [[API Documentation]]
