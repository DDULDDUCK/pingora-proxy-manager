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

### `POST /hosts`
Add a new proxy host.

**Request Body:**
```json
{
  "domain": "example.com",
  "target": "127.0.0.1:3000",
  "scheme": "http",
  "ssl_forced": true
}
```

### `DELETE /hosts/{domain}`
Delete a proxy host.

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
