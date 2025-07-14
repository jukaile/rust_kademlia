use serde::{Serialize, Deserialize};
use crate::kademlia::kad_id::NodeId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    Ping(NodeId,u16),
    Pong(NodeId,u16),
    FindNode(NodeId),
    FoundNodes(Vec<(NodeId, String)>),
}

pub fn xor_distance(a: &NodeId, b: &NodeId) -> NodeId {
    let mut out = [0u8; 20];
    for i in 0..20 {
        out[i] = a.0[i] ^ b.0[i];
    }
    NodeId(out)
}
