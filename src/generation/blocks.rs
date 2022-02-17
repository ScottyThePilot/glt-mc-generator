use std::borrow::Cow;

use super::Block;

macro_rules! const_block {
  ($base_block:literal) => {
    Block {
      base_block: Cow::Borrowed($base_block),
      extra_block: None
    }
  };
  ($base_block:literal, $extra_block:literal) => {
    Block {
      base_block: Cow::Borrowed($base_block),
      extra_block: Some(Cow::Borrowed($extra_block))
    }
  };
}

pub const GRAVEL: Block = const_block!("minecraft:gravel");
pub const DEEPSLATE: Block = const_block!("minecraft:deepslate");
pub const BEDROCK: Block = const_block!("minecraft:bedrock");

pub const WATER: Block = const_block!("minecraft:water");
pub const SEAGRASS_SHORT: Block = const_block!("minecraft:seagrass", "minecraft:water");
pub const SEAGRASS_TALL_UPPER: Block = const_block!("minecraft:tall_seagrass[half=upper]", "minecraft:water");
pub const SEAGRASS_TALL_LOWER: Block = const_block!("minecraft:tall_seagrass[half=lower]", "minecraft:water");

pub const GRAY_CONCRETE: Block = const_block!("minecraft:gray_concrete");
