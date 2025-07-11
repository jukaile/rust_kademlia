use rand::Rng;
use serde::{Serialize, Deserialize};
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

const NODE_ID_PATH: &str = "._";

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub [u8; 20]);

fn generate_node_id() -> NodeId {
    let mut rng = rand::rng();
    let mut id = [0u8; 20];
    rng.fill(&mut id);
    NodeId(id)
}

pub fn load_or_generate_node_id(port: u16) -> NodeId {
    let path = if port == 0 {
        NODE_ID_PATH.to_string()
    } else {
        format!("{}.{port}", NODE_ID_PATH)
    };
    if Path::new(&path).exists() {
        if let Ok(mut file) = File::open(&path) {
            let mut buf = [0u8; 20];
            if let Ok(_) = file.read_exact(&mut buf) {
                return NodeId(buf);
            }
        }
    }
    let new_id = generate_node_id();
    if let Ok(mut file) = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)
    {
        let _ = file.write_all(&new_id.0);
    }
    new_id
}

pub fn generate_random_id_in_bucket(my_id: &NodeId, bucket_index: usize) -> NodeId {
    let mut rng = rand::rng();
    let mut id = my_id.0;

    let byte_index = bucket_index / 8;
    let bit_index = 7 - (bucket_index % 8);

    id[byte_index] ^= 1 << bit_index;

    for i in byte_index + 1..20 {
        id[i] = rng.random();
    }

    NodeId(id)
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}
