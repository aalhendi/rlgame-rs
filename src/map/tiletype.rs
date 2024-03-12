#[derive(PartialEq, Eq, Hash, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum TileType {
    Wall,
    Stalactite,
    Stalagmite,
    Floor,
    DownStairs,
    Road,
    Grass,
    ShallowWater,
    DeepWater,
    WoodFloor,
    Bridge,
    Gravel,
    UpStairs,
}

// TODO(aalhendi): Refactor into impl
pub fn tile_walkable(tt: TileType) -> bool {
    match tt {
        TileType::DeepWater | TileType::Wall => false,
        TileType::Floor
        | TileType::DownStairs
        | TileType::Grass
        | TileType::Road
        | TileType::ShallowWater
        | TileType::WoodFloor
        | TileType::Gravel
        | TileType::UpStairs
        | TileType::Bridge
        | TileType::Stalactite
        | TileType::Stalagmite => true,
    }
}

pub fn tile_opaque(tt: TileType) -> bool {
    matches!(tt, TileType::Wall | TileType::Stalactite | TileType::Stalagmite)
}

pub fn tile_cost(tt: TileType) -> f32 {
    match tt {
        TileType::Road => 0.8,
        TileType::Gravel => 0.9,
        TileType::Grass => 1.1,
        TileType::ShallowWater => 1.2,
        _ => 1.0,
    }
}
