use async_trait::async_trait;
use log::info;
use pingora::prelude::*;
use std::sync::Arc;

/// The public host at which the load balancer can be reached.
static SNI: &str = "0.0.0.0:6188";

/// Use TLS or not.
static USE_TLS: bool = false;

pub struct LB(pub Arc<LoadBalancer<RoundRobin>>);

#[async_trait]
impl ProxyHttp for LB {
    /// For this small example, we don't need context storage
    type CTX = ();
    fn new_ctx(&self) -> () {
        ()
    }

    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        upstream_request: &mut RequestHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()>
    where
        Self::CTX: Send + Sync,
    {
        upstream_request
            .insert_header("Host", "one.one.one.one")
            .unwrap();

        Ok(())
    }

    async fn upstream_peer(&self, _session: &mut Session, _ctx: &mut ()) -> Result<Box<HttpPeer>> {
        let bs = self.0.backends().get_backend();

        // if bs.is_empty() {
        //     self.0.update().await.unwrap()
        // }

        info!("{:?}", bs.len());

        let upstream = self
            .0
            .select(b"", 256) // hash doesn't matter for round robin
            .unwrap();

        println!("upstream peer is: {upstream:?}");

        let peer = Box::new(HttpPeer::new(upstream, USE_TLS, SNI.to_string()));
        Ok(peer)
    }
}
