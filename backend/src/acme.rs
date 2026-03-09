use crate::db::{self, DbPool};
use crate::state::AppState;
use std::error::Error;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs;
use tokio::process::Command; // Import for setting file permissions

pub const HTTP01_WEBROOT: &str = "data/acme-challenge";
pub const HTTP01_CHALLENGE_PREFIX: &str = "/.well-known/acme-challenge/";

pub fn is_valid_http01_token(token: &str) -> bool {
    !token.is_empty() && !token.contains("..") && !token.contains('/') && !token.contains('\\')
}

pub fn http01_token_path(token: &str) -> Option<PathBuf> {
    if !is_valid_http01_token(token) {
        return None;
    }

    Some(
        Path::new(HTTP01_WEBROOT)
            .join(".well-known")
            .join("acme-challenge")
            .join(token),
    )
}

pub struct AcmeManager {
    state: Arc<AppState>,
    db_pool: DbPool,
    contact_email: String,
}

impl AcmeManager {
    pub fn new(state: Arc<AppState>, db_pool: DbPool, email: String) -> Self {
        Self {
            state,
            db_pool,
            contact_email: email,
        }
    }

    /// Requests a certificate using Certbot CLI.
    /// If `provider_id` is provided, uses DNS-01 challenge.
    /// Otherwise, defaults to HTTP-01 challenge using --webroot.
    pub async fn request_certificate(
        &self,
        domain: &str,
        provider_id: Option<i64>,
    ) -> Result<(), Box<dyn Error>> {
        tracing::info!("🔐 Requesting certificate for {} via Certbot", domain);

        let mut cmd = Command::new("certbot");

        // Common Certbot arguments
        cmd.arg("certonly")
            .arg("-d")
            .arg(domain)
            .arg("--email")
            .arg(&self.contact_email)
            .arg("--agree-tos")
            .arg("--non-interactive");

        // Temp file path holder for credentials, to ensure it's deleted
        let mut credentials_file_path: Option<String> = None;

        if let Some(pid) = provider_id {
            // --- DNS-01 Challenge ---
            let provider = db::get_dns_provider(&self.db_pool, pid)
                .await?
                .ok_or("DNS Provider not found")?;

            tracing::info!(
                "👉 Using DNS Provider: {} ({})",
                provider.name,
                provider.provider_type
            );

            // Create temporary credentials file
            let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            let temp_path = format!("/tmp/dns-creds-{}-{}.ini", provider.provider_type, now);

            // Write credentials to file
            fs::write(&temp_path, provider.credentials.trim()).await?;

            // Set permissions to 600 (required by Certbot DNS plugins)
            let mut perms = fs::metadata(&temp_path).await?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&temp_path, perms).await?;

            credentials_file_path = Some(temp_path.clone());

            // Add provider-specific arguments
            match provider.provider_type.as_str() {
                "cloudflare" => {
                    cmd.arg("--dns-cloudflare")
                        .arg("--dns-cloudflare-credentials")
                        .arg(&temp_path)
                        .arg("--dns-cloudflare-propagation-seconds")
                        .arg("30");
                }
                "route53" => {
                    cmd.arg("--dns-route53")
                        .env("AWS_SHARED_CREDENTIALS_FILE", &temp_path); // Use env var for Route53
                    tracing::warn!("⚠️ Route53: AWS credentials loaded from temp file via environment variable. Ensure file content is in AWS credentials format.");
                }
                "digitalocean" => {
                    cmd.arg("--dns-digitalocean")
                        .arg("--dns-digitalocean-credentials")
                        .arg(&temp_path);
                }
                "google" => {
                    cmd.arg("--dns-google")
                        .arg("--dns-google-credentials")
                        .arg(&temp_path);
                }
                "hetzner" => {
                    cmd.arg("--authenticator")
                        .arg("dns-hetzner")
                        .arg("--dns-hetzner-credentials")
                        .arg(&temp_path)
                        .arg("--dns-hetzner-propagation-seconds")
                        .arg("60");
                }
                _ => {
                    return Err(
                        format!("Unsupported provider type: {}", provider.provider_type).into(),
                    );
                }
            }
        } else {
            // --- HTTP-01 Challenge ---
            tracing::info!("👉 Using HTTP-01 (Webroot)");
            let webroot_path = HTTP01_WEBROOT;
            fs::create_dir_all(webroot_path).await?;

            cmd.arg("--webroot").arg("-w").arg(webroot_path);
        }

        // 2. Execute Certbot
        tracing::info!("🚀 Running Certbot command for {}", domain);

        // Capture output (async)
        let output = cmd.output().await?;

        // 3. Cleanup Credentials File
        if let Some(path) = credentials_file_path {
            let _ = fs::remove_file(path).await; // Ignore deletion errors
        }

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::error!("❌ Certbot failed: {}", stderr);
            return Err(format!("Certbot failed: {}", stderr).into());
        }

        tracing::info!("✅ Certbot finished successfully.");

        // 4. Locate and Copy Certificates
        let clean_domain = domain.replace("*.", "");
        let cert_base_path_exact = Path::new("/etc/letsencrypt/live").join(domain);
        let cert_base_path_wildcard = Path::new("/etc/letsencrypt/live").join(&clean_domain);

        let cert_base_path = if fs::try_exists(&cert_base_path_exact).await.unwrap_or(false) {
            cert_base_path_exact
        } else if fs::try_exists(&cert_base_path_wildcard)
            .await
            .unwrap_or(false)
        {
            cert_base_path_wildcard
        } else {
            return Err(format!(
                "Certificates directory not found for {} at {:?} or {:?}",
                domain, cert_base_path_exact, cert_base_path_wildcard
            )
            .into());
        };

        let privkey_path = cert_base_path.join("privkey.pem");
        let fullchain_path = cert_base_path.join("fullchain.pem");

        if !fs::try_exists(&privkey_path).await.unwrap_or(false)
            || !fs::try_exists(&fullchain_path).await.unwrap_or(false)
        {
            return Err(format!("Certificates not found at {:?}", cert_base_path).into());
        }

        // 5. Copy certificates to our local data directory
        let local_cert_dir = Path::new("data/certs");
        if !fs::try_exists(local_cert_dir).await.unwrap_or(false) {
            fs::create_dir_all(local_cert_dir).await?;
        }

        let local_key_path = local_cert_dir.join(format!("{}.key", domain));
        let local_cert_path = local_cert_dir.join(format!("{}.crt", domain));

        fs::copy(&privkey_path, &local_key_path).await?;
        fs::copy(&fullchain_path, &local_cert_path).await?;

        tracing::info!("💾 Certificates copied to {:?}", local_cert_dir);

        // 6. Update Database with Expiration Date
        let cert_content = fs::read(&local_cert_path).await?;
        if let Ok((_, pem)) = x509_parser::pem::parse_x509_pem(&cert_content) {
            if let Ok(cert) = pem.parse_x509() {
                let expires_at = cert.validity().not_after.timestamp();
                db::upsert_cert(&self.db_pool, domain, expires_at, provider_id).await?;
                tracing::info!("📅 Certificate expiration updated in DB: {}", expires_at);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{http01_token_path, is_valid_http01_token, HTTP01_WEBROOT};
    use std::path::Path;

    #[test]
    fn maps_http01_token_to_certbot_webroot_layout() {
        let path = http01_token_path("token-123").expect("valid token path");

        assert_eq!(
            path,
            Path::new(HTTP01_WEBROOT)
                .join(".well-known")
                .join("acme-challenge")
                .join("token-123")
        );
    }

    #[test]
    fn rejects_invalid_http01_tokens() {
        for token in ["", "../token", "nested/token", r"nested\\token"] {
            assert!(
                !is_valid_http01_token(token),
                "token should be invalid: {token}"
            );
            assert!(
                http01_token_path(token).is_none(),
                "path should be rejected: {token}"
            );
        }
    }

    #[test]
    fn accepts_single_segment_http01_tokens() {
        assert!(is_valid_http01_token("abcDEF_123-xyz"));
        assert!(http01_token_path("abcDEF_123-xyz").is_some());
    }
}
