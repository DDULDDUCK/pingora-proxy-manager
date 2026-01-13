# Getting Started

Welcome to Pingora Proxy Manager! This guide will help you get your proxy up and running in minutes.

## Table of Contents
1. [Prerequisites](#prerequisites)
2. [Installation](#installation)
3. [First Login](#first-login)
4. [Creating Your First Proxy Host](#creating-your-first-proxy-host)

## Prerequisites
- [Docker](https://docs.docker.com/get-docker/) and [Docker Compose](https://docs.docker.com/compose/install/) installed.
- Ports 80 and 443 available for the proxy.
- Port 81 available for the management dashboard.

## Installation

### Using Docker Compose (Recommended)
Create a `docker-compose.yml` file:

```yaml
services:
  pingora-proxy:
    image: dduldduck/pingora-proxy-manager:latest
    container_name: pingora-proxy
    restart: always
    ports:
      - "80:8080"   # HTTP Proxy
      - "443:443"   # HTTPS Proxy
      - "81:81"     # Dashboard/API
    volumes:
      - ./data:/app/data
      - ./logs:/app/logs
    environment:
      - JWT_SECRET=your_super_secret_key
      - RUST_LOG=info
```

Run the container:
```bash
docker compose up -d
```

## First Login
1. Open your browser and navigate to `http://your-server-ip:81`.
2. Log in with the default credentials:
   - **Username**: `admin`
   - **Password**: `changeme`
3. **Important**: Change your password immediately in the user settings.

## Creating Your First Proxy Host
1. Go to the **Proxy Hosts** tab.
2. Click **Add Proxy Host**.
3. Enter your domain (e.g., `app.example.com`).
4. Enter the **Forward Host** (e.g., `192.168.1.10`) and **Forward Port** (e.g., `3000`).
5. Click **Save**.
6. Your application should now be accessible via `http://app.example.com`.

---
Next: [[Configuration Guide]]
