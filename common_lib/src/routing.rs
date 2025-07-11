use crate::kad_id::NodeId;
use crate::protocol::xor_distance;
use std::collections::VecDeque;
use std::time::Instant;

/// Kademlia 路由表（基于 K桶）
#[derive(Debug)]
pub struct RoutingTable {
    pub buckets: Vec<Vec<(NodeId, String)>>, // 160 桶，每桶 Vec 存 (NodeId, Addr)
    pub cache_buckets: Vec<VecDeque<(NodeId, String)>>, // 缓存桶
    pub my_id: NodeId,
    pub k: usize, // 每桶最大节点数
    pub last_touched: Vec<Instant>, // 记录每个桶的最后访问时间
}

impl RoutingTable {
    pub fn new(my_id: NodeId, k: usize) -> Self {
        Self {
            buckets: vec![Vec::new(); 160],
            cache_buckets: vec![VecDeque::new(); 160],
            my_id,
            k,
            last_touched: vec![Instant::now(); 160], // 初始化每个桶的最后访问时间为当前时间
        }
    }

    /// 插入节点（若桶未满且非本节点）
    pub fn insert(&mut self, id: NodeId, addr: String) {
        if id == self.my_id {
            return; // 不插入自身
        }

        let index = Self::bucket_index(&self.my_id, &id);
        let bucket = &mut self.buckets[index];
        let cache_bucket = &mut self.cache_buckets[index];

        if bucket.iter().any(|(nid, a)| *nid == id && *a == addr) {
            self.last_touched[index] = Instant::now();
            return;
        }

        bucket.retain(|(nid, a)| {
            let keep = !((nid == &id && a != &addr) || (nid != &id && a == &addr));
            if !keep {
                println!(
                    "[RoutingTable] Removed conflicting node: id={:?}, addr={}",
                    nid, a
                );
            }
            keep
        });
        if bucket.len() < self.k {
            bucket.push((id, addr));
            self.last_touched[index] = Instant::now();
        } else {
            if let Some(existing) = cache_bucket.iter_mut().find(|(nid, _)| *nid == id) {
                existing.1 = addr; // 更新缓存地址
            } else {
                // 如果缓存满了，移除最旧的节点
                if cache_bucket.len() > self.k/2 {
                    cache_bucket.pop_front();
                }
                cache_bucket.push_back((id, addr)); // 添加到缓存
            }
        }
    }

    pub fn remove(&mut self, id: &NodeId) {
        let bucket_index = Self::bucket_index(&self.my_id, id);
        if let Some(pos) = self.buckets[bucket_index]
            .iter()
            .position(|(nid, _)| nid == id)
        {
            self.buckets[bucket_index].remove(pos);
        }
    }

    pub fn substitute_or_remove_node(&mut self, id: NodeId) {
        let bucket_index = Self::bucket_index(&self.my_id, &id);
        if let Some(pos) = self.buckets[bucket_index]
            .iter()
            .position(|(nid, _)| *nid == id)
        {
            let sub_n = self.cache_buckets[bucket_index].pop_front();
            if sub_n.is_none() {
                print!("[RoutingTable] No cached node to substitute for {}, target id will be removed for bucket[{}]\n", id, bucket_index);
                self.buckets[bucket_index].remove(pos);
                return;
            }
            self.buckets[bucket_index][pos] = sub_n.unwrap(); // 替换为缓存节点
            println!("[RoutingTable] Substitute node {} with cached node {} in bucket[{}]", id, self.buckets[bucket_index][pos].0, bucket_index);
        }
    }

    /// 查找与目标 id 最接近的 k 个节点
    pub fn find_closest(&self, target: &NodeId, k: usize) -> Vec<(NodeId, String)> {
        let mut all_nodes = Vec::new();

        for bucket in &self.buckets {
            for (id, addr) in bucket {
                let dist = xor_distance(id, target);
                all_nodes.push((id.clone(), addr.clone(), dist));
            }
        }

        all_nodes.sort_by_key(|(_, _, dist)| dist.0.clone());
        all_nodes.into_iter().take(k).map(|(id, addr, _)| (id, addr)).collect()
    }

    /// 计算目标节点属于哪个桶（返回桶索引 0~159）
    pub fn bucket_index(a: &NodeId, b: &NodeId) -> usize {
        let NodeId(x) = xor_distance(a, b);
        for i in 0..160 {
            let byte_index = i / 8;
            let bit_index = 7 - (i % 8);
            if (x[byte_index] >> bit_index) & 1 == 1 {
                return i;
            }
        }
        159 // fallback，理论不会触发
    }

    // 返回所有节点的id和地址列表
    pub fn all_nodes(&self) -> Vec<(usize, NodeId, String)> {
        let mut nodes = Vec::new();
        for (i,bucket) in self.buckets.iter().enumerate() {
            for (id, addr) in bucket {
                nodes.push((i, id.clone(), addr.clone()));
            }
        }
        nodes
    }
}