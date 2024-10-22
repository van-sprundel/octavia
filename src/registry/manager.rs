use super::{
    entry::{write_registry_packet, RegistryEntry},
    Biome, ChatParameters, ChatType, DamageType, DimensionType, RegistryData, TrimMaterial,
    TrimPattern, WolfVariant,
};
use crate::{error::Result, packet::reader::PacketReader, tag::*};
use bytes::{BufMut, BytesMut};
use std::collections::HashMap;
use tokio::{io::AsyncWriteExt, net::TcpStream};
use tracing::debug;

pub struct RegistryManager {
    registry_data: RegistryData,
}

impl RegistryManager {
    pub fn new() -> Result<Self> {
        let default_registry_data = include_str!("../../default_registry.json");
        let registry_data: RegistryData = serde_json::from_str(default_registry_data)?;

        Ok(Self { registry_data })
    }

    pub async fn write_registry_data(&self, socket: &mut TcpStream) -> Result<()> {
        self.send_registry_packet(socket, "worldgen/biome", self.registry_data.biomes.keys())
            .await?;
        self.send_registry_packet(socket, "chat_type", self.registry_data.chat_types.keys())
            .await?;
        self.send_registry_packet(
            socket,
            "trim_pattern",
            self.registry_data.trim_patterns.keys(),
        )
        .await?;
        self.send_registry_packet(
            socket,
            "trim_material",
            self.registry_data.trim_materials.keys(),
        )
        .await?;
        self.send_registry_packet(
            socket,
            "wolf_variant",
            self.registry_data.wolf_variants.keys(),
        )
        .await?;
        self.send_registry_packet(
            socket,
            "painting_variant",
            self.registry_data.painting_variants.keys(),
        )
        .await?;
        self.send_registry_packet(
            socket,
            "dimension_type",
            self.registry_data.dimension_types.keys(),
        )
        .await?;
        self.send_registry_packet(
            socket,
            "damage_type",
            self.registry_data.damage_types.keys(),
        )
        .await?;
        self.send_registry_packet(
            socket,
            "banner_pattern",
            self.registry_data.banner_patterns.keys(),
        )
        .await?;
        self.send_registry_packet(
            socket,
            "enchantment",
            self.registry_data.enchantments.keys(),
        )
        .await?;
        self.send_registry_packet(
            socket,
            "jukebox_song",
            self.registry_data.jukebox_songs.keys(),
        )
        .await?;

        Ok(())
    }

    async fn send_registry_packet<'a, I>(
        &self,
        socket: &mut TcpStream,
        registry_name: &str,
        entries: I,
    ) -> Result<()>
    where
        I: IntoIterator<Item = &'a String>,
    {
        let entries: Vec<&str> = entries.into_iter().map(|s| s.as_str()).collect();
        let packet = write_registry_packet(registry_name, &entries);

        socket.write_all(&packet).await?;

        Ok(())
    }
    pub async fn write_update_tags(&self, socket: &mut TcpStream) -> Result<()> {
        let default_tags = include_str!("../../default_tags.json");
        let tag_data: TagData = serde_json::from_str(default_tags)?;

        let mut packet = BytesMut::new();

        // Packet ID for Update Tags
        PacketReader::write_varint(&mut packet, 0x0D);

        // Count how many non-empty registry types we have
        let registry_count = [
            (!tag_data.banner_patterns.is_empty()) as i32,
            (!tag_data.blocks.is_empty()) as i32,
            (!tag_data.cat_variants.is_empty()) as i32,
            (!tag_data.damage_types.is_empty()) as i32,
            (!tag_data.enchantments.is_empty()) as i32,
            (!tag_data.entity_types.is_empty()) as i32,
            (!tag_data.fluids.is_empty()) as i32,
            (!tag_data.game_events.is_empty()) as i32,
            (!tag_data.instruments.is_empty()) as i32,
            (!tag_data.items.is_empty()) as i32,
            (!tag_data.painting_variants.is_empty()) as i32,
            (!tag_data.point_of_interest_types.is_empty()) as i32,
            (!tag_data.worldgen_biomes.is_empty()) as i32,
        ]
        .iter()
        .sum();

        // Write number of registries that have tags
        PacketReader::write_varint(&mut packet, registry_count);

        // Helper function to write tag groups

        // Write each registry's tags
        Self::write_tag_groups(&mut packet, "banner_pattern", &tag_data.banner_patterns);
        Self::write_tag_groups(&mut packet, "block", &tag_data.blocks);
        Self::write_tag_groups(&mut packet, "cat_variant", &tag_data.cat_variants);
        Self::write_tag_groups(&mut packet, "damage_type", &tag_data.damage_types);
        Self::write_tag_groups(&mut packet, "enchantment", &tag_data.enchantments);
        Self::write_tag_groups(&mut packet, "entity_type", &tag_data.entity_types);
        Self::write_tag_groups(&mut packet, "fluid", &tag_data.fluids);
        Self::write_tag_groups(&mut packet, "game_event", &tag_data.game_events);
        Self::write_tag_groups(&mut packet, "instrument", &tag_data.instruments);
        Self::write_tag_groups(&mut packet, "item", &tag_data.items);
        Self::write_tag_groups(&mut packet, "painting_variant", &tag_data.painting_variants);
        Self::write_tag_groups(
            &mut packet,
            "point_of_interest_type",
            &tag_data.point_of_interest_types,
        );
        Self::write_tag_groups(&mut packet, "worldgen/biome", &tag_data.worldgen_biomes);

        // Write final packet with length prefix
        let packet_len = packet.len();
        let mut final_packet = BytesMut::new();
        PacketReader::write_varint(&mut final_packet, packet_len as i32);
        final_packet.extend_from_slice(&packet);

        socket.write_all(&final_packet).await?;

        Ok(())
    }

    fn write_tag_groups(packet: &mut BytesMut, registry_name: &str, tags: &[TagGroup]) {
        if tags.is_empty() {
            return;
        }

        PacketReader::write_identifier(packet, "minecraft", registry_name);

        PacketReader::write_varint(packet, tags.len() as i32);

        for tag in tags {
            let tag_name = &tag.tag_name;
            PacketReader::write_identifier(packet, &tag_name.namespace, &tag_name.name);

            let entries = &tag.entries;
            PacketReader::write_varint(packet, entries.len() as i32);
            for &entry_id in entries {
                PacketReader::write_varint(packet, entry_id);
            }
        }
    }
}
