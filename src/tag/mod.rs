use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TagData {
    #[serde(rename = "minecraft:banner_pattern")]
    pub banner_patterns: Vec<TagGroup>,
    #[serde(rename = "minecraft:block")]
    pub blocks: Vec<TagGroup>,
    #[serde(rename = "minecraft:cat_variant")]
    pub cat_variants: Vec<TagGroup>,
    #[serde(rename = "minecraft:damage_type")]
    pub damage_types: Vec<TagGroup>,
    #[serde(rename = "minecraft:enchantment")]
    pub enchantments: Vec<TagGroup>,
    #[serde(rename = "minecraft:entity_type")]
    pub entity_types: Vec<TagGroup>,
    #[serde(rename = "minecraft:fluid")]
    pub fluids: Vec<TagGroup>,
    #[serde(rename = "minecraft:game_event")]
    pub game_events: Vec<TagGroup>,
    #[serde(rename = "minecraft:instrument")]
    pub instruments: Vec<TagGroup>,
    #[serde(rename = "minecraft:item")]
    pub items: Vec<TagGroup>,
    #[serde(rename = "minecraft:painting_variant")]
    pub painting_variants: Vec<TagGroup>,
    #[serde(rename = "minecraft:point_of_interest_type")]
    pub point_of_interest_types: Vec<TagGroup>,
    #[serde(rename = "minecraft:worldgen/biome")]
    pub worldgen_biomes: Vec<TagGroup>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TagName {
    pub name: String,
    pub namespace: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TagGroup {
    pub entries: Vec<i32>,
    pub tag_name: TagName,
}
