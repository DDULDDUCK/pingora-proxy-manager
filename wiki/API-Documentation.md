# API Documentation

The Pingora Proxy Manager provides a RESTful API on port 81. All requests (except login) require a JWT token in the `Authorization` header.

## Base URL
`http://<your-server-ip>:81/api`

## Authentication

### `POST /login`
Authenticates a user and returns a JWT.

**Request Body:**
```json
{
  "username": "admin",
  "password": "changeme"
}
```

**Response:**
```json
{
  "token": "eyJhbG..."
}
```

---

## Proxy Hosts

### `GET /hosts`
List all proxy hosts.

Each host object now includes these optional advanced upstream fields:

- `connection_timeout_ms`
- `read_timeout_ms`
- `write_timeout_ms`
- `max_request_body_bytes`

Location objects returned inside `locations` include the same four optional fields.

**Response Example:**
```json
[
  {
    "domain": "example.com",
    "target": "127.0.0.1:3000",
    "scheme": "http",
    "ssl_forced": true,
    "verify_ssl": true,
    "connection_timeout_ms": null,
    "read_timeout_ms": 60000,
    "write_timeout_ms": null,
    "max_request_body_bytes": 10485760,
    "locations": [
      {
        "path": "/api",
        "target": "127.0.0.1:4000",
        "scheme": "http",
        "rewrite": false,
        "verify_ssl": true,
        "connection_timeout_ms": null,
        "read_timeout_ms": 90000,
        "write_timeout_ms": null,
        "max_request_body_bytes": null
      }
    ]
  }
]
```

### `POST /hosts`
Add a new proxy host.

**Request Body:**
```json
{
  "domain": "example.com",
  "target": "127.0.0.1:3000",
  "scheme": "http",
  "ssl_forced": true,
  "connection_timeout_ms": 500,
  "read_timeout_ms": 10000,
  "write_timeout_ms": 5000,
  "max_request_body_bytes": 10485760
}
```

All four advanced fields are optional. When omitted or set to `null`, PPM keeps the built-in defaults.

### `DELETE /hosts/{domain}`
Delete a proxy host.

### `POST /hosts/{domain}/locations`
Add or replace a location for a proxy host.

**Request Body:**
```json
{
  "path": "/api",
  "target": "127.0.0.1:4000",
  "scheme": "http",
  "rewrite": false,
  "verify_ssl": true,
  "connection_timeout_ms": 500,
  "read_timeout_ms": 90000,
  "write_timeout_ms": 5000,
  "max_request_body_bytes": 10485760
}
```

The same four advanced fields are optional here as well. When omitted or set to `null`, the location inherits the host-level value first, then PPM falls back to the built-in defaults.

### `DELETE /hosts/{domain}/locations?path=/api`
Delete a location from a proxy host.

---

## SSL Certificates

### `GET /certs`
List all managed certificates.

### `POST /certs`
Request a new certificate via Let's Encrypt.

---

## Streams (L4)

### `GET /streams`
List all L4 streams.

### `POST /streams`
Add a new stream.

**Request Body:**
```json
{
  "listen_port": 3306,
  "forward_host": "db.internal",
  "forward_port": 3306,
  "protocol": "tcp"
}
```

---

## Access Control

### `GET /access-lists`
List all Access Control Lists.

### `POST /access-lists`
Create a new ACL.

---

## Monitoring

### `GET /stats/realtime`
Get current traffic statistics.

### `GET /stats/history`
Get historical traffic data (time-series).

### `GET /audit-logs`
Get recent admin activity logs.

### `GET /metrics` (Root Level)
Prometheus compatible metrics.

---

*Note: For a full list of endpoints and parameters, refer to the `backend/src/api/mod.rs` file.*
