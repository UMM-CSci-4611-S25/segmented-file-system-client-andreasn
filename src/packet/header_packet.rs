use crate::packet::PacketParseError;
use std::convert::TryFrom;
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;

#[derive(Debug, PartialEq)]
pub struct HeaderPacket {
    pub status_byte: u8,
    pub file_id: u8,
    pub file_name: OsString,
}

impl TryFrom<&[u8]> for HeaderPacket {
    type Error = PacketParseError;

    fn try_from(buffer: &[u8]) -> Result<Self, Self::Error> {
        // Header packet needs at least 2 bytes: status byte and file ID
        if buffer.len() < 2 {
            return Err(PacketParseError::InvalidPacketLength);
        }

        let status_byte = buffer[0];
        
        // Status byte must be even for header packets
        if status_byte & 1 != 0 {
            return Err(PacketParseError::InvalidHeaderPacket);
        }
        
        let file_id = buffer[1];
        
        // The rest of the buffer is the filename
        let file_name_bytes = &buffer[2..];
        
        // Convert to OsString - handles non-UTF8 filenames
        let file_name = OsString::from_vec(file_name_bytes.to_vec());
        
        Ok(HeaderPacket {
            status_byte,
            file_id,
            file_name,
        })
    }
}