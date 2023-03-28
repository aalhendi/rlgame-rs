use super::{BuilderMap, InitialMapBuilder, Map};
use crate::TileType;
use rltk::RandomNumberGenerator;

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

    fn generate_maze(&mut self, build_data: &mut BuilderMap) {
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
                self.copy_to_map(&mut build_data.map);
                build_data.take_snapshot();
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

pub struct MazeBuilder {}

impl InitialMapBuilder for MazeBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl MazeBuilder {
    pub fn new() -> Box<MazeBuilder> {
        Box::new(MazeBuilder {})
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        // Maze gen
        let mut maze = Grid::new(
            (build_data.map.width / 2) - 2,
            (build_data.map.height / 2) - 2,
            rng,
        );
        maze.generate_maze(build_data);
    }
}
