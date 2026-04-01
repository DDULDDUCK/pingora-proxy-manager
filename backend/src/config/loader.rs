use crate::db::{self, DbPool};
use crate::state::{
    AccessListClientConfig, AccessListConfig, AccessListIpConfig, HeaderConfig, HostConfig,
    LocationConfig, ProxyConfig,
};
use std::collections::HashMap;

fn to_u64_opt(value: Option<i64>) -> Option<u64> {
    value.and_then(|v| u64::try_from(v).ok())
}

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
                // Split comma-separated targets
                let targets: Vec<String> = loc
                    .target
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                locations_map
                    .entry(loc.host_id)
                    .or_default()
                    .push(LocationConfig {
                        path: loc.path,
                        targets, // Use Vector
                        scheme: loc.scheme,
                        rewrite: loc.rewrite,
                        verify_ssl: loc.verify_ssl,
                        upstream_sni: loc.upstream_sni,
                        connection_timeout_ms: to_u64_opt(loc.connection_timeout_ms),
                        read_timeout_ms: to_u64_opt(loc.read_timeout_ms),
                        write_timeout_ms: to_u64_opt(loc.write_timeout_ms),
                        max_request_body_bytes: to_u64_opt(loc.max_request_body_bytes),
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
                ips_map
                    .entry(ip.list_id)
                    .or_default()
                    .push(AccessListIpConfig {
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
                headers_map
                    .entry(h.host_id)
                    .or_default()
                    .push(HeaderConfig {
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

                // Split comma-separated targets
                let targets: Vec<String> = row
                    .target
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                hosts.insert(
                    row.domain,
                    HostConfig {
                        id: row.id,
                        targets, // Use Vector
                        scheme: row.scheme,
                        locations: locs,
                        ssl_forced: row.ssl_forced,
                        verify_ssl: row.verify_ssl,
                        upstream_sni: row.upstream_sni,
                        connection_timeout_ms: to_u64_opt(row.connection_timeout_ms),
                        read_timeout_ms: to_u64_opt(row.read_timeout_ms),
                        write_timeout_ms: to_u64_opt(row.write_timeout_ms),
                        max_request_body_bytes: to_u64_opt(row.max_request_body_bytes),
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

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::{
        sqlite::{SqliteConnectOptions, SqlitePoolOptions},
        Row,
    };
    use std::str::FromStr;
    use tempfile::tempdir;

    #[tokio::test]
    async fn init_db_migrates_legacy_schema_and_preserves_nullable_advanced_config() {
        let temp_dir = tempdir().expect("create temp dir");
        let db_path = temp_dir.path().join("legacy.db");
        let connect_options = SqliteConnectOptions::from_str(&db_path.display().to_string())
            .expect("sqlite connect options")
            .create_if_missing(true);
        let legacy_pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(connect_options)
            .await
            .expect("connect legacy db");

        sqlx::query(
            r#"
            CREATE TABLE hosts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                domain TEXT NOT NULL UNIQUE,
                target TEXT NOT NULL,
                scheme TEXT NOT NULL DEFAULT 'http',
                ssl_forced BOOLEAN NOT NULL DEFAULT 0,
                redirect_to TEXT,
                redirect_status INTEGER NOT NULL DEFAULT 301
            )
            "#,
        )
        .execute(&legacy_pool)
        .await
        .expect("create legacy hosts");

        sqlx::query(
            r#"
            CREATE TABLE locations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                host_id INTEGER NOT NULL,
                path TEXT NOT NULL,
                target TEXT NOT NULL,
                scheme TEXT NOT NULL DEFAULT 'http',
                rewrite BOOLEAN NOT NULL DEFAULT 0,
                FOREIGN KEY(host_id) REFERENCES hosts(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&legacy_pool)
        .await
        .expect("create legacy locations");

        sqlx::query("INSERT INTO hosts (domain, target, scheme, ssl_forced, redirect_to, redirect_status) VALUES (?, ?, ?, ?, ?, ?)")
            .bind("legacy.local")
            .bind("127.0.0.1:8080")
            .bind("http")
            .bind(false)
            .bind(None::<String>)
            .bind(301_i64)
            .execute(&legacy_pool)
            .await
            .expect("insert legacy host");

        sqlx::query(
            "INSERT INTO locations (host_id, path, target, scheme, rewrite) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(1_i64)
        .bind("/api")
        .bind("127.0.0.1:8081")
        .bind("http")
        .bind(false)
        .execute(&legacy_pool)
        .await
        .expect("insert legacy location");

        drop(legacy_pool);

        let db_url = format!("sqlite://{}", db_path.display());
        let pool = db::init_db(&db_url).await.expect("migrate db");
        let config = ConfigLoader::load_from_db(&pool)
            .await
            .expect("load migrated config");

        let host = config
            .hosts
            .get("legacy.local")
            .expect("legacy host should exist");
        assert_eq!(host.targets, vec!["127.0.0.1:8080"]);
        assert_eq!(host.connection_timeout_ms, None);
        assert_eq!(host.read_timeout_ms, None);
        assert_eq!(host.write_timeout_ms, None);
        assert_eq!(host.max_request_body_bytes, None);
        assert!(
            host.verify_ssl,
            "legacy host should keep verify_ssl default"
        );

        let location = host
            .locations
            .iter()
            .find(|location| location.path == "/api")
            .expect("legacy location should exist");
        assert_eq!(location.targets, vec!["127.0.0.1:8081"]);
        assert_eq!(location.connection_timeout_ms, None);
        assert_eq!(location.read_timeout_ms, None);
        assert_eq!(location.write_timeout_ms, None);
        assert_eq!(location.max_request_body_bytes, None);
        assert!(
            location.verify_ssl,
            "legacy location should keep verify_ssl default"
        );

        let host_columns = sqlx::query("PRAGMA table_info(hosts)")
            .fetch_all(&pool)
            .await
            .expect("fetch host table info");
        let location_columns = sqlx::query("PRAGMA table_info(locations)")
            .fetch_all(&pool)
            .await
            .expect("fetch location table info");

        for expected_column in [
            "connection_timeout_ms",
            "read_timeout_ms",
            "write_timeout_ms",
            "max_request_body_bytes",
        ] {
            assert!(
                host_columns
                    .iter()
                    .any(|row| row.get::<String, _>("name") == expected_column),
                "hosts table should contain {expected_column}"
            );
            assert!(
                location_columns
                    .iter()
                    .any(|row| row.get::<String, _>("name") == expected_column),
                "locations table should contain {expected_column}"
            );
        }
    }
}
