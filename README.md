# 🚀 Pingora Proxy Manager

<div align="center">

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![React](https://img.shields.io/badge/react-%2320232a.svg?style=for-the-badge&logo=react&logoColor=%2361DAFB)
![TailwindCSS](https://img.shields.io/badge/tailwindcss-%2338B2AC.svg?style=for-the-badge&logo=tailwind-css&logoColor=white)
![Docker](https://img.shields.io/badge/docker-%230db7ed.svg?style=for-the-badge&logo=docker&logoColor=white)
![License](https://img.shields.io/badge/license-MIT-blue.svg?style=for-the-badge)

**A high-performance, zero-downtime reverse proxy manager built on Cloudflare's [Pingora](https://github.com/cloudflare/pingora).**

Simple, Modern, and Fast.

</div>

---

## ✨ Features

- **⚡️ High Performance:** Built on Rust & Pingora, capable of handling high traffic with low latency.
- **🔄 Zero-Downtime Configuration:** Dynamic reconfiguration without restarting the process.
- **🔒 Automatic SSL/TLS:** Automated certificate issuance and renewal via Let's Encrypt (ACME).
- **🎨 Modern Dashboard:** Clean and responsive UI built with React, Tailwind CSS, and shadcn/ui.
- **🔐 Secure:** Built-in authentication, JWT protection, and secure password hashing.
- **🐳 Docker Ready:** Single container deployment for easy setup and maintenance.

## 📸 Screenshots

*(Add your screenshots here)*

## 🚀 Getting Started

### Prerequisites

- Docker & Docker Compose installed on your machine.

### Installation

1. **Clone the repository:**
   ```bash
   git clone https://github.com/dduldduck/pingora-proxy-manager.git
   cd pingora-proxy-manager
   ```

2. **Start with Docker Compose:**
   ```bash
   docker compose up --build -d
   ```

3. **Access the Dashboard:**
   - Open your browser and go to `http://localhost:81`.
   - **Default Credentials:**
     - Email/Username: `admin`
     - Password: `changeme`

## 🛠️ Architecture

- **Data Plane (80/443):** [Pingora](https://github.com/cloudflare/pingora) handles all traffic with high efficiency.
- **Control Plane (81):** [Axum](https://github.com/tokio-rs/axum) serves the API and Dashboard.
- **State Management:** `ArcSwap` for lock-free configuration reads.
- **Database:** SQLite for persistent storage of hosts and certificates.

## 📦 Development

If you want to run it locally without Docker:

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
