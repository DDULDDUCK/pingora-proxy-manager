use super::{FilterResult, ProxyCtx, ProxyFilter};
use crate::constants;
use async_trait::async_trait;
use pingora::http::ResponseHeader;
use pingora::prelude::*;

pub struct RedirectFilter;

#[async_trait]
impl ProxyFilter for RedirectFilter {
    async fn request_filter(
        &self,
        session: &mut Session,
        ctx: &mut ProxyCtx,
    ) -> Result<FilterResult> {
        if let Some(host_config) = &ctx.host_config {
            if let Some(redirect_target) = &host_config.redirect_to {
                let status = host_config.redirect_status;
                let path = session.req_header().uri.path();
                let query = session
                    .req_header()
                    .uri
                    .query()
                    .map(|q| format!("?{}", q))
                    .unwrap_or_default();

                let target = if redirect_target.ends_with('/') && path.starts_with('/') {
                    &redirect_target[..redirect_target.len() - 1]
                } else {
                    redirect_target
                };

                let new_url = format!("{}{}{}", target, path, query);
                let mut header = match ResponseHeader::build(status, Some(4)) {
                    Ok(h) => h,
                    Err(e) => {
                        tracing::error!(
                            "Failed to build redirect response header for {}: {}",
                            ctx.host,
                            e
                        );
                        let _ = session.respond_error(constants::http::INTERNAL_ERROR).await;
                        return Ok(FilterResult::Handled);
                    }
                };
                if let Err(e) = header.insert_header("Location", new_url.clone()) {
                    tracing::error!("Failed to insert Location header for {}: {}", ctx.host, e);
                    let _ = session.respond_error(constants::http::INTERNAL_ERROR).await;
                    return Ok(FilterResult::Handled);
                }
                if let Err(e) = header.insert_header("Content-Length", "0") {
                    tracing::error!(
                        "Failed to insert Content-Length header for {}: {}",
                        ctx.host,
                        e
                    );
                    let _ = session.respond_error(constants::http::INTERNAL_ERROR).await;
                    return Ok(FilterResult::Handled);
                }

                session
                    .write_response_header(Box::new(header), true)
                    .await?;
                return Ok(FilterResult::Handled);
            }
        }
        Ok(FilterResult::Continue)
    }
}
