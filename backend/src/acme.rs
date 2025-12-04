use crate::db::{self, DbPool};
use crate::state::AppState;
use instant_acme::{Account, ChallengeType, Identifier, NewAccount, NewOrder, OrderStatus, RetryPolicy};
use std::error::Error;
use std::sync::Arc;
use std::path::Path;
use tokio::fs;

// Production (실제 인증서)
const LETS_ENCRYPT_PRODUCTION_URL: &str = "https://acme-v02.api.letsencrypt.org/directory";
// Staging (테스트용)
// const LETS_ENCRYPT_STAGING_URL: &str = "https://acme-staging-v02.api.letsencrypt.org/directory";

pub struct AcmeManager {
    state: Arc<AppState>,
    db_pool: DbPool,
    contact_email: String,
}

impl AcmeManager {
    pub fn new(state: Arc<AppState>, db_pool: DbPool, email: String) -> Self {
        Self { state, db_pool, contact_email: email }
    }

    pub async fn request_certificate(&self, domain: &str) -> Result<(), Box<dyn Error>> {
        tracing::info!("🔐 Requesting certificate for {}", domain);

        // 1. 계정 생성
        let (account, _) = Account::builder()? 
            .create(
                &NewAccount {
                    contact: &[&format!("mailto:{}", self.contact_email)],
                    terms_of_service_agreed: true,
                    only_return_existing: false,
                },
                LETS_ENCRYPT_PRODUCTION_URL.to_string(),
                None,
            )
            .await?;

        // 2. 주문 생성
        let mut order = account
            .new_order(&NewOrder::new(&[Identifier::Dns(domain.to_string())]))
            .await?;

        // 3. Authorizations 처리 (Challenge 설정)
        let mut auths = order.authorizations();
        while let Some(auth_result) = auths.next().await {
            let mut auth = auth_result?;
            
            if let Some(mut challenge) = auth.challenge(ChallengeType::Http01) {
                let token = challenge.token.to_string();
                let key_auth = challenge.key_authorization();
                let key_auth_str = key_auth.as_str();

                tracing::info!("📝 Setting ACME challenge: {} -> {}", token, key_auth_str);
                self.state.add_acme_challenge(token.clone(), key_auth_str.to_string());

                // Let's Encrypt에게 검증 요청
                challenge.set_ready().await?;
            }
        }

        // 4. 검증 대기 (Order Ready 상태 될 때까지)
        let state = order.poll_ready(&RetryPolicy::new()).await?;
        if state != OrderStatus::Ready {
            return Err(format!("Order failed to become ready: {:?}", state).into());
        }

        // 5. Finalize (Private Key 생성 및 CSR 전송)
        tracing::info!("🔑 Generating Private Key and Finalizing Order...");
        let private_key_pem = order.finalize().await?;

        // 6. 인증서 다운로드 대기
        tracing::info!("⬇️ Downloading Certificate...");
        let cert_chain_pem = order.poll_certificate(&RetryPolicy::new()).await?;

        // 7. 인증서 저장 (파일 시스템 & DB)
        let cert_dir = Path::new("data/certs");
        if !cert_dir.exists() {
            fs::create_dir_all(cert_dir).await?;
        }

        let key_path = cert_dir.join(format!("{}.key", domain));
        let cert_path = cert_dir.join(format!("{}.crt", domain));

        fs::write(&key_path, &private_key_pem).await?;
        fs::write(&cert_path, &cert_chain_pem).await?;
        
        tracing::info!("💾 Certificates saved to {:?}", cert_dir);

        // 8. DB에 만료일 업데이트
        // 인증서 파싱해서 만료일 알아내야 함 (x509-parser 사용)
        // 여기서는 간단히 현재시간 + 90일로 가정하거나, 실제 파싱 로직 추가
        // x509-parser가 있으므로 파싱 시도
        if let Ok((_, pem)) = x509_parser::pem::parse_x509_pem(cert_chain_pem.as_bytes()) {
             if let Ok(cert) = pem.parse_x509() {
                 let expires_at = cert.validity().not_after.timestamp();
                 db::upsert_cert(&self.db_pool, domain, expires_at).await?;
                 tracing::info!("📅 Certificate expiration updated in DB: {}", expires_at);
             }
        }

        tracing::info!("✅ Certificate issued successfully for {}!", domain);
        
        Ok(())
    }
}

