// packet

use std::ffi::OsString;
use std::convert::TryFrom;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::io::{self, Write};
use std::net::UdpSocket;

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
            let file_name = OsString::from(String::from_utf8_lossy(&bytes[2..]).into_owned());
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

// file manager 
pub struct PacketGroup {
    file_name: Option<OsString>,
    expected_number_of_packets: Option<usize>,
    packets: HashMap<u16, Vec<u8>>,
}

#[derive(Default)]
pub struct FileManager {
    packet_groups: HashMap<u8, PacketGroup>,
}

impl PacketGroup {
    fn new() -> Self {
        PacketGroup {
            file_name: None,
            expected_number_of_packets: None,
            packets: HashMap::new(),
        }
    }
    
    fn is_complete(&self) -> bool {
        if self.file_name.is_none() {
            return false;
        }
        
        if let Some(expected) = self.expected_number_of_packets {
            self.packets.len() >= expected
        } else {
            false
        }
    }
    
    fn write_to_file(&self) -> io::Result<()> {
        if let Some(file_name) = &self.file_name {
            let mut file = File::create(Path::new(file_name))?;
            
            // Sort packet numbers to ensure correct order
            let mut packet_nums: Vec<_> = self.packets.keys().collect();
            packet_nums.sort();
            
            for &packet_num in packet_nums {
                if let Some(data) = self.packets.get(&packet_num) {
                    file.write_all(data)?;
                }
            }
        }
        Ok(())
    }
}

impl FileManager {
    fn received_all_packets(&self) -> bool {
        if self.packet_groups.is_empty() {
            return false;
        }
        
        self.packet_groups.values().all(|pg| pg.is_complete())
    }
    
    fn process_packet(&mut self, packet: Packet) {
        match packet {
            Packet::Header(header) => {
                let group = self.packet_groups
                    .entry(header.file_id)
                    .or_insert_with(PacketGroup::new);
                group.file_name = Some(header.file_name);
            },
            Packet::Data(data) => {
                let group = self.packet_groups
                    .entry(data.file_id)
                    .or_insert_with(PacketGroup::new);
                
                group.packets.insert(data.packet_number, data.data);
                
                if data.is_last_packet {
                    // If this is the last packet, we know the total number of packets
                    group.expected_number_of_packets = Some((data.packet_number + 1) as usize);
                }
            }
        }
    }
    
    fn write_all_files(&self) -> io::Result<()> {
        for group in self.packet_groups.values() {
            group.write_to_file()?;
        }
        Ok(())
    }
}

// client

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