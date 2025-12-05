use crate::db::{self, DbPool};
use crate::state::AppState;
use std::error::Error;
use std::process::Command;
use std::sync::Arc;
use std::path::Path;
use tokio::fs;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct AcmeManager {
    state: Arc<AppState>,
    db_pool: DbPool,
    contact_email: String,
}

impl AcmeManager {
    pub fn new(state: Arc<AppState>, db_pool: DbPool, email: String) -> Self {
        Self { state, db_pool, contact_email: email }
    }

    /// Requests a certificate using Certbot CLI.
    /// If `provider_id` is provided, uses DNS-01 challenge.
    /// Otherwise, defaults to HTTP-01 challenge using --webroot.
    pub async fn request_certificate(&self, domain: &str, provider_id: Option<i64>) -> Result<(), Box<dyn Error>> {
        tracing::info!("🔐 Requesting certificate for {} via Certbot", domain);

        // 1. Prepare Certbot Arguments
        let mut args = vec![
            "certonly".to_string(),
            "-d".to_string(),
            domain.to_string(),
            "--email".to_string(),
            self.contact_email.clone(),
            "--agree-tos".to_string(),
            "--non-interactive".to_string(),
        ];

        // Temp file path holder to ensure we can delete it later
        let mut credentials_file_path: Option<String> = None;

        if let Some(pid) = provider_id {
            // --- DNS-01 Challenge ---
            let provider = db::get_dns_provider(&self.db_pool, pid).await?
                .ok_or("DNS Provider not found")?;
            
            tracing::info!("👉 Using DNS Provider: {} ({})", provider.name, provider.provider_type);

            // Create temporary credentials file
            let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            let temp_path = format!("/tmp/dns-creds-{}-{}.ini", provider.provider_type, now);
            
            // Write credentials to file
            // Ensure the content is trimmed and valid INI format
            fs::write(&temp_path, provider.credentials.trim()).await?;
            
            // Set permissions to 600 (required by Certbot plugins)
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&temp_path).await?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&temp_path, perms).await?;

            credentials_file_path = Some(temp_path.clone());

            // Add provider-specific arguments
            match provider.provider_type.as_str() {
                "cloudflare" => {
                    args.push("--dns-cloudflare".to_string());
                    args.push("--dns-cloudflare-credentials".to_string());
                    args.push(temp_path);
                    // Optional: propagation seconds
                    args.push("--dns-cloudflare-propagation-seconds".to_string());
                    args.push("30".to_string());
                },
                "route53" => {
                    args.push("--dns-route53".to_string());
                    // Route53 plugin typically uses AWS env vars or ~/.aws/config, 
                    // but can technically take a config file if we set AWS_CONFIG_FILE env var.
                    // For simplicity here, we assume the plugin might use standard AWS setup, 
                    // or we implement a more complex env var injection wrapper later.
                    // BUT, certbot-dns-route53 doesn't have a simple --credentials flag like Cloudflare.
                    // It usually relies on environment variables.
                    // For now, let's warn if it's not fully supported in this generic implementation.
                    tracing::warn!("⚠️ Route53 support is experimental. Ensure AWS credentials are set in environment.");
                },
                "digitalocean" => {
                    args.push("--dns-digitalocean".to_string());
                    args.push("--dns-digitalocean-credentials".to_string());
                    args.push(temp_path);
                },
                "google" => {
                    args.push("--dns-google".to_string());
                    args.push("--dns-google-credentials".to_string());
                    args.push(temp_path);
                },
                _ => {
                    return Err(format!("Unsupported provider type: {}", provider.provider_type).into());
                }
            }

        } else {
            // --- HTTP-01 Challenge ---
            tracing::info!("👉 Using HTTP-01 (Webroot)");
            let webroot_path = "/app/data/acme-challenge";
            fs::create_dir_all(webroot_path).await?;
            
            args.push("--webroot".to_string());
            args.push("-w".to_string());
            args.push(webroot_path.to_string());
        }

        // 2. Execute Certbot
        tracing::info!("🚀 Running Certbot: certbot {}", args.join(" "));
        
        // Note: Command::new doesn't take Vec directly for args, so we iterate
        let mut cmd = Command::new("certbot");
        cmd.args(&args);

        // Capture output
        let output = cmd.output()?; // Blocking call (consider tokio::process::Command in pure async app)

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
        // Certbot saves to /etc/letsencrypt/live/<domain>/
        // Note: If domain contains wildcards (*.example.com), the cert directory name might be 'example.com'
        // We need to handle this mapping. Certbot usually names the dir after the first domain (-d).
        // Since we pass only one domain, it should match.
        // BUT for wildcard '*.example.com', the dir is usually 'example.com'.
        // Let's try to find the directory.
        let clean_domain = domain.replace("*.", "");
        let cert_base_path = Path::new("/etc/letsencrypt/live").join(&clean_domain);
        
        // If not found, try exact domain match (just in case)
        let cert_base_path = if !cert_base_path.exists() {
             Path::new("/etc/letsencrypt/live").join(domain)
        } else {
            cert_base_path
        };

        let privkey_path = cert_base_path.join("privkey.pem");
        let fullchain_path = cert_base_path.join("fullchain.pem");

        if !privkey_path.exists() || !fullchain_path.exists() {
            return Err(format!("Certificates not found at {:?}", cert_base_path).into());
        }

        // 5. Copy certificates to our local data directory
        let local_cert_dir = Path::new("data/certs");
        if !local_cert_dir.exists() {
            fs::create_dir_all(local_cert_dir).await?;
        }

        // Use the original requested domain name for our local filename to match DB
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
