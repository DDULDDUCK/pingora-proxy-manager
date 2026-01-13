# Features

Pingora Proxy Manager is packed with features designed for performance and ease of use.

## Table of Contents
- [Proxy Hosts](#proxy-hosts)
- [SSL/TLS Management](#ssltls-management)
- [L4 Streams](#l4-streams)
- [Access Control Lists](#access-control-lists)
- [Monitoring & Stats](#monitoring--stats)

## Proxy Hosts
Manage your HTTP/HTTPS reverse proxy rules with ease.
- **Domain Management**: Map multiple domains to different backends.
- **Path Routing**: Route requests based on URL paths (e.g., `/api` to one backend, `/` to another).
- **Load Balancing**: Distribute traffic across multiple upstream servers using the **Random** strategy.
- **Path Rewriting**: Strip or modify paths before forwarding to the upstream.
- **Custom Headers**: Add, modify, or remove HTTP headers for both requests and responses.

## SSL/TLS Management
Automated certificate management powered by Let's Encrypt and Certbot.
- **HTTP-01 Challenge**: Simple validation for standard domains.
- **DNS-01 Challenge**: Support for **Wildcard Certificates** (`*.example.com`).
- **DNS Providers**: Integrated support for Cloudflare, AWS Route53, DigitalOcean, and more.
- **Automatic Renewal**: Certificates are automatically renewed before they expire.
- **Custom Certificates**: Upload your own existing SSL certificates.

## L4 Streams
Forward raw TCP and UDP traffic.
- **TCP Forwarding**: Perfect for databases (MySQL, Postgres), SSH, etc.
- **UDP Forwarding**: Suitable for game servers, DNS, and other UDP-based protocols.
- **Port Mapping**: Map any external port to any internal IP and port.

## Access Control Lists (ACL)
Secure your hosts with IP-based or User-based restrictions.
- **IP Whitelisting/Blacklisting**: Allow or deny traffic based on source IP address.
- **Basic Auth**: Protect your web services with a username and password.
- **Global or Host-specific**: Apply ACLs to specific proxy hosts.

## Monitoring & Stats
Keep an eye on your traffic in real-time.
- **Real-time Dashboard**: View current requests per second, bandwidth, and status codes.
- **Historical Data**: Analyze traffic patterns over time with built-in charts.
- **Audit Logs**: Track configuration changes made by administrators.
- **Access Logs**: View detailed proxy logs for troubleshooting.
- **Prometheus Metrics**: Export metrics to your existing monitoring stack (Grafana/Prometheus) via the `/metrics` endpoint.
