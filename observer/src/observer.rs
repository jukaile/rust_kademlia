use common_lib::kademlia::kademlia::KademliaNode;

pub struct Observer {
    pub node: KademliaNode,
}

impl Observer {
    pub fn new(port: u16) -> Self {
        Observer {
            node: KademliaNode::new(port),
        }
    }

    pub fn observe(&mut self, bootstrap_addr: &str) {
        println!("[observer] Bootstrapping into network...");
        self.node.bootstrap(bootstrap_addr);

        println!("[observer] Routing table after bootstrap:");
        self.print_routing_table();
    }

    pub fn print_routing_table(&self) {
        let routing = self.node.routing.lock().unwrap();
        for (i, bucket) in routing.buckets.iter().enumerate() {
            if !bucket.is_empty() {
                println!("Bucket {:03}: {} nodes", i, bucket.len());
                for (id, addr) in bucket {
                    println!("  - {:?} @ {}", &id.0[..4], addr); // 打印前4字节
                }
            }
        }
    }
}
