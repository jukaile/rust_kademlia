use std::sync::Arc;

use common_lib::kademlia::KademliaNode;

fn main() {
    let mut args = std::env::args().skip(1);

    let port = args
        .next()
        .expect("Usage: node <listen_port> [bootstrap_addr]")
        .parse::<u16>()
        .expect("Invalid port number");

    let bootstrap = args.next();

    let mut node = KademliaNode::new(port);
    // let mut node = KademliaNode::new(9000);
    // 启动服务器
    node.start_server();

    // 启动维护线程
    node.start_maintenance();

    // 启动路由表监控线程
    node.start_monitoring(30);

    if let Some(addr) = bootstrap {
        node.bootstrap(&addr);
    }

    let node = Arc::new(node);
    node.start_bucket_maintenance();

    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}