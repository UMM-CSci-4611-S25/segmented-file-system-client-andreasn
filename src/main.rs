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

// fn main() -> Result<(), ClientError> {
//     let sock = UdpSocket::bind("0.0.0.0:7077")?;

//     let remote_addr = "127.0.0.1:6014";
//     sock.connect(remote_addr)?;
//     let mut buf = [0; 1028];

//     let _ = sock.send(&buf[..1028]);

//     let mut file_manager = FileManager::default();

//     while !file_manager.received_all_packets() {
//         let len = sock.recv(&mut buf)?;
//         let packet: Packet = buf[..len].try_into()?;
//         print!(".");
//         io::stdout().flush()?;
//         file_manager.process_packet(packet);
//     }

//     file_manager.write_all_files()?;

//     Ok(())
// }
// Don't fully delete. This is for testing purposes

fn main() {}

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

        // TODO: Test expected_number_of_packets
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

        // TODO: Test expected_number_of_packets
    }
}
