use crate::packet::{data_packet::DataPacket, header_packet::HeaderPacket, Packet};
use crate::PacketGroup;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

pub struct FileManager {
    pub packet_groups: Vec<PacketGroup>,
}

impl Default for FileManager {
    fn default() -> Self {
        Self {
            packet_groups: Vec::new(),
        }
    }
}

impl FileManager {
    pub fn received_all_packets(&self) -> bool {
        if self.packet_groups.is_empty() {
            return false;
        }

        for packet_group in &self.packet_groups {
            if packet_group.expected_number_of_packets.is_none() || 
               packet_group.expected_number_of_packets != Some(packet_group.packets.len()) {
                return false;
            }
        }

        true
    }

    pub fn process_packet(&mut self, packet: Packet) {
        match packet {
            Packet::HeaderPacket(header_packet) => self.process_header_packet(header_packet),
            Packet::DataPacket(data_packet) => self.process_data_packet(data_packet),
        }
    }

    pub fn process_header_packet(&mut self, header_packet: HeaderPacket) {
        let file_id = header_packet.file_id;
        let file_name = header_packet.file_name;

        // Check if we already have a packet group for this file ID
        for packet_group in &mut self.packet_groups {
            if packet_group.file_id == file_id {
                packet_group.file_name = Some(file_name);
                return;
            }
        }

        // If not, create a new packet group
        let packet_group = PacketGroup {
            file_name: Some(file_name),
            file_id,
            expected_number_of_packets: None,
            packets: HashMap::new(),
        };
        self.packet_groups.push(packet_group);
    }

    pub fn process_data_packet(&mut self, data_packet: DataPacket) {
        let file_id = data_packet.file_id;
        let is_last_data_packet = data_packet.is_last_data_packet();
        let packet_number = data_packet.packet_number;

        // Try to find an existing packet group
        for packet_group in &mut self.packet_groups {
            if packet_group.file_id == file_id {
                // If this is the last packet, update the expected number of packets
                if is_last_data_packet {
                    packet_group.expected_number_of_packets = Some(packet_number as usize + 1);
                }
                
                packet_group.packets.insert(packet_number, data_packet.data);
                return;
            }
        }

        // If no packet group exists, create a new one
        let mut packets = HashMap::new();
        packets.insert(packet_number, data_packet.data);
        
        let expected_packets = if is_last_data_packet {
            Some(packet_number as usize + 1)
        } else {
            None
        };
        
        let packet_group = PacketGroup {
            file_name: None,
            file_id,
            expected_number_of_packets: expected_packets,
            packets,
        };
        
        self.packet_groups.push(packet_group);
    }

    pub fn write_all_files(&self) -> io::Result<()> {
        for packet_group in &self.packet_groups {
            if let Some(file_name) = &packet_group.file_name {
                if let Some(expected_packets) = packet_group.expected_number_of_packets {
                    if packet_group.packets.len() == expected_packets {
                        // Write the file if all packets received
                        let mut file = File::create(Path::new(&file_name))?;
                        
                        // Write the packets in order by packet number
                        for packet_num in 0..expected_packets {
                            if let Some(data) = packet_group.packets.get(&(packet_num as u16)) {
                                file.write_all(data)?;
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
}