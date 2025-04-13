use super::PacketParseError;
// use crate::packet::PacketParseError;

#[derive(Debug, PartialEq)]
pub struct DataPacket {
    pub(crate) status_byte: u8,
    pub(crate) file_id: u8,
    pub(crate) packet_number: u16,
    pub(crate) data: Vec<u8>,
}

impl DataPacket {
    #[must_use]
    pub fn is_last_data_packet(&self) -> bool {
        self.status_byte % 4 == 3
    }
    // x % 4 == 3
}

impl TryFrom<&[u8]> for DataPacket {
    type Error = PacketParseError;

    fn try_from(bytes: &[u8]) -> Result<Self, PacketParseError> {
        let status_byte: u8 = bytes[0];
        let file_id: u8 = bytes[1];
        let packet_number_bytes: [u8; 2] = [bytes[2], bytes[3]];
        let packet_number: u16 = u16::from_be_bytes(packet_number_bytes);
        let data: Vec<u8> = bytes[4..bytes.len()].to_vec();
        Ok(DataPacket {
            status_byte,
            file_id,
            packet_number,
            data,
        })
    }
}
