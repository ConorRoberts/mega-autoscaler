mod lb;
mod services;

use env_logger;
use lb::LB;
use pingora::lb::{Backends, LoadBalancer};
use pingora::prelude::*;
use services::aws::aws_service_discovery::AWSServiceDiscovery;
use std::time::Duration;

fn main() {
    env_logger::init();

    let opt = Opt::default();

    let mut my_server = Server::new(Some(opt)).unwrap();
    my_server.bootstrap();

    let disc = AWSServiceDiscovery::new();
    let backends = Backends::new(Box::new(disc));

    // let upstreams: Vec<&'static str> = vec![];
    // let mut load_balancer = LoadBalancer::try_from_iter(upstreams).unwrap();
    let mut load_balancer = LoadBalancer::from_backends(backends);

    let hc = TcpHealthCheck::new();
    load_balancer.set_health_check(hc);
    load_balancer.health_check_frequency = Some(std::time::Duration::from_secs(1));
    load_balancer.update_frequency = Some(Duration::from_secs(5));

    let disc_background = background_service("discovery", load_balancer);
    let disc_task = disc_background.task();

    let mut lb = http_proxy_service(&my_server.configuration, LB(disc_task));
    lb.add_tcp("0.0.0.0:6188");

    my_server.add_service(disc_background);

    my_server.add_service(lb);

    my_server.run_forever();
}
