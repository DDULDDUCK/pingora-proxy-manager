use crate::db::{self, DbPool};
use crate::state::{
    AccessListClientConfig, AccessListConfig, AccessListIpConfig, HeaderConfig, HostConfig,
    LocationConfig, ProxyConfig,
};
use std::collections::HashMap;

pub struct ConfigLoader;

impl ConfigLoader {
    pub async fn load_from_db(pool: &DbPool) -> Result<ProxyConfig, Box<dyn std::error::Error>> {
        let hosts_result = db::get_all_hosts(pool).await;
        let locations_result = db::get_all_locations(pool).await;
        let access_lists_result = db::get_all_access_lists(pool).await;
        let clients_result = db::get_access_list_clients(pool).await;
        let ips_result = db::get_access_list_ips(pool).await;
        let headers_result = db::get_all_headers(pool).await;

        if let (
            Ok(rows),
            Ok(loc_rows),
            Ok(al_rows),
            Ok(client_rows),
            Ok(ip_rows),
            Ok(header_rows),
        ) = (
            hosts_result,
            locations_result,
            access_lists_result,
            clients_result,
            ips_result,
            headers_result,
        ) {
            // 1. Locations
            let mut locations_map: HashMap<i64, Vec<LocationConfig>> = HashMap::new();
            for loc in loc_rows {
                locations_map
                    .entry(loc.host_id)
                    .or_default()
                    .push(LocationConfig {
                        path: loc.path,
                        target: loc.target,
                        scheme: loc.scheme,
                        rewrite: loc.rewrite,
                    });
            }

            // 2. Access Lists
            let mut access_lists = HashMap::new();

            // Group Clients and IPs by list_id
            let mut clients_map: HashMap<i64, Vec<AccessListClientConfig>> = HashMap::new();
            for c in client_rows {
                clients_map
                    .entry(c.list_id)
                    .or_default()
                    .push(AccessListClientConfig {
                        username: c.username,
                        password_hash: c.password_hash,
                    });
            }

            let mut ips_map: HashMap<i64, Vec<AccessListIpConfig>> = HashMap::new();
            for ip in ip_rows {
                ips_map.entry(ip.list_id).or_default().push(AccessListIpConfig {
                    ip: ip.ip_address,
                    action: ip.action,
                });
            }

            for al in al_rows {
                access_lists.insert(
                    al.id,
                    AccessListConfig {
                        id: al.id,
                        name: al.name,
                        clients: clients_map.remove(&al.id).unwrap_or_default(),
                        ips: ips_map.remove(&al.id).unwrap_or_default(),
                    },
                );
            }

            // 3. Headers (grouped by host_id for ProxyConfig)
            let mut headers_map: HashMap<i64, Vec<HeaderConfig>> = HashMap::new();
            for h in header_rows {
                headers_map.entry(h.host_id).or_default().push(HeaderConfig {
                    id: h.id,
                    name: h.name,
                    value: h.value,
                    target: h.target,
                });
            }

            let mut hosts = HashMap::new();
            for row in rows {
                let locs = locations_map.remove(&row.id).unwrap_or_default();
                let host_headers = headers_map.get(&row.id).cloned().unwrap_or_default();
                hosts.insert(
                    row.domain,
                    HostConfig {
                        id: row.id,
                        target: row.target,
                        scheme: row.scheme,
                        locations: locs,
                        ssl_forced: row.ssl_forced,
                        redirect_to: row.redirect_to,
                        redirect_status: row.redirect_status as u16,
                        access_list_id: row.access_list_id,
                        headers: host_headers,
                    },
                );
            }
            Ok(ProxyConfig {
                hosts,
                access_lists,
                headers: headers_map,
            })
        } else {
            Err("Failed to load initial configuration from DB".into())
        }
    }
}
