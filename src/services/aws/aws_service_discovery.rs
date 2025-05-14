use async_trait::async_trait;
use pingora::lb::discovery::ServiceDiscovery;
use pingora::lb::{Backend, Extensions};
use pingora::prelude::*;
use pingora::protocols::l4::socket::SocketAddr;
use std::collections::{BTreeSet, HashMap};

pub struct AWSServiceDiscovery;
#[async_trait]

impl ServiceDiscovery for AWSServiceDiscovery {
    async fn discover(&self) -> Result<(BTreeSet<Backend>, HashMap<u64, bool>)> {
        // Implement logic to discover upstreams, e.g., read from a file or query a service registry
        let backends = vec!["localhost:3000", "localhost:3001"]
            .into_iter()
            .map(|x| x.parse::<SocketAddr>().unwrap())
            .map(|addr| Backend {
                addr,
                weight: 1,
                ext: Extensions::new(),
            })
            .collect::<BTreeSet<_>>();

        Ok((backends, HashMap::new()))
    }
}
