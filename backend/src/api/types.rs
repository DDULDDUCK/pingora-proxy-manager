use crate::state::LocationConfig;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct LoginReq {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginRes {
    pub token: String,
}

#[derive(Deserialize)]
pub struct CreateHostReq {
    pub domain: String,
    pub target: String,
    pub scheme: Option<String>,
    pub ssl_forced: Option<bool>,
    pub verify_ssl: Option<bool>,
    pub redirect_to: Option<String>,
    pub redirect_status: Option<i64>,
    pub access_list_id: Option<i64>,
}

#[derive(Deserialize)]
pub struct CreateLocationReq {
    pub path: String,
    pub target: String,
    pub scheme: Option<String>,
    pub rewrite: Option<bool>,
    pub verify_ssl: Option<bool>,
}

#[derive(Deserialize)]
pub struct DeleteLocationQuery {
    pub path: String,
}

#[derive(Serialize)]
pub struct LocationRes {
    pub path: String,
    pub target: String,
    pub scheme: String,
    pub rewrite: bool,
    pub verify_ssl: bool,
}

#[derive(Serialize)]
pub struct HostRes {
    pub domain: String,
    pub target: String,
    pub scheme: String,
    pub ssl_forced: bool,
    pub verify_ssl: bool,
    pub redirect_to: Option<String>,
    pub redirect_status: u16,
    pub locations: Vec<LocationRes>,
    pub access_list_id: Option<i64>,
    pub headers: Vec<HeaderRes>,
}

#[derive(Deserialize)]
pub struct CreateCertReq {
    pub domain: String,
    pub email: String,
    pub provider_id: Option<i64>,
}

#[derive(Serialize)]
pub struct CertRes {
    pub id: i64,
    pub domain: String,
    pub expires_at: i64,
}

#[derive(Serialize)]
pub struct RealtimeStatsRes {
    pub requests: u64,
    pub bytes: u64,
    pub status_2xx: u64,
    pub status_4xx: u64,
    pub status_5xx: u64,
}

#[derive(Deserialize)]
pub struct HistoryStatsQuery {
    pub hours: Option<i64>,
}

#[derive(Deserialize)]
pub struct LogsQuery {
    pub lines: Option<usize>,
}

#[derive(Deserialize)]
pub struct CreateStreamReq {
    pub listen_port: u16,
    pub forward_host: String,
    pub forward_port: u16,
    pub protocol: Option<String>,
}

#[derive(Serialize)]
pub struct StreamRes {
    pub id: i64,
    pub listen_port: i64,
    pub forward_host: String,
    pub forward_port: i64,
    pub protocol: String,
}

#[derive(Deserialize)]
pub struct ErrorPageReq {
    pub html: String,
}

// --- Access List Structs ---

#[derive(Deserialize)]
pub struct CreateAccessListReq {
    pub name: String,
    #[serde(default)]
    pub clients: Vec<AccessListClientReq>,
    #[serde(default)]
    pub ips: Vec<AccessListIpReq>,
}

#[derive(Deserialize)]
pub struct AccessListClientReq {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct AccessListIpReq {
    pub ip: String,
    pub action: String, // "allow" or "deny"
}

#[derive(Serialize)]
pub struct AccessListRes {
    pub id: i64,
    pub name: String,
    pub clients: Vec<AccessListClientRes>,
    pub ips: Vec<AccessListIpRes>,
}

#[derive(Serialize)]
pub struct AccessListClientRes {
    pub username: String,
    // password_hash removed
}

#[derive(Serialize)]
pub struct AccessListIpRes {
    pub ip: String,
    pub action: String,
}

// --- User Management Structs ---

#[derive(Deserialize)]
pub struct CreateUserReq {
    pub username: String,
    pub password: String,
    pub role: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateUserReq {
    pub password: Option<String>,
    pub role: Option<String>,
}

#[derive(Serialize)]
pub struct UserRes {
    pub id: i64,
    pub username: String,
    pub role: String,
    pub created_at: i64,
    pub last_login: Option<i64>,
}

#[derive(Deserialize)]
pub struct ChangePasswordReq {
    pub current_password: String,
    pub new_password: String,
}

// --- Audit Log Structs ---

#[derive(Deserialize)]
pub struct AuditLogQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub username: Option<String>,
    pub resource_type: Option<String>,
}

#[derive(Serialize)]
pub struct AuditLogRes {
    pub id: i64,
    pub timestamp: i64,
    pub username: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub details: Option<String>,
    pub ip_address: Option<String>,
}

// --- DNS Provider Structs ---

#[derive(Deserialize)]
pub struct CreateDnsProviderReq {
    pub name: String,
    pub provider_type: String,
    pub credentials: String,
}

#[derive(Serialize)]
pub struct DnsProviderRes {
    pub id: i64,
    pub name: String,
    pub provider_type: String,
    pub created_at: i64,
}

// --- Custom Header Structs ---
#[derive(Deserialize)]
pub struct CreateHeaderReq {
    pub name: String,
    pub value: String,
    pub target: String,
}

#[derive(Serialize)]
pub struct HeaderRes {
    pub id: i64,
    pub name: String,
    pub value: String,
    pub target: String,
}
