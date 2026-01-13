# Configuration Guide

This guide provides detailed instructions on how to configure the various components of Pingora Proxy Manager.

## Table of Contents
1. [Proxy Hosts](#1-proxy-hosts)
2. [Locations & Path Routing](#2-locations--path-routing)
3. [SSL Certificates](#3-ssl-certificates)
4. [L4 Streams](#4-l4-streams)
5. [Access Control Lists](#5-access-control-lists)

## 1. Proxy Hosts
A Proxy Host is the primary way to route incoming HTTP/HTTPS traffic.

- **Domain**: The public domain name (e.g., `www.example.com`).
- **Scheme**: Choose `http` or `https` for the connection to your upstream server.
- **Forward Host**: The IP address or hostname of your internal service.
- **Forward Port**: The port your service is listening on.
- **SSL Forced**: Redirect all HTTP traffic to HTTPS automatically.

## 2. Locations & Path Routing
You can add multiple locations to a single Proxy Host to route different paths to different services.

| Feature | Description |
| --- | --- |
| **Path** | The URL path to match (e.g., `/api`). |
| **Target** | The upstream address for this specific path. |
| **Rewrite** | If enabled, the matched path is removed from the URL before forwarding (e.g., `/api/users` -> `/users`). |

## 3. SSL Certificates
We support two types of Let's Encrypt challenges:

### HTTP-01
- Requires port 80 to be open and reachable from the internet.
- Does not support wildcard domains.

### DNS-01 (Wildcard)
- Required for wildcard certificates (e.g., `*.example.com`).
- Requires a DNS Provider configuration (e.g., Cloudflare API Token).
- Validation happens via DNS TXT records.

## 4. L4 Streams
Used for non-HTTP traffic.

- **Listen Port**: The port on the proxy server.
- **Protocol**: `TCP` or `UDP`.
- **Forward Host**: The internal destination IP.
- **Forward Port**: The internal destination port.

## 5. Access Control Lists
Create an ACL to restrict access to your Proxy Hosts.

- **IP Restrictions**: Add IPs to the list and set the action to `allow` or `deny`.
- **Basic Auth**: Create users with usernames and passwords. Only authorized users can access the host.

> **Note**: If both IP restrictions and Basic Auth are used, the user must satisfy both conditions (depending on implementation, usually Basic Auth is prompt after IP whitelist check).

---
Next: [[Deployment]]
