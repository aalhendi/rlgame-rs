use specs::prelude::*;
use specs::saveload::{DeserializeComponents, SimpleMarker, SimpleMarkerAllocator};

#[cfg(not(target_arch = "wasm32"))]
use specs::saveload::{MarkedBuilder, SerializeComponents};

use crate::spatial;

use super::components::*;
use std::convert::Infallible as NoError;
use std::path::Path;

// Used to avoid compile fail when over 16 component types...
#[cfg(not(target_arch = "wasm32"))]
macro_rules! serialize_individually {
    ($ecs:expr, $ser:expr, $data:expr, $( $type:ty),*) => {
        $(
        SerializeComponents::<NoError, SimpleMarker<IsSerialized>>::serialize(
            &( $ecs.read_storage::<$type>(), ),
            &$data.0,
            &$data.1,
            &mut $ser,
        )
        .unwrap();
        )*
    };
}

macro_rules! deserialize_individually {
    ($ecs:expr, $de:expr, $data:expr, $( $type:ty),*) => {
        $(
        DeserializeComponents::<NoError, _>::deserialize(
            &mut ( &mut $ecs.write_storage::<$type>(), ),
            &    $data.0, // entities
            &mut $data.1, // marker
            &mut $data.2, // allocater
            &mut $de,
        )
        .unwrap();
        )*
    };
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_game(ecs: &mut World) {
    // Create helper
    let mapcopy = ecs.get_mut::<super::map::Map>().unwrap().clone();
    let dungeon_master = ecs
        .get_mut::<super::map::dungeon::MasterDungeonMap>()
        .unwrap()
        .clone();

    let savehelper = ecs
        .create_entity()
        .with(SerializationHelper { map: mapcopy })
        .marked::<SimpleMarker<IsSerialized>>()
        .build();

    let savehelper2 = ecs
        .create_entity()
        .with(DMSerializationHelper {
            map: dungeon_master,
        })
        .marked::<SimpleMarker<IsSerialized>>()
        .build();

    // Actually serialize
    {
        let data = (
            ecs.entities(),
            ecs.read_storage::<SimpleMarker<IsSerialized>>(),
        );

        let writer = std::fs::File::create("./savegame.json").unwrap();
        let mut serializer = serde_json::Serializer::new(writer);
        serialize_individually!(
            ecs,
            serializer,
            data,
            Position,
            Renderable,
            Player,
            Viewshed,
            Name,
            BlocksTile,
            WantsToMelee,
            Item,
            Consumable,
            Ranged,
            InflictsDamage,
            AreaOfEffect,
            Confusion,
            ProvidesHealing,
            InBackpack,
            WantsToPickupItem,
            WantsToUseItem,
            WantsToDropItem,
            SerializationHelper,
            Equippable,
            ParticleLifetime,
            HungerClock,
            ProvidesFood,
            MagicMapper,
            Hidden,
            EntryTrigger,
            EntityMoved,
            SingleActivation,
            BlocksVisibility,
            Door,
            Quips,
            Attributes,
            Skills,
            Pools,
            Weapon,
            Wearable,
            NaturalAttackDefense,
            LootTable,
            OtherLevelPosition,
            DMSerializationHelper,
            LightSource,
            Initiative,
            MyTurn,
            Faction,
            WantsToApproach,
            WantsToFlee,
            MoveMode,
            Chasing,
            EquipmentChanged,
            Vendor,
            TownPortal,
            TeleportTo,
            ApplyMove,
            ApplyTeleport,
            MagicItem,
            ObfuscatedName,
            IdentifiedItem,
            SpawnParticleBurst,
            SpawnParticleLine,
            CursedItem,
            ProvidesRemoveCurse,
            ProvidesIdentification,
            AttributeBonus,
            Duration,
            StatusEffect,
            KnownSpells,
            WantsToCastSpell,
            SpellTemplate,
            ProvidesMana,
            TeachesSpell,
            Slow,
            DamageOverTime,
            SpecialAbilities,
            TileSize,
            OnDeath,
            AlwaysTargetsSelf,
            Target,
            WantsToShoot
        );
    }

    // Clean up
    ecs.delete_entity(savehelper).expect("Crash on cleanup");
    ecs.delete_entity(savehelper2).expect("Crash on cleanup");
}

pub fn save_exists() -> bool {
    Path::new("./savegame.json").exists()
}

pub fn load_game(ecs: &mut World) {
    {
        // Delete everything
        let mut to_delete = Vec::new();
        for e in ecs.entities().join() {
            to_delete.push(e);
        }
        for del in to_delete.iter() {
            ecs.delete_entity(*del).expect("Deletion failed");
        }
    }

    let data = std::fs::read_to_string("./savegame.json").unwrap();
    let mut de = serde_json::Deserializer::from_str(&data);

    {
        let mut d = (
            &mut ecs.entities(),
            &mut ecs.write_storage::<SimpleMarker<IsSerialized>>(),
            &mut ecs.write_resource::<SimpleMarkerAllocator<IsSerialized>>(),
        );

        deserialize_individually!(
            ecs,
            de,
            d,
            Position,
            Renderable,
            Player,
            Viewshed,
            Name,
            BlocksTile,
            WantsToMelee,
            Item,
            Consumable,
            Ranged,
            InflictsDamage,
            AreaOfEffect,
            Confusion,
            ProvidesHealing,
            InBackpack,
            WantsToPickupItem,
            WantsToUseItem,
            WantsToDropItem,
            SerializationHelper,
            Equippable,
            ParticleLifetime,
            HungerClock,
            ProvidesFood,
            MagicMapper,
            Hidden,
            EntryTrigger,
            EntityMoved,
            SingleActivation,
            BlocksVisibility,
            Door,
            Quips,
            Attributes,
            Skills,
            Pools,
            Weapon,
            Wearable,
            NaturalAttackDefense,
            LootTable,
            OtherLevelPosition,
            DMSerializationHelper,
            LightSource,
            Initiative,
            MyTurn,
            Faction,
            WantsToApproach,
            WantsToFlee,
            MoveMode,
            Chasing,
            EquipmentChanged,
            Vendor,
            TownPortal,
            TeleportTo,
            MagicItem,
            ObfuscatedName,
            IdentifiedItem,
            SpawnParticleBurst,
            SpawnParticleLine,
            CursedItem,
            ProvidesRemoveCurse,
            ProvidesIdentification,
            AttributeBonus,
            Duration,
            StatusEffect,
            KnownSpells,
            WantsToCastSpell,
            SpellTemplate,
            ProvidesMana,
            TeachesSpell,
            Slow,
            DamageOverTime,
            SpecialAbilities,
            TileSize,
            OnDeath,
            AlwaysTargetsSelf,
            Target,
            WantsToShoot
        );
    }

    let mut deleteme: Option<Entity> = None;
    let mut deleteme2: Option<Entity> = None;
    {
        let entities = ecs.entities();
        let helper = ecs.read_storage::<SerializationHelper>();
        let helper2 = ecs.read_storage::<DMSerializationHelper>();
        let player = ecs.read_storage::<Player>();
        let position = ecs.read_storage::<Position>();
        for (e, h) in (&entities, &helper).join() {
            let mut worldmap = ecs.write_resource::<super::map::Map>();
            *worldmap = h.map.clone();
            spatial::set_size((worldmap.height * worldmap.width) as usize);
            deleteme = Some(e);
        }
        for (e, h) in (&entities, &helper2).join() {
            let mut dungeonmaster = ecs.write_resource::<super::map::dungeon::MasterDungeonMap>();
            *dungeonmaster = h.map.clone();
            deleteme2 = Some(e);
        }
        for (e, _p, pos) in (&entities, &player, &position).join() {
            let mut ppos = ecs.write_resource::<rltk::Point>();
            *ppos = rltk::Point::new(pos.x, pos.y);
            let mut player_resource = ecs.write_resource::<Entity>();
            *player_resource = e;
        }
    }
    ecs.delete_entity(deleteme.unwrap())
        .expect("Unable to delete helper");
    ecs.delete_entity(deleteme2.unwrap())
        .expect("Unable to delete helper");
}

pub fn delete_save() {
    if Path::new("./savegame.json").exists() {
        std::fs::remove_file("./savegame.json").expect("Unable to delete file");
    }
}

// TODO: Impl via web-sys crate and localStorage on browser
#[cfg(target_arch = "wasm32")]
pub fn save_game(_ecs: &mut World) {}
