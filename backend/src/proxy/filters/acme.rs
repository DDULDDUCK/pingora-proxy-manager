use async_trait::async_trait;
use bytes::Bytes;
use pingora::prelude::*;
use pingora::http::ResponseHeader;
use std::path::Path;
use tokio::fs;
use crate::constants;
use super::{ProxyFilter, FilterResult, ProxyCtx};

pub struct AcmeFilter;

#[async_trait]
impl ProxyFilter for AcmeFilter {
    async fn request_filter(
        &self,
        session: &mut Session,
        _ctx: &mut ProxyCtx,
    ) -> Result<FilterResult> {
        let path = session.req_header().uri.path();

        if path.starts_with("/.well-known/acme-challenge/") {
            let token = path.trim_start_matches("/.well-known/acme-challenge/");
            
            // Use a relative path from the current working directory (which is usually the project root or where binary is run)
            // instead of a hardcoded absolute path like "/app/data/acme-challenge".
            // This makes it work on both local dev (./data) and Docker (WORKDIR /app -> ./data).
            let file_path = Path::new("data/acme-challenge").join(token);

            // Security: Prevent directory traversal attacks
            if token.contains("..") || token.contains('/') || token.contains('\\') {
                tracing::warn!("⚠️ Attempted directory traversal in ACME challenge: {}", token);
                let _ = session.respond_error(constants::http::FORBIDDEN).await;
                return Ok(FilterResult::Handled);
            }

            match fs::read(&file_path).await {
                Ok(content) => {
                    let mut header = match ResponseHeader::build(constants::http::OK, Some(4)) {
                        Ok(h) => h,
                        Err(e) => {
                            tracing::error!("Failed to build ACME response header: {}", e);
                            let _ = session.respond_error(constants::http::INTERNAL_ERROR).await;
                            return Ok(FilterResult::Handled);
                        }
                    };
                    if let Err(e) = header.insert_header("Content-Type", "text/plain") {
                        tracing::error!("Failed to insert Content-Type header: {}", e);
                        let _ = session.respond_error(constants::http::INTERNAL_ERROR).await;
                        return Ok(FilterResult::Handled);
                    }
                    if let Err(e) = header.insert_header("Content-Length", content.len().to_string()) {
                        tracing::error!("Failed to insert Content-Length header: {}", e);
                        let _ = session.respond_error(constants::http::INTERNAL_ERROR).await;
                        return Ok(FilterResult::Handled);
                    }

                    session.write_response_header(Box::new(header), false).await?;
                    session.write_response_body(Some(Bytes::from(content)), true).await?;
                    return Ok(FilterResult::Handled);
                }
                Err(_) => {
                    tracing::debug!("ACME challenge not found: {:?}", file_path);
                    let _ = session.respond_error(constants::http::NOT_FOUND).await;
                    return Ok(FilterResult::Handled);
                }
            }
        }

        Ok(FilterResult::Continue)
    }
}
