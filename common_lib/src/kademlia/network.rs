use std::io::{Read, Write};
use std::net::TcpStream;
use crate::kademlia::protocol::Message;

pub fn send_message(stream: &mut TcpStream, msg: &Message) -> std::io::Result<()> {
    let bytes = bincode::serialize(msg).unwrap();
    let len = (bytes.len() as u32).to_be_bytes();
    stream.write_all(&len)?;
    stream.write_all(&bytes)?;
    Ok(())
}

pub fn receive_message(stream: &mut TcpStream) -> std::io::Result<Message> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf)?;
    let len = u32::from_be_bytes(len_buf) as usize;
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf)?;
    Ok(bincode::deserialize(&buf).unwrap())
}