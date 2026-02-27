# Deployment Guide

Best practices for deploying Pingora Proxy Manager in a production environment.

## Table of Contents
- [Docker Deployment](#docker-deployment)
- [Persistence](#persistence)
- [Security](#security)
- [Performance Tuning](#performance-tuning)

## Docker Deployment
Using Docker is the recommended way to deploy.

### Sample `docker-compose.yml`
```yaml
services:
  pingora-proxy:
    image: dduldduck/pingora-proxy-manager:latest
    container_name: pingora-proxy
    restart: always
    network_mode: host # Recommended for L4 streams and performance
    volumes:
      - ./data:/app/data
      - ./logs:/app/logs
    environment:
      - JWT_SECRET=change_me_to_something_long_and_random
      - RUST_LOG=warn # Reduce log noise in production
```

## Persistence
It is crucial to mount the `/app/data` volume. This directory contains:
- `data.db`: The SQLite database with all your configurations.
- `certs/`: All generated SSL certificates and private keys.

Without this volume, all your settings and certificates will be lost when the container is deleted.

## Security
1. **JWT Secret**: Always change the `JWT_SECRET` environment variable to a unique, random string.
2. **Dashboard Port**: By default, the dashboard is on port 81. Consider restricting access to this port via a firewall or a VPN.
3. **Running as Non-Root**: The Docker image is designed to run with necessary capabilities to bind to ports 80/443 without being full root where possible, but `network_mode: host` usually requires higher privileges.
4. **Trusted Proxy Headers**: If PPM is behind another reverse proxy/load balancer, set `PPM_TRUSTED_PROXY_IPS` (or `TRUSTED_PROXY_IPS`) to the IP addresses of the immediate upstream proxy hop. By default, only loopback (`127.0.0.1`, `::1`) is trusted for forwarded headers.

### Trusted Proxy Example

```yaml
environment:
  - PPM_TRUSTED_PROXY_IPS=127.0.0.1,10.0.0.10
```

Without this setting in proxied deployments, `X-Forwarded-For` and `X-Forwarded-Proto` may be ignored, which can change ACL and SSL-force behavior.

## Performance Tuning
Pingora is highly efficient, but you can optimize it further:
- **File Descriptors**: Ensure your host system has a high enough limit for open files (`ulimit -n`).
- **CPU Pinning**: For extreme performance, consider pinning the process to specific CPU cores.
- **Log Rotation**: Logs can grow quickly. Ensure you have a log rotation mechanism in place (the app logs to `logs/access.log`).

## Troubleshooting
If you encounter issues binding to port 443:
- Check if another service (like Nginx or Apache) is already using it.
- Ensure the user running the container has the `CAP_NET_BIND_SERVICE` capability.
