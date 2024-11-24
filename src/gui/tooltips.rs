use std::cmp::Ordering;

use rltk::{ColorPair, DrawBatch, Point, Rect, Rltk, RGB};
use specs::{Join, World, WorldExt};

use crate::{
    map::camera::{self, PANE_WIDTH},
    spatial, Attributes, Duration, Hidden, Map, Name, Pools, StatusEffect,
};

use super::item_render::get_item_display_name;

pub struct Tooltip {
    lines: Vec<String>,
}

impl Tooltip {
    fn new() -> Tooltip {
        Tooltip { lines: Vec::new() }
    }

    fn add_line<T: Into<String>>(&mut self, line: T) {
        self.lines.push(line.into());
    }

    /// Wrapping not supported, yet
    fn get_width(&self) -> usize {
        self.lines.iter().map(|s| s.len()).max().unwrap_or(0) + 2 // +2 for border
    }

    fn get_height(&self) -> usize {
        self.lines.len() + 2 // +2 for border
    }

    fn render(&self, draw_batch: &mut DrawBatch, x: i32, y: i32) {
        let box_gray: RGB = RGB::from_hex("#999999").expect("Oops");
        let light_gray: RGB = RGB::from_hex("#DDDDDD").expect("Oops");
        let white = RGB::named(rltk::WHITE);
        let black = RGB::named(rltk::BLACK);
        draw_batch.draw_box(
            Rect::with_size(
                x,
                y,
                self.get_width() as i32 - 1,
                self.get_height() as i32 - 1,
            ),
            ColorPair::new(white, box_gray),
        );
        for (i, s) in self.lines.iter().enumerate() {
            let col = if i == 0 { white } else { light_gray };
            draw_batch.print_color(
                Point::new(x + 1, y + i as i32 + 1),
                s,
                ColorPair::new(col, black),
            );
        }
    }
}

pub fn draw_tooltips(ecs: &World, ctx: &mut Rltk) {
    let mut draw_batch = DrawBatch::new();

    let (min_x, _max_x, min_y, _max_y) = camera::get_screen_bounds(ecs, ctx);
    let white = RGB::named(rltk::WHITE);
    let box_gray: RGB = RGB::from_hex("#999999").expect("Could not parse color from hex");

    let map = ecs.fetch::<Map>();
    let hidden = ecs.read_storage::<Hidden>();
    let attributes = ecs.read_storage::<Attributes>();
    let pools = ecs.read_storage::<Pools>();
    let statuses = ecs.read_storage::<StatusEffect>();
    let durations = ecs.read_storage::<Duration>();
    let names = ecs.read_storage::<Name>();

    let mouse_pos = ctx.mouse_pos();
    let mut mouse_map_pos = mouse_pos;
    // -1 compensate for map being offset from screen
    mouse_map_pos.0 += min_x;
    mouse_map_pos.1 += min_y;

    // Check if mouse is on map
    if mouse_map_pos.0 >= map.width - 1
        || mouse_map_pos.1 >= map.height - 1
        || mouse_map_pos.0 < 1
        || mouse_map_pos.1 < 1
    {
        return;
    }

    let mouse_idx = map.xy_idx(mouse_map_pos.0, mouse_map_pos.1);
    if !map.visible_tiles[mouse_idx] {
        return;
    }

    // Check if mouse-pos is on an entity, add its TT
    let mut tip_boxes: Vec<Tooltip> = Vec::new();
    spatial::for_each_tile_content(mouse_idx, |entity| {
        if hidden.get(entity).is_some() {
            return;
        }

        let mut tip = Tooltip::new();
        tip.add_line(get_item_display_name(ecs, entity));

        // Comment on attributes
        if let Some(attr) = attributes.get(entity) {
            let mut tip_text = String::new();

            match attr.might.bonus.cmp(&0) {
                Ordering::Less => tip_text += "Weak. ",
                Ordering::Equal => (),
                Ordering::Greater => tip_text += "Strong. ",
            }

            match attr.quickness.bonus.cmp(&0) {
                Ordering::Less => tip_text += "Clumsy. ",
                Ordering::Equal => (),
                Ordering::Greater => tip_text += "Agile. ",
            }

            match attr.fitness.bonus.cmp(&0) {
                Ordering::Less => tip_text += "Unhealthy. ",
                Ordering::Equal => (),
                Ordering::Greater => tip_text += "Healthy. ",
            }

            match attr.intelligence.bonus.cmp(&0) {
                Ordering::Less => tip_text += "Unintelligent. ",
                Ordering::Equal => (),
                Ordering::Greater => tip_text += "Smart. ",
            }

            if tip_text.is_empty() {
                tip_text = "Quite average".to_string();
            }

            tip.add_line(tip_text);
        }

        // Comment on pools
        let stat = pools.get(entity);
        if let Some(stat) = stat {
            tip.add_line(format!("Level: {lvl}", lvl = stat.level));
        }

        // Comment on durations (Status effects)
        for (status, duration, name) in (&statuses, &durations, &names).join() {
            if status.target == entity {
                tip.add_line(format!("{} ({})", name.name, duration.turns));
            }
        }

        tip_boxes.push(tip);
    });

    // No TT on mouse-pos
    if tip_boxes.is_empty() {
        return;
    }

    // Determine if TT renders to right or left of the target
    let arrow_y = mouse_pos.1;
    let (arrow, arrow_x) = if mouse_pos.0 < 40 {
        // Left
        (rltk::to_cp437('→'), mouse_pos.0 - 1)
    } else {
        // Right
        (rltk::to_cp437('←'), mouse_pos.0 + 1)
    };

    draw_batch.set(
        Point::new(arrow_x, arrow_y),
        ColorPair::new(white, box_gray),
        arrow,
    );

    let mut total_height = 0;
    for tt in tip_boxes.iter() {
        total_height += tt.get_height();
    }

    // Shunt all boxes upwards to center the stack
    let mut y = mouse_pos.1 - (total_height as i32 / 2);
    while y + (total_height as i32 / 2) > 50 {
        y -= 1;
    }

    // Draw the boxes
    for tt in tip_boxes.iter() {
        // -2 for border
        let x = if mouse_pos.0 < (PANE_WIDTH - 2) {
            mouse_pos.0 - (1 + tt.get_width() as i32)
        } else {
            mouse_pos.0 + (1 + tt.get_width() as i32)
        };
        tt.render(&mut draw_batch, x, y);
        y += tt.get_height() as i32;
    }

    let _ = draw_batch.submit(7000);
}
