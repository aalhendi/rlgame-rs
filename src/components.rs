use rltk::{Point, RGB};
use serde::{Deserialize, Serialize};
use specs::{
    prelude::*,
    saveload::{ConvertSaveload, Marker},
};
use specs_derive::*;
use std::{collections::HashMap, convert::Infallible as NoError};

use crate::map;

#[derive(Component, ConvertSaveload, Clone, Default, Copy)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

// TODO: Impl TryFrom and TryInto for Point/Position
impl From<Position> for Point {
    fn from(value: Position) -> Self {
        Point {
            x: value.x,
            y: value.y,
        }
    }
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order: i32,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Player {}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Viewshed {
    pub visible_tiles: Vec<Point>,
    pub range: i32,
    pub dirty: bool,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Name {
    pub name: String,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct BlocksTile;

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct WantsToMelee {
    pub target: Entity,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Item {
    pub initiative_penalty: f32,
    pub weight_lbs: f32,
    pub base_value: f32,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct ProvidesHealing {
    pub heal_amount: i32,
}

#[derive(Component, Debug, ConvertSaveload)]
pub struct InBackpack {
    pub owner: Entity,
}

impl Owned for InBackpack {
    fn owned_by(&self, entity: &Entity) -> bool {
        self.owner == *entity
    }
}

#[derive(Component, Debug, ConvertSaveload)]
pub struct WantsToPickupItem {
    pub collected_by: Entity,
    pub item: Entity,
}
#[derive(Component, Debug, ConvertSaveload)]
pub struct WantsToUseItem {
    pub item: Entity,
    pub target: Option<Point>,
}

#[derive(Component, Debug, ConvertSaveload)]
pub struct WantsToDropItem {
    pub item: Entity,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Consumable {
    pub max_charges: i32,
    pub charges: i32,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Ranged {
    pub range: i32,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct InflictsDamage {
    pub damage: i32,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct AreaOfEffect {
    pub radius: i32,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Confusion {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Duration {
    pub turns: i32,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct StatusEffect {
    pub target: Entity,
}

pub struct IsSerialized;

// Special component that exists to help serialize the game data
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SerializationHelper {
    pub map: map::Map,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct DMSerializationHelper {
    pub map: map::dungeon::MasterDungeonMap,
}

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum EquipmentSlot {
    Melee,
    Shield,
    Head,
    Torso,
    Legs,
    Feet,
    Hands,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Equippable {
    pub slot: EquipmentSlot,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Equipped {
    pub owner: Entity,
    pub slot: EquipmentSlot,
}

impl Owned for Equipped {
    fn owned_by(&self, entity: &Entity) -> bool {
        self.owner == *entity
    }
}

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum WeaponAttribute {
    Might,
    Quickness,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Weapon {
    pub attribute: WeaponAttribute,
    pub damage_n_dice: i32,
    pub damage_die_type: i32,
    pub damage_bonus: i32,
    pub hit_bonus: i32,
    pub proc_chance: Option<f32>,
    pub proc_target: Option<String>,
    pub range: Option<i32>,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Wearable {
    pub armor_class: f32,
    pub slot: EquipmentSlot,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct WantsToRemoveItem {
    pub item: Entity,
}

pub trait Owned {
    fn owned_by(&self, entity: &Entity) -> bool;
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct ParticleLifetime {
    pub lifetime_ms: f32,
    pub animation: Option<ParticleAnimation>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ParticleAnimation {
    pub step_time: f32,
    pub path: Vec<Point>,
    pub current_step: usize,
    pub timer: f32,
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq)]
pub enum HungerState {
    WellFed,
    Normal,
    Hungry,
    Starving,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct HungerClock {
    pub state: HungerState,
    pub duration: i32,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct ProvidesFood {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct MagicMapper {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Hidden {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct EntryTrigger {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct EntityMoved {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct SingleActivation {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct BlocksVisibility {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Door {
    pub open: bool,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Quips {
    pub available: Vec<String>,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
// Not actually a component, its just used by one. Doesn't need to be registered in saveload and main
pub struct Attribute {
    pub base: i32,
    pub modifiers: i32,
    pub bonus: i32,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Attributes {
    pub might: Attribute,
    pub fitness: Attribute,
    pub quickness: Attribute,
    pub intelligence: Attribute,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
// Not actually a component, its just used by one. Doesn't need to be registered in saveload and main
pub enum Skill {
    Melee,
    Defense,
    Magic,
}

#[derive(Default, Component, Debug, Serialize, Deserialize, Clone)]
pub struct Skills {
    pub skills: HashMap<Skill, i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
// Not actually a component, its just used by one. Doesn't need to be registered in saveload and main
pub struct Pool {
    pub max: i32,
    pub current: i32,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Pools {
    pub hit_points: Pool,
    pub mana: Pool,
    pub xp: i32,
    pub level: i32,
    pub total_weight: f32,
    pub total_initiative_penalty: f32,
    pub gold: f32,
    pub god_mode: bool,
}

#[derive(Serialize, Deserialize, Clone)]
// Not actually a component, its just used by one. Doesn't need to be registered in saveload and main
pub struct NaturalAttack {
    pub name: String,
    pub damage_n_dice: i32,
    pub damage_die_type: i32,
    pub damage_bonus: i32,
    pub hit_bonus: i32,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct NaturalAttackDefense {
    pub armor_class: Option<i32>,
    pub attacks: Vec<NaturalAttack>,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct LootTable {
    pub name: String,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct OtherLevelPosition {
    // TODO(aalhendi): Can this be a Position directly?
    pub x: i32,
    pub y: i32,
    pub depth: i32,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct LightSource {
    pub color: RGB,
    pub range: i32,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Initiative {
    pub current: i32,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct MyTurn {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Faction {
    pub name: String,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct WantsToApproach {
    pub idx: i32,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct WantsToFlee {
    pub indices: Vec<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub enum Movement {
    Static,
    Random,
    RandomWaypoint { path: Option<Vec<usize>> },
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct MoveMode {
    pub mode: Movement,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Chasing {
    pub target: Entity,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct EquipmentChanged {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Vendor {
    pub categories: Vec<String>,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct TownPortal {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct TeleportTo {
    pub x: i32,
    pub y: i32,
    pub depth: i32,
    pub player_only: bool,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct ApplyMove {
    pub dest_idx: usize,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct ApplyTeleport {
    pub dest_x: i32,
    pub dest_y: i32,
    pub dest_depth: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub enum MagicItemClass {
    Common,
    Rare,
    Legendary,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct MagicItem {
    pub class: MagicItemClass,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct ObfuscatedName {
    pub name: String,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct IdentifiedItem {
    pub name: String,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SpawnParticleLine {
    pub glyph: rltk::FontCharType,
    pub color: RGB,
    pub lifetime_ms: f32,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SpawnParticleBurst {
    pub glyph: rltk::FontCharType,
    pub color: RGB,
    pub lifetime_ms: f32,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct CursedItem {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct ProvidesRemoveCurse {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct ProvidesIdentification {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct AttributeBonus {
    pub might: Option<i32>,
    pub fitness: Option<i32>,
    pub quickness: Option<i32>,
    pub intelligence: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KnownSpell {
    pub display_name: String,
    pub mana_cost: i32,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct KnownSpells {
    pub spells: Vec<KnownSpell>,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct SpellTemplate {
    pub mana_cost: i32,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct WantsToCastSpell {
    pub spell: Entity,
    pub target: Option<Point>,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct ProvidesMana {
    pub mana_amount: i32,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct TeachesSpell {
    pub spell: String,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Slow {
    pub initiative_penalty: f32,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct DamageOverTime {
    pub damage: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpecialAbility {
    pub spell: String,
    pub chance: f32,
    pub range: f32,
    pub min_range: f32,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone, Default)]
pub struct SpecialAbilities {
    pub abilities: Vec<SpecialAbility>,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct TileSize {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone, Default)]
pub struct OnDeath {
    pub abilities: Vec<SpecialAbility>,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct AlwaysTargetsSelf {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Target {}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct WantsToShoot {
    pub target: Entity,
}
