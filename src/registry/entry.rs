use crate::packet::reader::PacketReader;
use bytes::{BufMut, BytesMut};
use serde::Deserialize;
use tracing::debug;

#[derive(Debug, Deserialize)]
pub struct RegistryEntry {
    pub namespace: String,
    pub name: String,
}

impl RegistryEntry {
    pub fn new(name: &str) -> Self {
        if let Some((namespace, name)) = name.split_once(':') {
            Self {
                namespace: namespace.to_string(),
                name: name.to_string(),
            }
        } else {
            Self {
                namespace: "minecraft".to_string(),
                name: name.to_string(),
            }
        }
    }

    pub fn write_to(&self, buf: &mut BytesMut) {
        PacketReader::write_identifier(buf, &self.namespace, &self.name);
    }
}

// Registry packets look like the following
// Packet Length         varint
// Packet ID             varint
// Length of identifier  varint
// Registry name         identifier
// Entry count           varint
//
// foreach entry
// --------------------------------
// Length of identifier  varint
// Entry name            identifier
// Null terminate        varint
// --------------------------------
pub fn write_registry_packet(registry_name: &str, entries: &[&str]) -> BytesMut {
    let mut packet = BytesMut::new();

    PacketReader::write_varint(&mut packet, 0x07);

    PacketReader::write_identifier(&mut packet, "minecraft", registry_name);

    PacketReader::write_varint(&mut packet, entries.len() as i32);

    for entry in entries {
        let registry_entry = RegistryEntry::new(entry);
        registry_entry.write_to(&mut packet);
        packet.put_u8(0); // Has Data = false
    }

    let packet_len = packet.len();
    let mut final_packet = BytesMut::new();
    PacketReader::write_varint(&mut final_packet, packet_len as i32);
    final_packet.extend_from_slice(&packet);

    final_packet
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_packet() {
        let expected = b"\x74\x07\x18minecraft:dimension_type\x04\x13minecraft:overworld\0\x19minecraft:overworld_caves\0\x14minecraft:the_nether\0\x11minecraft:the_end\0";

        let entries = &["overworld", "overworld_caves", "the_nether", "the_end"];
        let packet = write_registry_packet("dimension_type", entries);

        // 0x74                          Packet Length         varint
        // 0x07                          Packet ID             varint
        // 0x18                          Length of identifier  varint
        // minecraft:dimension_type      Registry name         identifier
        // 0x04                          Entry count           varint
        //
        // foreach entry
        // --------------------------------------------------------------
        // 0x13                          Length of identifier  varint
        // minecraft:overworld           Entry name            identifier
        // 0x00                          Null terminate        varint
        // --------------------------------------------------------------
        // ... et cetera
        assert!(!packet.is_empty());
        assert_eq!(&packet[..], expected);
    }
}
