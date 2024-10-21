use super::{
    entry::{write_registry_packet, RegistryEntry},
    Biome, ChatParameters, ChatType, DamageType, DimensionType, RegistryData, TrimMaterial,
    TrimPattern, WolfVariant,
};
use crate::{error::Result, packet::reader::PacketReader};
use bytes::{BufMut, BytesMut};
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

        self.write_update_tags(socket);

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
        // TODO write actual tags to client
        unimplemented!("Update tags packet not yet implemented");
    }
}
