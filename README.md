# Pingora Proxy Manager
<p align="center">
  <!-- 프로젝트 로고 -->
  <img width="150" height="150" alt="ppnicon-removebg-preview" src="https://github.com/user-attachments/assets/3c9ec9cd-02f6-4a96-85e8-c125adb628cb" />
  <br>
</p>
<div align="center">

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![React](https://img.shields.io/badge/react-%2320232a.svg?style=for-the-badge&logo=react&logoColor=%2361DAFB)
![TailwindCSS](https://img.shields.io/badge/tailwindcss-%2338B2AC.svg?style=for-the-badge&logo=tailwind-css&logoColor=white)
![Docker](https://img.shields.io/badge/docker-%230db7ed.svg?style=for-the-badge&logo=docker&logoColor=white)
![License](https://img.shields.io/badge/license-MIT-blue.svg?style=for-the-badge)
<a href="https://www.buymeacoffee.com/dduldduck">
  <img src="https://img.shields.io/badge/Donate-Buy%20Me%20A%20Coffee-orange.svg?style=for-the-badge&logo=buymeacoffee" alt="Donate" />
</a>

**A high-performance, zero-downtime reverse proxy manager built on Cloudflare's [Pingora](https://github.com/cloudflare/pingora).**

Simple, Modern, and Fast. Now supports Wildcard SSL & TCP/UDP Streams!

</div>

---

## ✨ Features

- **⚡️ High Performance:** Built on Rust & Pingora, capable of handling high traffic with low latency.
- **🔄 Zero-Downtime Configuration:** Dynamic reconfiguration without restarting the process.
- **🔒 SSL/TLS Automation:** 
  - **HTTP-01:** Standard challenge for single domains.
  - **DNS-01:** **Wildcard certificate support** (`*.example.com`) via Cloudflare, AWS Route53, etc. (powered by Certbot).
- **🌐 Proxy Hosts:** Easy management of virtual hosts, locations, and path rewriting.
- **📡 Streams (L4):** TCP and UDP forwarding for databases, game servers, etc.
- **🛡️ Access Control:** IP whitelisting/blacklisting and Basic Authentication support.
- **🎨 Modern Dashboard:** Clean and responsive UI built with React, Tailwind CSS, and shadcn/ui.
- **🐳 Docker Ready:** Single container deployment for easy setup and maintenance.

## ❤️ Support the Development

**Is Pingora Proxy Manager saving you time?**

This project is built with love, caffeine, and many sleepless nights to provide a high-performance, free alternative for the community. Maintaining an open-source project takes significant effort. 

If you'd like to support the ongoing development, bug fixes, and new features, please consider buying me a coffee! ☕️

<div align="center">
  <a href="https://www.buymeacoffee.com/dduldduck" target="_blank">
    <img width="400" alt="Buy Me A Coffee" src="https://github.com/user-attachments/assets/120ade05-f821-4a0a-913a-03b6532ce77b" />
  </a>
  <p><i>Your support keeps the code flowing.</i></p>
</div>

## 🚀 Getting Started

### Quick Start (Docker Hub)

You can run the pre-built image directly from Docker Hub.

**Using Docker CLI:**
```bash
docker run -d \
  --name pingora-proxy \
  -p 80:8080 \
  -p 81:81 \
  -v ./data:/app/data \
  -v ./logs:/app/logs \
  dduldduck/pingora-proxy-manager:latest
```

**Using Docker Compose:**
Create a `docker-compose.yml`:

```yaml
services:
  pingora-proxy:
    image: dduldduck/pingora-proxy-manager:latest
    container_name: pingora-proxy
    restart: always
    ports:
      - "80:8080"   # HTTP Proxy (Backend listens on 8080)
      - "81:81"     # Dashboard/API (Backend listens on 81)
      # Map 443 if you want to serve HTTPS directly (requires privilege or capability)
      # - "443:443" 
    volumes:
      - ./data:/app/data        # DB and Certs persistence
      - ./logs:/app/logs        # Logs persistence
    environment:
      - JWT_SECRET=changeme_in_production_please
      - RUST_LOG=info
```

Then run:
```bash
docker compose up -d
```

### Access the Dashboard
- Open your browser and go to `http://localhost:81`.
- **Default Credentials:**
  - Username: `admin`
  - Password: `changeme` (Please change this immediately!)

## 🛠️ Building from Source

If you want to build the image yourself:

1. **Clone the repository:**
   ```bash
   git clone https://github.com/dduldduck/pingora-proxy-manager.git
   cd pingora-proxy-manager
   ```

2. **Build and Start:**
   ```bash
   docker compose up --build -d
   ```

## 📦 Development

### Native Development (Recommended)

You can run the backend and frontend locally without Docker for faster iteration.

**1. Start Backend:**
The backend will automatically detect it's running in dev mode and use the project root `data/` folder.
```bash
# Terminal 1
cd backend
cargo run
# Backend listens on 0.0.0.0:81 (API) and 0.0.0.0:8080 (Proxy)
```

**2. Start Frontend:**
The frontend dev server is configured to proxy API requests to `localhost:81`.
```bash
# Terminal 2
cd frontend
npm install
npm run dev
# Open http://localhost:5173 (or the port shown in terminal)
```

### Docker Development

If you prefer to test the production build locally:

```bash
# Uses docker-compose.dev.yml to build from local source
docker compose -f docker-compose.dev.yml up --build
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
