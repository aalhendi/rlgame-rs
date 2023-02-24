use super::{
    BlocksTile, CombatStats, Consumable, InflictsDamage, Item, Monster, Name, Player, Position,
    ProvidesHealing, Ranged, Rect, Renderable, Viewshed, MAPWIDTH,
};
use rltk::{RandomNumberGenerator, RGB};
use specs::prelude::*;

const MAX_MONSTERS: i32 = 4;
const MAX_ITEMS: i32 = 2;

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
        .build()
}

/// Spawns a random monster at a given location
pub fn random_monster(ecs: &mut World, pos: Position) {
    let roll: i32;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1, 2);
    }
    match roll {
        1 => orc(ecs, pos),
        _ => goblin(ecs, pos),
    }
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
        .with(BlocksTile {})
        .with(CombatStats {
            max_hp: 16,
            hp: 16,
            defense: 1,
            power: 4,
        })
        .build();
}

/// Spawns entites in rooms
pub fn spawn_room(ecs: &mut World, room: &Rect) {
    let mut monster_spawn_points: Vec<usize> = Vec::new();
    let mut item_spawn_points: Vec<usize> = Vec::new();

    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_monsters = rng.roll_dice(1, MAX_MONSTERS + 2) - 3;
        let num_items = rng.roll_dice(1, MAX_ITEMS + 2) - 3;

        // Generate monster spawn points
        for _ in 0..num_monsters {
            let mut added = false;
            while !added {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAPWIDTH) + x;
                if !monster_spawn_points.contains(&idx) {
                    monster_spawn_points.push(idx);
                    added = true;
                }
            }
        }

        // Generate item spawn points
        for _ in 0..num_items {
            let mut added = false;
            while !added {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAPWIDTH) + x;
                if !item_spawn_points.contains(&idx) {
                    item_spawn_points.push(idx);
                    added = true;
                }
            }
        }
    }

    // Spawning monsters
    for idx in monster_spawn_points.iter() {
        let pos = Position {
            x: (*idx % MAPWIDTH) as i32,
            y: (*idx / MAPWIDTH) as i32,
        };
        random_monster(ecs, pos)
    }

    // Spawning items
    for idx in item_spawn_points.iter() {
        let pos = Position {
            x: (*idx % MAPWIDTH) as i32,
            y: (*idx / MAPWIDTH) as i32,
        };
        random_item(ecs, pos)
    }
}

/// Spawns a random item at a given location
pub fn random_item(ecs: &mut World, pos: Position) {
    let roll: i32;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1, 2);
    }
    match roll {
        1 => health_potion(ecs, pos),
        _ => magic_missile_scroll(ecs, pos),
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
        .with(Item)
        .with(ProvidesHealing { heal_amount: 8 })
        .with(Consumable)
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
        .with(Item)
        .with(Consumable)
        .with(Ranged { range: 6 })
        .with(InflictsDamage { damage: 8 })
        .build();
}
