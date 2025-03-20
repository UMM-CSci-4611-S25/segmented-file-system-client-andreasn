// Below is a version of the `main` function and some error types. This assumes
// the existence of types like `FileManager`, `Packet`, and `PacketParseError`.
// You can use this code as a starting point for the exercise, or you can
// delete it and write your own code with the same function signature.

// packet structure

use std::ffi::OsString;
use std::convert::TryFrom;

pub struct Header {
    file_id: u8,
    file_name: OsString,
}
pub struct Data {
    file_id: u8,
    packet_number: u16,
    is_last_packet: bool,
    data: Vec<u8>,
}

pub enum Packet {
    Header(Header),
    Data(Data),
}

#[derive(Debug)]
pub struct PacketParseError;

impl TryFrom<&[u8]> for Packet {
    type Error = PacketParseError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < 2 {
            return Err(PacketParseError);
        }

        let status_byte = bytes[0];
        let file_id = bytes[1];

        // If status byte is even (least significant bit is 0), it's a header packet
        if status_byte % 2 == 0 {
            // Extract file name from the rest of the bytes
            let file_name = OsString::from(String::from_utf8_lossy(&bytes[2..]));
            Ok(Packet::Header(Header { file_id, file_name }))
        } else {
            // It's a data packet
            if bytes.len() < 4 {
                return Err(PacketParseError);
            }

            // Extract packet number (2 bytes)
            let packet_number_bytes: [u8; 2] = [bytes[2], bytes[3]];
            let packet_number = u16::from_be_bytes(packet_number_bytes);
            
            // Check if it's the last packet
            let is_last_packet = (status_byte & 2) != 0; // Check if second bit is set
            
            // Extract data
            let data = bytes[4..].to_vec();
            
            Ok(Packet::Data(Data {
                file_id,
                packet_number,
                is_last_packet,
                data,
            }))
        }
    }
}



use std::{
    io::{self, Write},
    net::UdpSocket,
};

#[derive(Debug)]
pub enum ClientError {
    IoError(std::io::Error),
    PacketParseError(PacketParseError),
}

impl From<std::io::Error> for ClientError {
    fn from(e: std::io::Error) -> Self {
        ClientError::IoError(e)
    }
}

impl From<PacketParseError> for ClientError {
    fn from(e: PacketParseError) -> Self {
        Self::PacketParseError(e)
    }
}

fn main() -> Result<(), ClientError> {
    let sock = UdpSocket::bind("0.0.0.0:7077")?;

    let remote_addr = "127.0.0.1:6014";
    sock.connect(remote_addr)?;
    let mut buf = [0; 1028];

    let _ = sock.send(&buf[..1028]);

    let mut file_manager = FileManager::default();

    while !file_manager.received_all_packets() {
        let len = sock.recv(&mut buf)?;
        let packet: Packet = buf[..len].try_into()?;
        print!(".");
        io::stdout().flush()?;
        file_manager.process_packet(packet);
    }

    file_manager.write_all_files()?;

    Ok(())
}


