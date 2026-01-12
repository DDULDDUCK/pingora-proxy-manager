use async_trait::async_trait;
use pingora::prelude::*;
use super::ProxyCtx;

pub mod acme;
pub mod acl;
pub mod redirect;
pub mod ssl;

#[derive(Debug)]
pub enum FilterResult {
    Handled,
    Continue,
}

#[async_trait]
pub trait ProxyFilter: Send + Sync {
    async fn request_filter(
        &self,
        session: &mut Session,
        ctx: &mut ProxyCtx,
    ) -> Result<FilterResult>;
}
