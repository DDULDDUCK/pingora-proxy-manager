use super::ProxyCtx;
use async_trait::async_trait;
use pingora::prelude::*;

pub mod acl;
pub mod acme;
pub mod redirect;
pub mod ssl;
pub mod trusted_proxy;

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
