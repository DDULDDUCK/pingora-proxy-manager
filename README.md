

# Pingora Proxy Manager
<p align="center">
  <!-- 프로젝트 로고를 여기에 추가할 수 있습니다. -->
  <img width="150" height="150" alt="ppnicon-removebg-preview" src="https://github.com/user-attachments/assets/3c9ec9cd-02f6-4a96-85e8-c125adb628cb" />
  <br>
</p>
<div align="center">

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![React](https://img.shields.io/badge/react-%2320232a.svg?style=for-the-badge&logo=react&logoColor=%2361DAFB)
![TailwindCSS](https://img.shields.io/badge/tailwindcss-%2338B2AC.svg?style=for-the-badge&logo=tailwind-css&logoColor=white)
![Docker](https://img.shields.io/badge/docker-%230db7ed.svg?style=for-the-badge&logo=docker&logoColor=white)
![License](https://img.shields.io/badge/license-MIT-blue.svg?style=for-the-badge)

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

## 🏗️ Architecture

- **Data Plane (8080/443):** [Pingora](https://github.com/cloudflare/pingora) handles all traffic with high efficiency.
- **Control Plane (81):** [Axum](https://github.com/tokio-rs/axum) serves the API and Dashboard.
- **SSL Management:** Integrated `Certbot` for robust ACME handling.
- **State Management:** `ArcSwap` for lock-free configuration reads.
- **Database:** SQLite for persistent storage of hosts and certificates.

## 📦 Development

To run locally without Docker (requires Rust and Node.js):

**Backend:**
```bash
cd backend
cargo run
```

**Frontend:**
```bash
cd frontend
npm install
npm run dev
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
