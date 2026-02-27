use pingora::prelude::*;
use std::collections::HashSet;
use std::net::IpAddr;
use std::sync::OnceLock;

fn trusted_proxy_ips() -> &'static HashSet<IpAddr> {
    static TRUSTED_PROXY_IPS: OnceLock<HashSet<IpAddr>> = OnceLock::new();

    TRUSTED_PROXY_IPS.get_or_init(|| {
        let mut ips = HashSet::new();

        ips.insert(IpAddr::from([127, 0, 0, 1]));
        ips.insert(IpAddr::from([0, 0, 0, 0, 0, 0, 0, 1]));

        if let Ok(raw) =
            std::env::var("PPM_TRUSTED_PROXY_IPS").or_else(|_| std::env::var("TRUSTED_PROXY_IPS"))
        {
            for part in raw.split(',').map(str::trim).filter(|p| !p.is_empty()) {
                match part.parse::<IpAddr>() {
                    Ok(ip) => {
                        ips.insert(ip);
                    }
                    Err(_) => {
                        tracing::warn!("Ignoring invalid trusted proxy IP: {}", part);
                    }
                }
            }
        }

        ips
    })
}

pub fn downstream_client_ip(session: &Session) -> Option<IpAddr> {
    session
        .client_addr()
        .and_then(|addr| addr.as_inet().map(|inet| inet.ip()))
}

pub fn is_trusted_proxy_hop(session: &Session) -> bool {
    downstream_client_ip(session)
        .map(|ip| trusted_proxy_ips().contains(&ip))
        .unwrap_or(false)
}

pub fn forwarded_proto_is_https(session: &Session) -> bool {
    if !is_trusted_proxy_hop(session) {
        return false;
    }

    session
        .req_header()
        .headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("https"))
        .unwrap_or(false)
}

pub fn effective_client_ip(session: &Session) -> Option<IpAddr> {
    if is_trusted_proxy_hop(session) {
        if let Some(forwarded_for) = session
            .req_header()
            .headers
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
        {
            for candidate in forwarded_for.split(',').map(str::trim) {
                if let Ok(ip) = candidate.parse::<IpAddr>() {
                    return Some(ip);
                }
            }
        }
    }

    downstream_client_ip(session)
}
