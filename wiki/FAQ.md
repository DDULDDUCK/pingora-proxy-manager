# FAQ & Troubleshooting

Common questions and issues.

## General Questions

### How does zero-downtime reloading work?
We use `ArcSwap` to manage the proxy configuration. When you make a change in the dashboard, the backend updates the SQLite database and then atomically swaps the in-memory configuration. The Pingora proxy handles the next request using the new configuration without dropping any active connections.

### Can I use this instead of Nginx Proxy Manager?
Yes! Pingora Proxy Manager is a modern alternative written in Rust. It's designed to be faster, more memory-efficient, and easier to extend.

## Troubleshooting

### Why is my SSL certificate not generating?
1. **HTTP-01**: Ensure your domain points to your server's public IP and port 80 is open.
2. **DNS-01**: Check your DNS Provider credentials. Ensure the API token has permissions to edit TXT records.
3. **Logs**: Check the logs in the dashboard or run `docker logs pingora-proxy` to see detailed error messages from Certbot.

### I forgot my admin password. How can I reset it?
Currently, password reset is available via the database.
1. Access the SQLite database: `sqlite3 data/data.db`
2. Update the password hash for the admin user (requires manual hash generation or using a tool).
*Future updates will include a CLI tool for password resets.*

### Port 80/443 is already in use
This usually happens if another web server is running. Stop any existing Nginx, Apache, or Traefik services before starting Pingora Proxy Manager.

### High CPU/Memory Usage
Pingora is very efficient. If you see high usage:
- Check the `access.log` for a possible DDoS attack.
- Verify if any upstream services are slow or timing out, causing connection pile-ups.
- Check the `RUST_LOG` level; `debug` or `trace` can impact performance.

---
Still have questions? Open an issue on [GitHub](https://github.com/dduldduck/pingora-proxy-manager/issues).
