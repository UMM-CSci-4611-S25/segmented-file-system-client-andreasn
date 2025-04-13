pub mod data_packet;
pub mod header_packet;

use header_packet::HeaderPacket;

#[derive(Debug, PartialEq)]
pub enum Packet {
    HeaderPacket(HeaderPacket),
    DataPacket(data_packet::DataPacket),
}

impl Packet {
    pub fn new_header(bytes: &[u8]) -> Result<Self, PacketParseError> {
        Ok(Packet::HeaderPacket(HeaderPacket::try_from(bytes)?))
    }

    pub fn new_data(bytes: &[u8]) -> Result<Self, PacketParseError> {
        Ok(Packet::DataPacket(data_packet::DataPacket::try_from(
            bytes,
        )?))
    }
}

impl TryFrom<&[u8]> for Packet {
    type Error = PacketParseError;

    fn try_from(bytes: &[u8]) -> Result<Self, PacketParseError> {
        let status_byte: u8 = bytes[0];
        Ok(if status_byte == 0 {
            Packet::new_header(bytes)?
            // Packet::HeaderPacket(HeaderPacket::try_from(bytes)?)
            // let file_name = OsString::from_str(str::from_utf8(&bytes[2..bytes.len()]).unwrap()).unwrap(); // Uhhhhhh what is this line... there is probably a better way to do this?
        } else {
            Packet::new_data(bytes)?
        })
    }
}

#[derive(Debug)]
pub enum PacketParseError {}
