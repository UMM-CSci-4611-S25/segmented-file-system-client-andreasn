pub mod data_packet;
pub mod header_packet;

use data_packet::DataPacket;
use header_packet::HeaderPacket;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub enum Packet {
    HeaderPacket(HeaderPacket),
    DataPacket(DataPacket),
}

#[derive(Debug, PartialEq)]
pub enum PacketParseError {
    InvalidPacketType,
    InvalidPacketLength,
    InvalidHeaderPacket,
    InvalidDataPacket,
}

impl TryFrom<&[u8]> for Packet {
    type Error = PacketParseError;

    fn try_from(buffer: &[u8]) -> Result<Self, Self::Error> {
        if buffer.is_empty() {
            return Err(PacketParseError::InvalidPacketLength);
        }

        let status_byte = buffer[0];
        
        // Even status byte (least significant bit is 0) means header packet
        if status_byte & 1 == 0 {
            let header_packet = HeaderPacket::try_from(buffer)?;
            Ok(Packet::HeaderPacket(header_packet))
        } 
        // Odd status byte (least significant bit is 1) means data packet
        else {
            let data_packet = DataPacket::try_from(buffer)?;
            Ok(Packet::DataPacket(data_packet))
        }
    }
}