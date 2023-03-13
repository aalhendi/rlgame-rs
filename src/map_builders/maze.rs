use super::{
    common::generate_voronoi_spawn_regions, common::remove_unreachable_areas_get_most_distant, Map,
    MapBuilder,
};
use crate::{spawner, Position, TileType, SHOW_MAPGEN_VISUALIZER};
use rltk::RandomNumberGenerator;
use specs::World;
use std::collections::HashMap;

#[derive(Copy, Clone)]
struct NeighborWalls {
    top: bool,
    right: bool,
    bottom: bool,
    left: bool,
}

#[derive(Copy, Clone)]
struct Cell {
    row: i32,
    column: i32,
    walls: NeighborWalls,
    visited: bool,
}

impl Cell {
    fn new(row: i32, column: i32) -> Cell {
        Cell {
            row,
            column,
            // walls: [true, true, true, true],
            walls: NeighborWalls {
                top: true,
                right: true,
                bottom: true,
                left: true,
            },
            visited: false,
        }
    }

    fn remove_walls(&mut self, next: &mut Cell) {
        let x = self.column - next.column;
        let y = self.row - next.row;

        if x == 1 {
            self.walls.left = false;
            next.walls.right = false;
        } else if x == -1 {
            self.walls.right = false;
            next.walls.left = false;
        } else if y == 1 {
            self.walls.top = false;
            next.walls.bottom = false;
        } else if y == -1 {
            self.walls.bottom = false;
            self.walls.top = false;
        }
    }
}

/// Inspired by <https://github.com/cyucelen/mazeGenerator/>  [MIT LICENSE]
struct Grid<'a> {
    width: i32,
    height: i32,
    cells: Vec<Cell>,
    backtrace: Vec<usize>,
    current: usize,
    rng: &'a mut RandomNumberGenerator,
}

impl<'a> Grid<'a> {
    fn new(width: i32, height: i32, rng: &mut RandomNumberGenerator) -> Grid {
        let mut grid = Grid {
            width,
            height,
            cells: Vec::new(),
            backtrace: Vec::new(),
            current: 0,
            rng,
        };

        for row in 0..height {
            for column in 0..width {
                grid.cells.push(Cell::new(row, column));
            }
        }

        grid
    }

    /// Returns cell index given row and column if invalid
    fn calculate_index(&self, row: i32, column: i32) -> Option<i32> {
        if row < 0 || column < 0 || column > self.width - 1 || row > self.height - 1 {
            None
        } else {
            Some(column + (row * self.width))
        }
    }

    fn get_available_neighbors(&self) -> Vec<usize> {
        let mut neighbors: Vec<usize> = Vec::new();

        let cur_row = self.cells[self.current].row;
        let cur_col = self.cells[self.current].column;

        let neighbor_indices: [Option<i32>; 4] = [
            self.calculate_index(cur_row - 1, cur_col),
            self.calculate_index(cur_row, cur_col + 1),
            self.calculate_index(cur_row + 1, cur_col),
            self.calculate_index(cur_row, cur_col - 1),
        ];

        for i in neighbor_indices.iter().flatten() {
            if !self.cells[*i as usize].visited {
                neighbors.push(*i as usize)
            }
        }

        neighbors
    }

    fn find_next_cell(&mut self) -> Option<usize> {
        let neighbors = self.get_available_neighbors();
        if neighbors.is_empty() {
            return None;
        }

        if neighbors.len() == 1 {
            Some(neighbors[0])
        } else {
            Some(neighbors[(self.rng.roll_dice(1, neighbors.len() as i32) - 1) as usize])
        }
    }

    fn generate_maze(&mut self, generator: &mut MazeBuilder) {
        let mut i = 0;
        loop {
            self.cells[self.current].visited = true;

            if let Some(next) = self.find_next_cell() {
                self.cells[next].visited = true;
                self.backtrace.push(self.current);
                //   __lower_part__      __higher_part_
                //   /            \      /            \
                // --------cell1------ | cell2-----------
                let (lower_part, higher_part) =
                    self.cells.split_at_mut(std::cmp::max(self.current, next));
                let cell1 = &mut lower_part[std::cmp::min(self.current, next)];
                let cell2 = &mut higher_part[0];
                cell1.remove_walls(cell2);
                self.current = next;
            } else if !self.backtrace.is_empty() {
                // More cells to go, recurse
                self.current = self.backtrace[0];
                self.backtrace.remove(0);
            } else {
                break;
            }

            // Periodic snapshots, don't want too many
            if i % 50 == 0 {
                self.copy_to_map(&mut generator.map);
                generator.take_snapshot();
            }
            i += 1;
        }
    }

    /// Copies the maze to the map, takes snapshot for iterative map generation renderer.
    fn copy_to_map(&self, map: &mut Map) {
        // Clear the map
        for i in map.tiles.iter_mut() {
            *i = TileType::Wall;
        }

        for cell in self.cells.iter() {
            let x = cell.column + 1;
            let y = cell.row + 1;
            let idx = map.xy_idx(x * 2, y * 2);

            map.tiles[idx] = TileType::Floor;
            if !cell.walls.top {
                map.tiles[idx - map.width as usize] = TileType::Floor
            }
            if !cell.walls.right {
                map.tiles[idx + 1] = TileType::Floor
            }
            if !cell.walls.bottom {
                map.tiles[idx + map.width as usize] = TileType::Floor
            }
            if !cell.walls.left {
                map.tiles[idx - 1] = TileType::Floor
            }
        }
    }
}

pub struct MazeBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
}

impl MapBuilder for MazeBuilder {
    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&self) -> Position {
        self.starting_position.clone()
    }

    fn get_snapshot_history(&self) -> Vec<Map> {
        self.history.clone()
    }

    fn build_map(&mut self) {
        self.build();
    }

    fn spawn_entities(&mut self, ecs: &mut World) {
        for (_area_id, tile_ids) in self.noise_areas.iter() {
            spawner::spawn_region(ecs, tile_ids, self.depth);
        }
    }

    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }
}

impl MazeBuilder {
    pub fn new(new_depth: i32) -> MazeBuilder {
        MazeBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
        }
    }

    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        // Maze gen
        let mut maze = Grid::new(
            (self.map.width / 2) - 2,
            (self.map.height / 2) - 2,
            &mut rng,
        );
        maze.generate_maze(self);

        // Start at top-left of map
        self.starting_position = Position { x: 2, y: 2 };
        let start_idx = self
            .map
            .xy_idx(self.starting_position.x, self.starting_position.y);
        self.take_snapshot();

        let exit_tile_idx = remove_unreachable_areas_get_most_distant(&mut self.map, start_idx);
        self.take_snapshot();

        self.map.tiles[exit_tile_idx] = TileType::DownStairs;
        self.take_snapshot();

        //Build noise map for spawning entities later
        self.noise_areas = generate_voronoi_spawn_regions(&self.map, &mut rng);
    }
}
