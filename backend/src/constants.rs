/// HTTP Status Codes
pub mod http {
    use http::StatusCode;

    pub const OK: u16 = StatusCode::OK.as_u16();
    pub const FORBIDDEN: u16 = StatusCode::FORBIDDEN.as_u16();
    pub const NOT_FOUND: u16 = StatusCode::NOT_FOUND.as_u16();
    pub const INTERNAL_ERROR: u16 = StatusCode::INTERNAL_SERVER_ERROR.as_u16();
    pub const UNAUTHORIZED: u16 = StatusCode::UNAUTHORIZED.as_u16();
    pub const CREATED: u16 = StatusCode::CREATED.as_u16();
}

/// Network Configuration
pub mod network {
    pub const API_PORT_STR: &str = "0.0.0.0:81";
    pub const PROXY_PORT_STR: &str = "0.0.0.0:8080";
    pub const TLS_PORT_STR: &str = "0.0.0.0:443";
    pub const UDP_BUFFER_SIZE: usize = 65535;
}

/// Certificate Settings
pub mod cert {
    pub const RSA_BITS: u32 = 2048;
    pub const VALIDITY_DAYS: u32 = 3650; // 10 years
    pub const RENEWAL_THRESHOLD_DAYS: u64 = 30;
}

/// Timeouts (milliseconds/seconds)
pub mod timeout {
    pub const CONNECTION_MS: u64 = 500;
    pub const READ_SECS: u64 = 10;
    pub const WRITE_SECS: u64 = 5;
    pub const TCP_TIMEOUT_SECS: u64 = 300;
    pub const UDP_SESSION_TIMEOUT_SECS: u64 = 60;
}
