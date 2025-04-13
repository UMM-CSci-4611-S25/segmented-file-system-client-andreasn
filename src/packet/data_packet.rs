use crate::packet::PacketParseError;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub struct DataPacket {
    pub status_byte: u8,
    pub file_id: u8,
    pub packet_number: u16,
    pub data: Vec<u8>,
}

impl DataPacket {
    pub fn is_last_data_packet(&self) -> bool {
        // If the second bit is 1 (status byte % 4 == 3), it's the last packet
        self.status_byte & 0b10 != 0
    }
}

impl TryFrom<&[u8]> for DataPacket {
    type Error = PacketParseError;

    fn try_from(buffer: &[u8]) -> Result<Self, Self::Error> {
        // Data packet needs at least 4 bytes: status byte, file ID, and 2 bytes for packet number
        if buffer.len() < 4 {
            return Err(PacketParseError::InvalidPacketLength);
        }

        let status_byte = buffer[0];
        
        // Status byte must be odd for data packets
        if status_byte & 1 == 0 {
            return Err(PacketParseError::InvalidDataPacket);
        }
        
        let file_id = buffer[1];
        
        // Construct packet number using big endian (first byte is most significant)
        let packet_number = u16::from_be_bytes([buffer[2], buffer[3]]);
        
        // The rest of the buffer is the data
        let data = buffer[4..].to_vec();
        
        Ok(DataPacket {
            status_byte,
            file_id,
            packet_number,
            data,
        })
    }
}