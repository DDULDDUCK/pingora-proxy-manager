use async_trait::async_trait;
use pingora::prelude::*;
use pingora::http::ResponseHeader;
use super::{ProxyFilter, FilterResult, ProxyCtx};

pub struct SslFilter;

#[async_trait]
impl ProxyFilter for SslFilter {
    async fn request_filter(
        &self,
        session: &mut Session,
        ctx: &mut ProxyCtx,
    ) -> Result<FilterResult> {
        if let Some(host_config) = &ctx.host_config {
            let is_tls = session
                .server_addr()
                .map(|a| a.as_inet().map(|s| s.port() == 443).unwrap_or(false))
                .unwrap_or(false);

            if host_config.ssl_forced && !is_tls {
                let path = session.req_header().uri.path();
                let query = session
                    .req_header()
                    .uri
                    .query()
                    .map(|q| format!("?{}", q))
                    .unwrap_or_default();
                let new_url = format!("https://{}{}{}", ctx.host, path, query);

                let mut header = ResponseHeader::build(301, Some(4))
                    .map_err(|e| Error::explain(ErrorType::InternalError, format!("Failed to build 301 header: {}", e)))?;
                header.insert_header("Location", new_url)
                    .map_err(|e| Error::explain(ErrorType::InternalError, format!("Failed to insert Location header: {}", e)))?;
                header.insert_header("Content-Length", "0")
                    .map_err(|e| Error::explain(ErrorType::InternalError, format!("Failed to insert Content-Length header: {}", e)))?;

                session.write_response_header(Box::new(header), true).await?;
                return Ok(FilterResult::Handled);
            }
        }
        Ok(FilterResult::Continue)
    }
}
