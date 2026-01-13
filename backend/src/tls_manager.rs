use crate::constants;
use async_trait::async_trait;
use openssl::asn1::Asn1Time;
use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use openssl::rsa::Rsa;
use openssl::x509::X509;
use parking_lot::RwLock;
use pingora::listeners::TlsAccept;
use pingora::tls::ssl::{NameType, SslRef};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::fs as tokio_fs;

/// 도메인별 인증서를 동적으로 로드하는 TLS 관리자
/// SNI(Server Name Indication)를 기반으로 적절한 인증서를 선택합니다.
pub struct DynamicCertManager {
    /// 인증서 캐시: 도메인 -> (인증서 PEM, 키 PEM)
    cert_cache: Arc<RwLock<HashMap<String, CertKeyPair>>>,
    /// 인증서 디렉토리 경로
    cert_dir: String,
    /// 디폴트 인증서 (SNI가 없거나 인증서가 없는 경우)
    default_cert: CertKeyPair,
}

#[derive(Clone)]
struct CertKeyPair {
    cert_pem: Vec<u8>,
    key_pem: Vec<u8>,
}

/// 디폴트 인증서가 존재하는지 확인하고, 없으면 새로 생성합니다.
fn ensure_default_cert(cert_path: &str, key_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let cert_path = Path::new(cert_path);
    let key_path = Path::new(key_path);

    if cert_path.exists() && key_path.exists() {
        return Ok(());
    }

    tracing::warn!("⚠️ Default certificate not found. Generating a new self-signed certificate...");

    // 디렉토리 생성
    if let Some(parent) = cert_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // 1. RSA 키 생성
    let rsa = Rsa::generate(constants::cert::RSA_BITS)?;
    let pkey = PKey::from_rsa(rsa)?;

    // 2. X509 인증서 생성
    let mut x509 = X509::builder()?;
    x509.set_version(2)?;
    x509.set_pubkey(&pkey)?;

    let not_before = Asn1Time::days_from_now(0)?;
    let not_after = Asn1Time::days_from_now(constants::cert::VALIDITY_DAYS as u32)?; // 10년 유효
    x509.set_not_before(&not_before)?;
    x509.set_not_after(&not_after)?;

    // Subject 설정
    let mut name = openssl::x509::X509Name::builder()?;
    name.append_entry_by_text("CN", "Pingora Proxy Manager Default")?;
    let name = name.build();
    x509.set_subject_name(&name)?;
    x509.set_issuer_name(&name)?;

    // 서명
    x509.sign(&pkey, MessageDigest::sha256())?;

    let cert_pem = x509.build().to_pem()?;
    let key_pem = pkey.private_key_to_pem_pkcs8()?;

    // 파일 저장
    fs::write(cert_path, cert_pem)?;
    fs::write(key_path, key_pem)?;

    tracing::info!("✅ Generated new default certificate at {:?}", cert_path);

    Ok(())
}

impl DynamicCertManager {
    /// 새로운 DynamicCertManager를 생성합니다.
    ///
    /// # Arguments
    /// * `cert_dir` - 인증서가 저장된 디렉토리 경로 (예: "data/certs")
    /// * `default_cert_path` - 디폴트 인증서 경로
    /// * `default_key_path` - 디폴트 키 경로
    pub fn new(
        cert_dir: &str,
        default_cert_path: &str,
        default_key_path: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // 디폴트 인증서 확인 및 생성
        ensure_default_cert(default_cert_path, default_key_path)?;

        let default_cert = CertKeyPair {
            cert_pem: fs::read(default_cert_path)?,
            key_pem: fs::read(default_key_path)?,
        };

        Ok(Self {
            cert_cache: Arc::new(RwLock::new(HashMap::new())),
            cert_dir: cert_dir.to_string(),
            default_cert,
        })
    }

    /// 인증서 캐시를 초기화합니다.
    /// 디렉토리에서 모든 .crt/.key 파일을 로드합니다.
    pub fn preload_certs(&self) -> Result<usize, Box<dyn std::error::Error>> {
        let cert_path = Path::new(&self.cert_dir);
        if !cert_path.exists() {
            return Ok(0);
        }

        let mut count = 0;
        let mut cache = self.cert_cache.write();

        for entry in fs::read_dir(cert_path)? {
            let entry = entry?;
            let path = entry.path();

            if let Some(ext) = path.extension() {
                if ext == "crt" {
                    if let Some(stem) = path.file_stem() {
                        let domain = stem.to_string_lossy().to_string();

                        // default 인증서는 스킵
                        if domain == "default" {
                            continue;
                        }

                        let key_path = cert_path.join(format!("{}.key", domain));

                        if key_path.exists() {
                            match (fs::read(&path), fs::read(&key_path)) {
                                (Ok(cert_pem), Ok(key_pem)) => {
                                    cache.insert(domain.clone(), CertKeyPair { cert_pem, key_pem });
                                    tracing::info!("🔐 Loaded certificate for: {}", domain);
                                    count += 1;
                                }
                                (Err(e), _) => {
                                    tracing::warn!("⚠️ Failed to load cert for {}: {}", domain, e);
                                }
                                (_, Err(e)) => {
                                    tracing::warn!("⚠️ Failed to load key for {}: {}", domain, e);
                                }
                            }
                        }
                    }
                }
            }
        }

        tracing::info!("✅ Preloaded {} certificates", count);
        Ok(count)
    }

    /// 특정 도메인의 인증서를 캐시에서 가져오거나 파일에서 로드합니다.
    async fn get_cert_for_domain(&self, domain: &str) -> CertKeyPair {
        // 1. 캐시에서 먼저 검색
        {
            let cache = self.cert_cache.read();
            if let Some(pair) = cache.get(domain) {
                return pair.clone();
            }
        }

        // 2. 파일에서 로드 시도
        let cert_path = Path::new(&self.cert_dir).join(format!("{}.crt", domain));
        let key_path = Path::new(&self.cert_dir).join(format!("{}.key", domain));

        if tokio_fs::try_exists(&cert_path).await.unwrap_or(false)
            && tokio_fs::try_exists(&key_path).await.unwrap_or(false)
        {
            if let (Ok(cert_pem), Ok(key_pem)) = (
                tokio_fs::read(&cert_path).await,
                tokio_fs::read(&key_path).await,
            ) {
                let pair = CertKeyPair {
                    cert_pem: cert_pem.clone(),
                    key_pem: key_pem.clone(),
                };

                // 캐시에 저장
                {
                    let mut cache = self.cert_cache.write();
                    cache.insert(domain.to_string(), CertKeyPair { cert_pem, key_pem });
                }

                tracing::info!("🔐 Dynamically loaded certificate for: {}", domain);
                return pair;
            }
        }

        // 3. 와일드카드 인증서 검색 (예: *.example.com)
        if let Some(parent_domain) = domain.split_once('.').map(|(_, parent)| parent) {
            let wildcard = format!("*.{}", parent_domain);

            {
                let cache = self.cert_cache.read();
                if let Some(pair) = cache.get(&wildcard) {
                    return pair.clone();
                }
            }

            let cert_path = Path::new(&self.cert_dir).join(format!("{}.crt", wildcard));
            let key_path = Path::new(&self.cert_dir).join(format!("{}.key", wildcard));

            if tokio_fs::try_exists(&cert_path).await.unwrap_or(false)
                && tokio_fs::try_exists(&key_path).await.unwrap_or(false)
            {
                if let (Ok(cert_pem), Ok(key_pem)) = (
                    tokio_fs::read(&cert_path).await,
                    tokio_fs::read(&key_path).await,
                ) {
                    let pair = CertKeyPair {
                        cert_pem: cert_pem.clone(),
                        key_pem: key_pem.clone(),
                    };

                    {
                        let mut cache = self.cert_cache.write();
                        cache.insert(wildcard.clone(), CertKeyPair { cert_pem, key_pem });
                    }

                    tracing::info!(
                        "🔐 Loaded wildcard certificate for: {} -> {}",
                        domain,
                        wildcard
                    );
                    return pair;
                }
            }
        }

        // 4. 디폴트 인증서 반환
        tracing::debug!("🔒 Using default certificate for: {}", domain);
        self.default_cert.clone()
    }

    /// 특정 도메인의 인증서 캐시를 무효화합니다.
    /// 인증서 갱신 후 호출해야 합니다.
    pub fn invalidate_cert(&self, domain: &str) {
        let mut cache = self.cert_cache.write();
        cache.remove(domain);
        tracing::info!("🔄 Certificate cache invalidated for: {}", domain);
    }

    /// 모든 인증서 캐시를 초기화합니다.
    pub fn clear_cache(&self) {
        let mut cache = self.cert_cache.write();
        cache.clear();
        tracing::info!("🔄 All certificate cache cleared");
    }
}

#[async_trait]
impl TlsAccept for DynamicCertManager {
    async fn certificate_callback(&self, ssl: &mut SslRef) {
        // 1. SNI에서 도메인 이름 추출
        let sni = ssl
            .servername(NameType::HOST_NAME)
            .unwrap_or("default")
            .to_string();

        tracing::debug!("🔍 TLS SNI callback for: {}", sni);

        // 2. 도메인에 맞는 인증서 가져오기
        let pair = self.get_cert_for_domain(&sni).await;

        // 3. X509 인증서와 키 파싱 및 적용
        match openssl::x509::X509::from_pem(&pair.cert_pem) {
            Ok(cert) => {
                if let Err(e) = ssl.set_certificate(&cert) {
                    tracing::error!("❌ Failed to set certificate for {}: {}", sni, e);
                }
            }
            Err(e) => {
                tracing::error!("❌ Failed to parse certificate for {}: {}", sni, e);
            }
        }

        match openssl::pkey::PKey::private_key_from_pem(&pair.key_pem) {
            Ok(key) => {
                if let Err(e) = ssl.set_private_key(&key) {
                    tracing::error!("❌ Failed to set private key for {}: {}", sni, e);
                }
            }
            Err(e) => {
                tracing::error!("❌ Failed to parse private key for {}: {}", sni, e);
            }
        }
    }
}

/// Arc 래퍼 타입 (Orphan rule 회피)
pub struct SharedCertManager(pub Arc<DynamicCertManager>);

impl SharedCertManager {
    pub fn new(manager: DynamicCertManager) -> Self {
        Self(Arc::new(manager))
    }

    pub fn inner(&self) -> &Arc<DynamicCertManager> {
        &self.0
    }
}

impl Clone for SharedCertManager {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

#[async_trait]
impl TlsAccept for SharedCertManager {
    async fn certificate_callback(&self, ssl: &mut SslRef) {
        self.0.certificate_callback(ssl).await
    }
}
