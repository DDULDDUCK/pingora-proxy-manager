//! API handlers for the Pingora Proxy Manager.
//!
//! This module contains the request handlers for various API endpoints,
//! including authentication, host management, certificate management,
//! and system statistics.

pub mod access_lists;
pub mod auth;
pub mod certs;
pub mod hosts;
pub mod stats;
pub mod streams;
pub mod users;
