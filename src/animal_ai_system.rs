use rltk::Point;
use specs::{Entities, Entity, Join, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage};

use crate::{
    Carnivore, EntityMoved, Herbivore, Item, Map, Position, RunState, Viewshed, WantsToMelee,
};

pub struct AnimalAISystem;

impl<'a> System<'a> for AnimalAISystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadExpect<'a, Entity>,
        ReadExpect<'a, RunState>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Herbivore>,
        ReadStorage<'a, Carnivore>,
        ReadStorage<'a, Item>,
        WriteStorage<'a, WantsToMelee>,
        WriteStorage<'a, EntityMoved>,
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map,
            player_entity,
            runstate,
            entities,
            mut viewshed,
            herbivores,
            carnivores,
            item,
            mut wants_to_melee,
            mut entity_moved,
            mut position,
        ) = data;

        if *runstate != RunState::MonsterTurn {
            return;
        }

        // Herbivores run away
        for (entity, viewshed, _herbivore, pos) in
            (&entities, &mut viewshed, &herbivores, &mut position).join()
        {
            let mut run_away_from = Vec::new();
            for other_tile in viewshed.visible_tiles.iter() {
                let view_idx = map.xy_idx(other_tile.x, other_tile.y);
                for other_entity in map.tile_content[view_idx].iter() {
                    // They don't run away from items
                    if item.get(*other_entity).is_none() {
                        run_away_from.push(view_idx);
                    }
                }
            }

            if !run_away_from.is_empty() {
                let my_idx = map.xy_idx(pos.x, pos.y);
                map.populate_blocked();
                let flee_map =
                    rltk::DijkstraMap::new(map.width, map.height, &run_away_from, &*map, 100.0);
                let flee_target = rltk::DijkstraMap::find_highest_exit(&flee_map, my_idx, &*map);
                if let Some(flee_target) = flee_target {
                    // if tgt tile not blocked, free current tile, block tgt tile
                    if !map.blocked[flee_target] {
                        map.blocked[my_idx] = false;
                        map.blocked[flee_target] = true;
                        viewshed.dirty = true;
                        let (new_x, new_y) = map.idx_xy(flee_target);
                        pos.x = new_x;
                        pos.y = new_y;
                        entity_moved
                            .insert(entity, EntityMoved {})
                            .expect("Unable to insert marker");
                    }
                }
            }
        }

        // Carnivores attack everything
        for (entity, viewshed, _carnivore, pos) in
            (&entities, &mut viewshed, &carnivores, &mut position).join()
        {
            let mut run_towards: Vec<usize> = Vec::new();
            let mut attacked = false;
            for other_tile in viewshed.visible_tiles.iter() {
                let view_idx = map.xy_idx(other_tile.x, other_tile.y);
                for other_entity in map.tile_content[view_idx].iter() {
                    // if other is a herbivore or player, chase or attack
                    if herbivores.get(*other_entity).is_some() || *other_entity == *player_entity {
                        let distance = rltk::DistanceAlg::Pythagoras
                            .distance2d(Point::new(pos.x, pos.y), *other_tile);
                        if distance < 1.5 {
                            wants_to_melee
                                .insert(
                                    entity,
                                    WantsToMelee {
                                        target: *other_entity,
                                    },
                                )
                                .expect("Unable to insert attack");
                            attacked = true;
                        } else {
                            run_towards.push(view_idx);
                        }
                    }
                }
            }

            // Nothing to chase or attack
            if run_towards.is_empty() || attacked {
                continue;
            }

            let player_idx = map.xy_idx(pos.x, pos.y);
            map.populate_blocked();
            let chase_map =
                rltk::DijkstraMap::new(map.width, map.height, &run_towards, &*map, 100.0);
            let chase_target = rltk::DijkstraMap::find_lowest_exit(&chase_map, player_idx, &*map);
            if let Some(chase_target) = chase_target {
                if !map.blocked[chase_target] {
                    map.blocked[player_idx] = false;
                    map.blocked[chase_target] = true;
                    viewshed.dirty = true;
                    let (new_x, new_y) = map.idx_xy(chase_target);
                    pos.x = new_x;
                    pos.y = new_y;
                    entity_moved
                        .insert(entity, EntityMoved {})
                        .expect("Unable to insert marker");
                }
            }
        }
    }
}
