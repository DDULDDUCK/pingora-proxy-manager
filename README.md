<div align="center">

<img width="180" height="180" src="https://github.com/user-attachments/assets/3c9ec9cd-02f6-4a96-85e8-c125adb628cb" alt="Pingora Proxy Manager Logo" />

# Pingora Proxy Manager

**High-Performance • Zero-Downtime • Modern UI**

Built on Cloudflare's [Pingora](https://github.com/cloudflare/pingora) and Rust.

[![GitHub Wiki](https://img.shields.io/badge/Documentation-Wiki-book?style=for-the-badge&logo=gitbook&logoColor=white)](https://github.com/DDULDDUCK/pingora-proxy-manager/wiki)
[![License](https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge)](LICENSE)
[![Docker](https://img.shields.io/badge/Docker-Hub-blue.svg?style=for-the-badge&logo=docker&logoColor=white)](https://hub.docker.com/r/dduldduck/pingora-proxy-manager)

[![Rust](https://img.shields.io/badge/Backend-Rust-black?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![React](https://img.shields.io/badge/Frontend-React_19-black?style=flat-square&logo=react)](https://react.dev/)
[![Tailwind](https://img.shields.io/badge/Style-Tailwind_CSS-black?style=flat-square&logo=tailwindcss)](https://tailwindcss.com/)

[Features](#-features) • [Quick Start](#-quick-start) • [Documentation](#-documentation) • [Contributing](#-contributing)

</div>

---

## 📖 Overview

**Pingora Proxy Manager** is a next-generation reverse proxy solution designed for speed and reliability. It combines the raw performance of Rust and Cloudflare's Pingora framework with a beautiful, modern React dashboard.

Unlike traditional proxies, PPM supports **zero-downtime configuration reloading**, meaning your active connections are never dropped when you add hosts or change SSL settings.

<img width="1302" height="724" alt="image" src="https://github.com/user-attachments/assets/aeb84f5a-5db8-4f8a-94cc-d355301907f4" />
<img width="1301" height="707" alt="image" src="https://github.com/user-attachments/assets/62add77b-a909-4ffb-8102-3b57c2007c3b" />
<img width="1289" height="637" alt="image" src="https://github.com/user-attachments/assets/9d0e3a07-f79a-4f45-9fe3-d97ae9867fef" />
<img width="1301" height="538" alt="image" src="https://github.com/user-attachments/assets/a1dfd699-492f-4218-8a23-579e9fdb17aa" />

## ✨ Features

| Feature | Description |
| :--- | :--- |
| **⚡️ High Performance** | Powered by **Rust** & **Pingora** for ultra-low latency and high throughput. |
| **🔄 Zero-Downtime** | Dynamic architecture allows configuration changes without restarting the process. |
| **🔒 SSL Automation** | **HTTP-01** and **DNS-01 (Wildcard)** support via Let's Encrypt. |
| **🌐 Proxy Hosts** | Easy management of virtual hosts, path routing, and advanced rewriting. |
| **📡 L4 Streams** | **TCP/UDP** forwarding for databases, game servers, and other non-HTTP services. |
| **🛡️ Access Control** | Secure your services with IP Whitelists/Blacklists and Basic Authentication. |
| **📊 Real-time Stats** | Live traffic monitoring, status codes, and historical data visualization. |
| **🐳 Docker Ready** | Single-container deployment with a lightweight footprint. |

## 🚀 Quick Start

The fastest way to get started is using **Docker Compose**.

### 1. Create `docker-compose.yml`

```yaml
services:
  pingora-proxy:
    image: dduldduck/pingora-proxy-manager:latest
    container_name: pingora-proxy
    restart: always
    network_mode: host # Recommended for performance & L4 streams
    volumes:
      - ./data:/app/data        # Database & Certs
      - ./logs:/app/logs        # Access Logs
    environment:
      - JWT_SECRET=changeme_in_production_please
      - RUST_LOG=info
```

> **Note**: We recommend `network_mode: host` for best performance and simplified port management. If you prefer bridge mode, ensure you map ports `80:8080`, `443:443`, and `81:81`.

### 2. Start the Service

```bash
docker compose up -d
```

### 3. Access Dashboard

Visit **http://localhost:81** (or your server IP) and log in:

*   **Username:** `admin`
*   **Password:** `changeme`

> ⚠️ **Important:** Please change your password immediately after logging in.

## 📚 Documentation

We have comprehensive documentation available in our **GitHub Wiki**:

*   **[Getting Started](https://github.com/DDULDDUCK/pingora-proxy-manager/wiki/Getting-Started)** - Installation and first steps.
*   **[Configuration Guide](https://github.com/DDULDDUCK/pingora-proxy-manager/wiki/Configuration-Guide)** - Deep dive into Proxy Hosts, SSL, and ACLs.
*   **[Architecture](https://github.com/DDULDDUCK/pingora-proxy-manager/wiki/Architecture)** - Understand the Control Plane and Data Plane.
*   **[API Reference](https://github.com/DDULDDUCK/pingora-proxy-manager/wiki/API-Documentation)** - Integrate with our REST API.
*   **[FAQ & Troubleshooting](https://github.com/DDULDDUCK/pingora-proxy-manager/wiki/FAQ)** - Common solutions.

## 🛠️ Development

Want to contribute or build from source?

### Native Development
You can run the backend (Rust) and frontend (React/Vite) independently for faster iteration.

1.  **Backend**: `cd backend && cargo run` (Listens on 81 & 8080)
2.  **Frontend**: `cd frontend && npm run dev` (Proxies API to 81)

See the **[Development Setup](https://github.com/DDULDDUCK/pingora-proxy-manager/wiki/Development-Setup)** guide for details.

## ❤️ Support the Project

**Is Pingora Proxy Manager saving you time?**

This project is built with love, caffeine, and many sleepless nights. If you'd like to support the development, server costs, and new features, consider buying me a coffee! ☕️

<div align="center">
  <a href="https://www.buymeacoffee.com/dduldduck" target="_blank">
    <img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy Me A Coffee" width="180" />
  </a>
</div>

## 🤝 Contributing

Contributions are what make the open-source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

Please check our **[Contributing Guide](https://github.com/DDULDDUCK/pingora-proxy-manager/wiki/Contributing-Guide)** for guidelines.

## 📄 License

Distributed under the MIT License. See [`LICENSE`](LICENSE) for more information.
