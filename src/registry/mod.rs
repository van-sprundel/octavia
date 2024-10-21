#![allow(unused)]

mod entry;
pub mod manager;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

type Byte = i8;
type Boolean = bool;
type Short = i16;
type Int = i32;
type Long = i64;
type Float = f32;
type Double = f64;

#[derive(Deserialize, Debug)]
pub struct RegistryData {
    #[serde(rename = "minecraft:banner_pattern")]
    banner_patterns: HashMap<String, BannerPattern>,
    #[serde(rename = "minecraft:chat_type")]
    chat_types: HashMap<String, ChatType>,
    #[serde(rename = "minecraft:damage_type")]
    damage_types: HashMap<String, DamageType>,
    #[serde(rename = "minecraft:dimension_type")]
    dimension_types: HashMap<String, DimensionType>,
    #[serde(rename = "minecraft:trim_material")]
    trim_materials: HashMap<String, TrimMaterial>,
    #[serde(rename = "minecraft:trim_pattern")]
    trim_patterns: HashMap<String, TrimPattern>,
    #[serde(rename = "minecraft:wolf_variant")]
    wolf_variants: HashMap<String, WolfVariant>,
    #[serde(rename = "minecraft:worldgen/biome")]
    biomes: HashMap<String, Biome>,
    #[serde(rename = "minecraft:painting_variant")]
    painting_variants: HashMap<String, PaintingVariant>,
    #[serde(rename = "minecraft:enchantment")]
    enchantments: HashMap<String, Enchantment>,
    #[serde(rename = "minecraft:jukebox_song")]
    jukebox_songs: HashMap<String, JukeboxSong>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BannerPattern {
    pub asset_id: String,
    pub translation_key: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatType {
    pub chat: ChatParameters,
    pub narration: ChatParameters,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatParameters {
    pub parameters: Vec<String>,
    pub translation_key: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DamageType {
    pub message_id: String,
    pub exhaustion: Float,
    pub scaling: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DimensionType {
    pub ambient_light: Float,
    pub bed_works: Byte,
    pub coordinate_scale: Double,
    pub effects: String,
    pub has_ceiling: Byte,
    pub has_raids: Byte,
    pub has_skylight: Byte,
    pub height: Int,
    pub infiniburn: String,
    pub logical_height: Int,
    pub min_y: Int,
    pub monster_spawn_block_light_limit: Int,
    pub monster_spawn_light_level: MonsterSpawnLightLevel,
    pub natural: Byte,
    pub piglin_safe: Byte,
    pub respawn_anchor_works: Byte,
    pub ultrawarm: Byte,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonsterSpawnLightLevel {
    pub max_inclusive: Int,
    pub min_inclusive: Int,
    pub r#type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaintingVariant {
    pub asset_id: String,
    pub height: Int,
    pub width: Int,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrimMaterial {
    pub asset_name: String,
    pub description: TrimMaterialDescription,
    pub ingredient: String,
    pub item_model_index: Float,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrimMaterialDescription {
    pub color: String,
    pub translate: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrimPattern {
    pub asset_id: String,
    pub description: TrimPatternDescription,
    pub template_item: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrimPatternDescription {
    pub translate: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WolfVariant {
    pub angry_texture: String,
    pub biomes: String,
    pub tame_texture: String,
    pub wild_texture: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Biome {
    pub downfall: Float,
    pub effects: BiomeEffects,
    pub has_precipitation: Boolean,
    pub temperature: Float,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BiomeEffects {
    pub fog_color: Int,
    pub foliage_color: Option<Int>,
    pub grass_color: Option<Int>,
    pub mood_sound: MoodSound,
    pub music: Option<Music>,
    pub sky_color: Int,
    pub water_color: Int,
    pub water_fog_color: Int,
    pub particle: Option<Particle>,
    pub ambient_sound: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MoodSound {
    pub block_search_extent: Int,
    pub offset: Double,
    pub sound: String,
    pub tick_delay: Int,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Music {
    pub max_delay: Int,
    pub min_delay: Int,
    pub replace_current_music: Boolean,
    pub sound: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Particle {
    pub options: ParticleOptions,
    pub probability: Float,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParticleOptions {
    pub r#type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Enchantment {}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JukeboxSong {}
