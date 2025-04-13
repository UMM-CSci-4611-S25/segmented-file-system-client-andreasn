// Below is a version of the `main` function and some error types. This assumes
// the existence of types like `FileManager`, `Packet`, and `PacketParseError`.
// You can use this code as a starting point for the exercise, or you can
// delete it and write your own code with the same function signature.

#![warn(clippy::pedantic)]
#![warn(clippy::style)]
#![warn(clippy::perf)]
#![warn(clippy::complexity)]
#![warn(clippy::correctness)]

mod file_manager;

#[allow(unused_imports)]
use std::{
    collections::HashMap,
    ffi::OsString,
    io::{self, Write},
    net::UdpSocket,
    str::{self, Bytes, FromStr},
};

use file_manager::FileManager;
use packet::Packet;

mod packet;

pub struct PacketGroup {
    file_name: Option<OsString>,
    file_id: u8,
    expected_number_of_packets: Option<usize>,
    packets: HashMap<u16, Vec<u8>>,
}

#[derive(Debug)]
pub enum ClientError {
    IoError(std::io::Error),
    PacketParseError(packet::PacketParseError),
}

impl From<std::io::Error> for ClientError {
    fn from(e: std::io::Error) -> Self {
        ClientError::IoError(e)
    }
}

impl From<packet::PacketParseError> for ClientError {
    fn from(e: packet::PacketParseError) -> Self {
        Self::PacketParseError(e)
    }
}

fn main() -> Result<(), ClientError> {
    let sock = UdpSocket::bind("0.0.0.0:7077")?;

    let remote_addr = "127.0.0.1:6014";
    sock.connect(remote_addr)?;
    let mut buf = [0; 1028];

    // Send an empty packet to initiate communication with the server
    // Fixed: Adding ? to handle errors and only sending 1 byte
    sock.send(&buf[..1])?;

    let mut file_manager = FileManager::default();

    println!("Receiving packets...");
    while !file_manager.received_all_packets() {
        let len = sock.recv(&mut buf)?;
        let packet: Packet = buf[..len].try_into()?;
        print!(".");
        io::stdout().flush()?;
        file_manager.process_packet(packet);
    }

    println!("\nAll packets received. Writing files...");
    file_manager.write_all_files()?;
    println!("Files successfully written!");

    Ok(())
}
// Don't fully delete. This is for testing purposes

#[cfg(test)]
mod tests {
    use crate::{
        file_manager::FileManager,
        packet::{data_packet::DataPacket, header_packet::HeaderPacket, Packet},
        *,
    };

    #[test]
    fn test_try_into_header_packet() {
        let header_packet_bytes: [u8; 6] = [0, 1, b't', b'e', b's', b't'];
        let packet = HeaderPacket::try_from(&header_packet_bytes[..]).unwrap();

        assert_eq!(
            packet,
            HeaderPacket {
                status_byte: 0,
                file_id: 1,
                file_name: OsString::from("test")
            }
        );
    }

    #[test]
    fn test_try_into_data_packet() {
        let data_packet_bytes: [u8; 6] = [1, 1, 2, 2, 3, 3];
        let packet = DataPacket::try_from(&data_packet_bytes[..]).unwrap();

        assert_eq!(
            packet,
            DataPacket {
                status_byte: 1,
                file_id: 1,
                packet_number: 514,
                data: vec![3, 3]
            }
        );
    }

    #[test]
    fn test_process_header_packet() {
        let packet_group1: PacketGroup = PacketGroup {
            file_name: Some(OsString::from("test")),
            file_id: 4,
            expected_number_of_packets: None,
            packets: HashMap::new(),
        };
        let mut file_manager: FileManager = FileManager {
            packet_groups: vec![packet_group1],
        };

        let header_packet_bytes: [u8; 6] = [0, 1, b't', b'e', b's', b't'];
        let packet = HeaderPacket::try_from(&header_packet_bytes[..]).unwrap();

        file_manager.process_packet(Packet::HeaderPacket(packet));

        assert_eq!(
            file_manager.packet_groups[0].file_name,
            Some(OsString::from("test"))
        );
    }

    #[test]
    fn test_empty_process_header_packet() {
        let mut file_manager: FileManager = FileManager {
            packet_groups: vec![],
        };

        let header_packet_bytes: [u8; 6] = [0, 1, b't', b'e', b's', b't'];
        let packet = HeaderPacket::try_from(&header_packet_bytes[..]).unwrap();

        assert!(file_manager.packet_groups.is_empty());
        file_manager.process_packet(Packet::HeaderPacket(packet));
        assert_eq!(file_manager.packet_groups.len(), 1);
        assert_eq!(
            file_manager.packet_groups[0].file_name,
            Some(OsString::from("test"))
        );
        assert_eq!(file_manager.packet_groups[0].file_id, 1);
    }

    #[test]
    fn test_process_data_packet() {
        let packet_group1: PacketGroup = PacketGroup {
            file_name: Some(OsString::from("test")),
            file_id: 4,
            expected_number_of_packets: None,
            packets: HashMap::new(),
        };
        let mut file_manager: FileManager = FileManager {
            packet_groups: vec![packet_group1],
        };

        let data_packet_bytes: [u8; 6] = [1, 1, 2, 2, 3, 3];
        let packet = DataPacket::try_from(&data_packet_bytes[..]).unwrap();

        file_manager.process_packet(Packet::DataPacket(packet));
        assert!(file_manager.packet_groups[1].packets.contains_key(&514));
        assert_eq!(
            file_manager.packet_groups[1].packets.get(&514),
            Some(&vec![3, 3])
        );
    }

    #[test]
    fn test_empty_process_data_packet() {
        let mut file_manager: FileManager = FileManager {
            packet_groups: vec![],
        };

        let data_packet_bytes: [u8; 6] = [1, 1, 2, 2, 3, 3];
        let packet = DataPacket::try_from(&data_packet_bytes[..]).unwrap();

        assert!(file_manager.packet_groups.is_empty());
        file_manager.process_packet(Packet::DataPacket(packet));
        assert_eq!(file_manager.packet_groups.len(), 1);
        assert!(file_manager.packet_groups[0].packets.contains_key(&514));
        assert_eq!(
            file_manager.packet_groups[0].packets.get(&514),
            Some(&vec![3, 3])
        );
    }

    #[test]
    fn test_is_last_data_packet() {
        // Regular data packet (status byte 1)
        let regular_packet = DataPacket {
            status_byte: 1,
            file_id: 1,
            packet_number: 0,
            data: vec![1, 2, 3],
        };
        assert!(!regular_packet.is_last_data_packet());

        // Last data packet (status byte 3 - both bits set)
        let last_packet = DataPacket {
            status_byte: 3,
            file_id: 1,
            packet_number: 5,
            data: vec![1, 2, 3],
        };
        assert!(last_packet.is_last_data_packet());
    }

    #[test]
    fn test_process_last_data_packet() {
        let mut file_manager = FileManager {
            packet_groups: vec![],
        };

        // Create a packet with status byte 3 (last packet)
        let last_data_packet_bytes: [u8; 6] = [3, 1, 0, 5, 3, 3]; // Status byte 3, packet #5
        let packet = DataPacket::try_from(&last_data_packet_bytes[..]).unwrap();

        file_manager.process_packet(Packet::DataPacket(packet));

        // Check if expected_number_of_packets was set correctly
        assert_eq!(
            file_manager.packet_groups[0].expected_number_of_packets,
            Some(6)
        ); // Packet #5 + 1
    }

    #[test]
    fn test_received_all_packets() {
        // Test with incomplete file
        let incomplete_group = PacketGroup {
            file_name: Some(OsString::from("test")),
            file_id: 1,
            expected_number_of_packets: Some(3),
            packets: {
                let mut packets = HashMap::new();
                packets.insert(0, vec![1, 2, 3]);
                packets.insert(1, vec![4, 5, 6]);
                // Missing packet #2
                packets
            },
        };

        let file_manager = FileManager {
            packet_groups: vec![incomplete_group],
        };

        assert!(!file_manager.received_all_packets());

        // Test with complete file
        let complete_group = PacketGroup {
            file_name: Some(OsString::from("test")),
            file_id: 1,
            expected_number_of_packets: Some(3),
            packets: {
                let mut packets = HashMap::new();
                packets.insert(0, vec![1, 2, 3]);
                packets.insert(1, vec![4, 5, 6]);
                packets.insert(2, vec![7, 8, 9]);
                packets
            },
        };

        let file_manager = FileManager {
            packet_groups: vec![complete_group],
        };

        assert!(file_manager.received_all_packets());
    }

    #[test]
    fn test_out_of_order_packet_processing() {
        let mut file_manager = FileManager::default();

        // Process data packets before header
        let data_packet1 = DataPacket {
            status_byte: 1,
            file_id: 1,
            packet_number: 0,
            data: vec![1, 2, 3],
        };

        let data_packet2 = DataPacket {
            status_byte: 3, // Last packet
            file_id: 1,
            packet_number: 1,
            data: vec![4, 5, 6],
        };

        file_manager.process_packet(Packet::DataPacket(data_packet1));
        file_manager.process_packet(Packet::DataPacket(data_packet2));

        // Now process header
        let header_packet = HeaderPacket {
            status_byte: 0,
            file_id: 1,
            file_name: OsString::from("test.txt"),
        };

        file_manager.process_packet(Packet::HeaderPacket(header_packet));

        // Verify everything is set correctly
        assert_eq!(
            file_manager.packet_groups[0].file_name,
            Some(OsString::from("test.txt"))
        );
        assert_eq!(
            file_manager.packet_groups[0].expected_number_of_packets,
            Some(2)
        );
        assert_eq!(file_manager.packet_groups[0].packets.len(), 2);
        assert!(file_manager.received_all_packets());
    }

    #[test]
    fn test_multiple_files_interleaved() {
        let mut file_manager = FileManager::default();

        // Process packets from two different files interleaved
        let header1 = HeaderPacket {
            status_byte: 0,
            file_id: 1,
            file_name: OsString::from("file1.txt"),
        };

        let data1_file1 = DataPacket {
            status_byte: 1,
            file_id: 1,
            packet_number: 0,
            data: vec![1, 2, 3],
        };

        let header2 = HeaderPacket {
            status_byte: 0,
            file_id: 2,
            file_name: OsString::from("file2.txt"),
        };

        let data1_file2 = DataPacket {
            status_byte: 1,
            file_id: 2,
            packet_number: 0,
            data: vec![7, 8, 9],
        };

        let data2_file1 = DataPacket {
            status_byte: 3, // Last packet
            file_id: 1,
            packet_number: 1,
            data: vec![4, 5, 6],
        };

        let data2_file2 = DataPacket {
            status_byte: 3, // Last packet
            file_id: 2,
            packet_number: 1,
            data: vec![10, 11, 12],
        };

        // Process in interleaved order
        file_manager.process_packet(Packet::HeaderPacket(header1));
        file_manager.process_packet(Packet::DataPacket(data1_file2));
        file_manager.process_packet(Packet::HeaderPacket(header2));
        file_manager.process_packet(Packet::DataPacket(data1_file1));
        file_manager.process_packet(Packet::DataPacket(data2_file2));
        file_manager.process_packet(Packet::DataPacket(data2_file1));

        assert!(file_manager.received_all_packets());
        assert_eq!(file_manager.packet_groups.len(), 2);
    }

    #[test]
    fn test_edge_case_single_packet_file() {
        let mut file_manager = FileManager::default();

        // File with a single packet
        let header = HeaderPacket {
            status_byte: 0,
            file_id: 1,
            file_name: OsString::from("single.txt"),
        };

        let data = DataPacket {
            status_byte: 3, // Last packet
            file_id: 1,
            packet_number: 0,
            data: vec![1, 2, 3],
        };

        file_manager.process_packet(Packet::HeaderPacket(header));
        file_manager.process_packet(Packet::DataPacket(data));

        assert!(file_manager.received_all_packets());
        assert_eq!(
            file_manager.packet_groups[0].expected_number_of_packets,
            Some(1)
        );
    }
}
