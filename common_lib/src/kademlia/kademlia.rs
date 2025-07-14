use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::kademlia::kad_id::{
    load_or_generate_node_id,
    generate_random_id_in_bucket,
    NodeId
};
use crate::kademlia::routing::RoutingTable;
use crate::kademlia::protocol::Message;
use crate::kademlia::network::{send_message, receive_message};

pub struct KademliaNode {
    pub id: NodeId,
    pub routing: Arc<Mutex<RoutingTable>>,
    pub port: u16,
}


impl KademliaNode {
    pub fn new(port: u16) -> Self {
        let id = load_or_generate_node_id(port);
        KademliaNode {
            id: id.clone(),
            routing: Arc::new(Mutex::new(RoutingTable::new(id.clone(), 8))),
            port,
        }
    }

    pub fn start_server(&mut self) {
        let routing = Arc::clone(&self.routing);
        let my_id = self.id.clone();
        let port = self.port;

        thread::spawn(move || {
            let listener = TcpListener::bind(("0.0.0.0", port)).unwrap();
            println!("Listening on port {}", port);

            for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                let peer_addr = stream.peer_addr().unwrap().to_string();
                let routing = Arc::clone(&routing);
                let my_id = my_id.clone();

                println!("Getting connection from {}", peer_addr);
                thread::spawn(move || {
                    if let Ok(msg) = receive_message(&mut stream) {
                        Self::handle_message(&my_id, port, &routing, &mut stream, msg, peer_addr);
                    }
                });
            }
        });
    }

    pub fn start_maintenance(&self) {
        let routing = Arc::clone(&self.routing);
        let my_id = self.id.clone();
        let my_port = self.port;

        std::thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_secs(60 / 2));

            let all_nodes = {
                let routing = routing.lock().unwrap();
                routing.all_nodes()
            };

            for (_, id, addr) in all_nodes {
                if id == my_id {
                    continue;
                }

                if let Ok(mut stream) = TcpStream::connect(&addr) {
                    // 发送 ping
                    let _ = send_message(&mut stream, &Message::Ping(my_id.clone(),my_port));
                    if let Ok(Message::Pong(_,_)) = receive_message(&mut stream) {
                        continue; // 节点在线
                    }
                }
                // 节点离线，尝试替换
                println!("[Maintenance] Node {} is offline, replacing...", id);
                let mut routing = routing.lock().unwrap();
                routing.substitute_or_remove_node(id.clone());
            }
        });
    }

    pub fn bootstrap(&mut self, addr: &str) {
        if let Ok(mut stream) = TcpStream::connect(addr) {
            send_message(&mut stream, &Message::Ping(self.id.clone(),self.port)).unwrap();
            if let Ok(msg) = receive_message(&mut stream) {
                println!("Bootstrap response: {:?}", msg);
            }
        }
        if let Ok(mut stream) = TcpStream::connect(addr) {
            send_message(&mut stream, &Message::FindNode(self.id.clone())).unwrap();
            if let Ok(msg) = receive_message(&mut stream) {
                if let Message::FoundNodes(nodes) = msg {
                    println!("Discovered {} nodes from bootstrap", nodes.len());
                    {
                        let mut routing = self.routing.lock().unwrap();
                        for (id, addr) in &nodes {
                            if *id != self.id {
                                routing.insert(id.clone(), addr.clone());
                            }
                        }
                    }
                    self.recursive_find_node(&self.id, 3); // 最多查找3轮
                }
            }
        }
    }

    pub fn start_bucket_maintenance(self: Arc<Self>) {
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_secs(60));
            for i in 0..160 {
                let routing = self.routing.lock().unwrap();
                if routing.buckets[i].is_empty() {
                    continue;
                }
                let last_used = routing.last_touched[i];
                drop(routing);

                if last_used.elapsed() > Duration::from_secs(60 * 30) {
                    let target_id = generate_random_id_in_bucket(&self.id, i);
                    println!("[Maintenance] Refreshing bucket {} with random target {}", i, target_id);
                    self.recursive_find_node(&target_id, 3);
                }
            }
        });
    }

    pub fn start_monitoring(&self, interval_secs: u64) {
        let routing = Arc::clone(&self.routing);

        std::thread::spawn(move || {
            use std::time::Duration;
            loop {
                {
                    let rt = routing.lock().unwrap();
                    let nodes = rt.all_nodes();
                    println!("--- Routing Table Snapshot ---");
                    for (i, id, addr) in nodes {
                        println!("Bucket index: {}, NodeId: {}, Addr: {}", i, id, addr);
                    }
                    println!("------------------------------");
                }
                std::thread::sleep(Duration::from_secs(interval_secs));
            }
        });
    }

    fn handle_message(
        my_id: &NodeId,
        my_port: u16,
        routing: &Arc<Mutex<RoutingTable>>,
        stream: &mut TcpStream,
        msg: Message,
        peer_addr: String,
    ) {
        match msg {
            Message::Ping(peer_id, peer_port) => {
                let insert_addr = if let Some(pos) = peer_addr.rfind(':') {
                    let peer_addr = format!("{}:{}", &peer_addr[..pos], peer_port);
                    println!("Received Ping from {}:{}", peer_id, peer_addr);
                    Some(peer_addr)
                } else {
                    println!("Received Ping from {} with invalid address", peer_id);
                    return;
                };
                routing.lock().unwrap().insert(peer_id.clone(), insert_addr.unwrap());
                let _ = send_message(stream, &Message::Pong(my_id.clone(), my_port));
            }
            Message::FindNode(target_id) => {
                let closest = routing.lock().unwrap().find_closest(&target_id, 5);
                let _ = send_message(stream, &Message::FoundNodes(closest));
            }
            Message::Pong(peer_id,peer_port) => {
                println!("Received Pong from {},{}", peer_id, peer_port);
                if let Some(pos) = peer_addr.rfind(':') {
                    let peer_addr = format!("{}:{}", &peer_addr[..pos], peer_port);
                    let mut routing = routing.lock().unwrap();
                    routing.insert(peer_id.clone(), peer_addr);
                } else {
                    println!("Received Pong from {} with invalid address", peer_id);
                    return;
                }
            } 
            Message::FoundNodes(_) => {
                println!("Received: {:?}", msg);
            }
        }
    }

    fn recursive_find_node(&self, target: &NodeId, max_rounds: usize) {
        use std::collections::HashSet;

        let mut queried = HashSet::new();
        let mut to_query = {
            let routing = self.routing.lock().unwrap();
            routing.find_closest(target, 5)
        };

        for _ in 0..max_rounds {
            let mut new_peers = Vec::new();

            for (id, addr) in &to_query {
                if queried.contains(id) {
                    continue;
                }

                if let Ok(mut stream) = TcpStream::connect(addr) {
                    let _ = send_message(&mut stream, &Message::FindNode(target.clone()));
                    if let Ok(Message::FoundNodes(nodes)) = receive_message(&mut stream) {
                        new_peers.extend(nodes);
                    }
                }

                queried.insert(id.clone());
            }

            let mut routing = self.routing.lock().unwrap();
            for (id, addr) in new_peers {
                if id != self.id {
                    routing.insert(id.clone(), addr.clone());
                    to_query.push((id, addr));
                }
            }
        }
    }
}
