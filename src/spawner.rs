use std::collections::HashMap;

use super::{
    AreaOfEffect, BlocksTile, CombatStats, Confusion, Consumable, DefenseBonus, EquipmentSlot,
    Equippable, InflictsDamage, IsSerialized, Item, MeleePowerBonus, Monster, Name, Player,
    Position, ProvidesHealing, Ranged, Rect, Renderable, Viewshed, MAPWIDTH,
};
use crate::random_table::RandomTable;
use rltk::{RandomNumberGenerator, RGB};
use specs::{
    prelude::*,
    saveload::{MarkedBuilder, SimpleMarker},
};

const MAX_MONSTERS: i32 = 4;

/// Spawns the player and returns its entity object
pub fn player(ecs: &mut World, player_pos: Position) -> Entity {
    ecs.create_entity()
        .with(player_pos)
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 0,
        })
        .with(Player {})
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(Name {
            name: "Player".to_string(),
        })
        .with(CombatStats {
            max_hp: 30,
            hp: 30,
            defense: 2,
            power: 5,
        })
        .marked::<SimpleMarker<IsSerialized>>()
        .build()
}

fn orc(ecs: &mut World, pos: Position) {
    monster(ecs, pos, rltk::to_cp437('o'), "Orc");
}
fn goblin(ecs: &mut World, pos: Position) {
    monster(ecs, pos, rltk::to_cp437('g'), "Goblin");
}

fn monster<S: ToString>(ecs: &mut World, pos: Position, glyph: rltk::FontCharType, name: S) {
    ecs.create_entity()
        .with(pos)
        .with(Renderable {
            glyph,
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            render_order: 1,
        })
        .with(Monster {})
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(Name {
            name: name.to_string(),
        })
        .with(BlocksTile)
        .with(CombatStats {
            max_hp: 16,
            hp: 16,
            defense: 1,
            power: 4,
        })
        .marked::<SimpleMarker<IsSerialized>>()
        .build();
}

/// Spawns entites in rooms
#[allow(clippy::map_entry)]
pub fn spawn_room(ecs: &mut World, room: &Rect, map_depth: i32) {
    let spawn_table = room_table(map_depth);
    let mut spawn_points: HashMap<usize, String> = HashMap::new();

    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_spawns = rng.roll_dice(1, MAX_MONSTERS + 3) + (map_depth - 1) - 3;

        // Generate spawn points
        for _ in 0..num_spawns {
            let mut tries = 0;
            while tries < 20 {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAPWIDTH) + x;
                if !spawn_points.contains_key(&idx) {
                    spawn_points.insert(idx, spawn_table.roll(&mut rng));
                    break;
                } else {
                    tries += 1;
                }
            }
        }
    }

    // Spawning things
    for (idx, name) in spawn_points.iter() {
        let pos = Position {
            x: (*idx % MAPWIDTH) as i32,
            y: (*idx / MAPWIDTH) as i32,
        };

        match name.as_ref() {
            "Goblin" => goblin(ecs, pos),
            "Orc" => orc(ecs, pos),
            "Health Potion" => health_potion(ecs, pos),
            "Fireball Scroll" => fireball_scroll(ecs, pos),
            "Confusion Scroll" => confusion_scroll(ecs, pos),
            "Magic Missile Scroll" => magic_missile_scroll(ecs, pos),
            "Dagger" => dagger(ecs, pos),
            "Shield" => shield(ecs, pos),
            _ => {}
        }
    }
}

fn health_potion(ecs: &mut World, pos: Position) {
    ecs.create_entity()
        .with(pos)
        .with(Renderable {
            glyph: rltk::to_cp437('¡'),
            fg: RGB::named(rltk::MAGENTA),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Health Potion".to_string(),
        })
        .with(Item {})
        .with(ProvidesHealing { heal_amount: 8 })
        .with(Consumable {})
        .marked::<SimpleMarker<IsSerialized>>()
        .build();
}

fn magic_missile_scroll(ecs: &mut World, pos: Position) {
    ecs.create_entity()
        .with(pos)
        .with(Renderable {
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Magic Missile Scroll".to_string(),
        })
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(InflictsDamage { damage: 8 })
        .marked::<SimpleMarker<IsSerialized>>()
        .build();
}

fn fireball_scroll(ecs: &mut World, pos: Position) {
    ecs.create_entity()
        .with(pos)
        .with(Renderable {
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::ORANGE),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Fireball Scroll".to_string(),
        })
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(InflictsDamage { damage: 20 })
        .with(AreaOfEffect { radius: 3 })
        .marked::<SimpleMarker<IsSerialized>>()
        .build();
}

fn confusion_scroll(ecs: &mut World, pos: Position) {
    ecs.create_entity()
        .with(pos)
        .with(Renderable {
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::PINK),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Confusion Scroll".to_string(),
        })
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(Confusion { turns: 4 })
        .marked::<SimpleMarker<IsSerialized>>()
        .build();
}

fn dagger(ecs: &mut World, pos: Position) {
    ecs.create_entity()
        .with(pos)
        .with(Renderable {
            glyph: rltk::to_cp437('/'),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Dagger".to_string(),
        })
        .with(Item {})
        .with(Equippable {
            slot: EquipmentSlot::Melee,
        })
        .with(MeleePowerBonus { amount: 2 })
        .marked::<SimpleMarker<IsSerialized>>()
        .build();
}

fn shield(ecs: &mut World, pos: Position) {
    ecs.create_entity()
        .with(pos)
        .with(Renderable {
            glyph: rltk::to_cp437('('),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name {
            name: "Shield".to_string(),
        })
        .with(Item {})
        .with(Equippable {
            slot: EquipmentSlot::Shield,
        })
        .with(DefenseBonus { amount: 1 })
        .marked::<SimpleMarker<IsSerialized>>()
        .build();
}

fn room_table(map_depth: i32) -> RandomTable {
    RandomTable::new()
        .add("Goblin", 10)
        .add("Orc", 1 + map_depth)
        .add("Health Potion", 7)
        .add("Fireball Scroll", 2 + map_depth)
        .add("Confusion Scroll", 2 + map_depth)
        .add("Magic Missile Scroll", 4)
        .add("Dagger", 3)
        .add("Shield", 3)
}
