use super::{Rect, Room};
use rltk::{Algorithm2D, BaseMap, Point, RandomNumberGenerator, Rltk, RGB};
use specs::prelude::*;
use std::cmp::{max, min};

#[cfg(test)]
mod tests {
    use super::*;

    /// All border tiles should be of TileType::Wall
    fn borders_check(map: Map) -> Result<(), String> {
        // iterate over all border tiles (x == 0, x == width - 1,
        // y == 0, y == height - 1)
        let cond = map
            .tiles
            .iter()
            .enumerate()
            .filter(|&(pt, _)| {
                (pt as i32) < map.width
                    || (pt as i32) > (map.tiles.len() as i32) - map.width
                    || (pt as i32) % map.width == 0
                    || (pt as i32) % map.width == map.width - 1
            })
            .all(|(_, tile)| tile == &TileType::Wall);

        if cond {
            Ok(())
        } else {
            Err("Border has non-wall tile".to_string())
        }
    }

    /// tests that all mapgen algorithms produce maps will
    /// preserve all outer walls (borders)
    #[test]
    fn test_mapgen_algos_border() -> Result<(), String> {
        let algo_list = [
            Map::new,
            Map::new_map_rooms_and_corridors,
            Map::new_map_all_open,
        ];

        for algo in algo_list.iter() {
            // tests with 30 randomly generated maps
            for _ in 0..30 {
                borders_check(algo(0))?;
            }
        }
        Ok(())
    }

    #[test]
    fn borders_check_new() -> Result<(), String> {
        let map = Map::new(0);
        borders_check(map)?;
        Ok(())
    }

    #[test]
    fn borders_check_all_open() -> Result<(), String> {
        let map = Map::new_map_all_open(0);
        borders_check(map)?;
        Ok(())
    }

    #[test]
    #[ignore]
    fn borders_check_corner_ul() {
        let mut map = Map::new_map_all_open(0);
        map.tiles[0] = TileType::Floor;

        assert!(borders_check(map).is_err());
    }

    #[test]
    #[ignore]
    fn borders_check_corner_ur() {
        let mut map = Map::new_map_all_open(0);
        map.tiles[(map.width - 1) as usize] = TileType::Floor;

        assert!(borders_check(map).is_err());
    }

    #[test]
    #[ignore]
    fn borders_check_corner_bl() {
        let mut map = Map::new_map_all_open(0);
        let len = map.tiles.len() as i32;
        map.tiles[(len - map.width + 1) as usize] = TileType::Floor;

        assert!(borders_check(map).is_err());
    }

    #[test]
    #[ignore]
    fn borders_check_corner_br() {
        let mut map = Map::new_map_all_open(0);
        let len = map.tiles.len() as i32;
        map.tiles[(len - 1) as usize] = TileType::Floor;

        assert!(borders_check(map).is_err());
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

pub struct Map {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub depth: i32,
}

impl Map {
    fn new(new_depth: i32) -> Map {
        Map {
            tiles: vec![TileType::Wall; 80 * 50],
            rooms: Vec::new(),
            width: 80,
            height: 50,
            revealed_tiles: vec![false; 80 * 50],
            visible_tiles: vec![false; 80 * 50],
            depth: new_depth,
        }
    }

    fn new_with_dimensions(w: usize, h: usize, new_depth: i32) -> Map {
        Map {
            tiles: vec![TileType::Wall; w * h],
            rooms: Vec::new(),
            width: w as i32,
            height: h as i32,
            revealed_tiles: vec![false; w * h],
            visible_tiles: vec![false; w * h],
            depth: new_depth,
        }
    }

    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    fn apply_room_to_map(&mut self, room: &dyn Room) {
        for (x, y) in room.spaces().iter() {
            let idx = self.xy_idx(*x, *y);
            self.tiles[idx] = TileType::Floor;
        }
    }

    /// "Digs out" a horizontal corridor (changes Wall -> Floor)
    /// given x1, x2, y
    fn apply_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        for x in min(x1, x2)..=max(x1, x2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < self.width as usize * self.height as usize {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    /// "Digs out" a vertical corridor (changes Wall -> Floor)
    /// given y1, y2, x
    fn apply_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2)..=max(y1, y2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < self.width as usize * self.height as usize {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    pub fn new_map_all_open(new_depth: i32) -> Map {

        let mut map = Map::new(new_depth);

        let new_room = Rect::new(0, 0, 78, 48);
        map.apply_room_to_map(&new_room);
        map.rooms.push(new_room);

        map
    }

    /// Makes a new map using the algorithm from http://rogueliketutorials.com/tutorials/tcod/part-3/
    /// This gives a handful of random rooms and corridors joining them together.
    pub fn new_map_rooms_and_corridors(new_depth: i32) -> Map {
        // base map
        let mut map = Map::new_with_dimensions(80, 50, new_depth);

        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        let mut rng = RandomNumberGenerator::new();

        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, map.width - w - 1) - 1;
            let y = rng.roll_dice(1, map.height - h - 1) - 1;
            let new_room = Rect::new(x, y, w, h);
            let mut ok = true;
            for other_room in map.rooms.iter() {
                if new_room.intersect(other_room) {
                    ok = false;
                }
            }
            if ok {
                map.apply_room_to_map(&new_room);

                if !map.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = map.rooms[map.rooms.len() - 1].center();
                    if rng.range(0, 2) == 1 {
                        map.apply_horizontal_tunnel(prev_x, new_x, prev_y);
                        map.apply_vertical_tunnel(prev_y, new_y, new_x);
                    } else {
                        map.apply_vertical_tunnel(prev_y, new_y, prev_x);
                        map.apply_horizontal_tunnel(prev_x, new_x, new_y);
                    }
                }

                map.rooms.push(new_room);
            }
        }

        map
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx as usize] == TileType::Wall
    }
}

pub fn draw_map(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();

    let mut y = 0;
    let mut x = 0;
    for (idx, tile) in map.tiles.iter().enumerate() {
        // Render a tile depending upon the tile type

        if map.revealed_tiles[idx] {
            let glyph;
            let mut fg;
            match tile {
                TileType::Floor => {
                    glyph = rltk::to_cp437('.');
                    fg = RGB::from_f32(0.0, 0.5, 0.5);
                }
                TileType::Wall => {
                    glyph = rltk::to_cp437('#');
                    fg = RGB::from_f32(0., 1.0, 0.);
                }
            }
            if !map.visible_tiles[idx] {
                fg = fg.to_greyscale()
            }
            ctx.set(x, y, fg, RGB::from_f32(0., 0., 0.), glyph);
        }

        // Move the coordinates
        x += 1;
        if x > 79 {
            x = 0;
            y += 1;
        }
    }
}
