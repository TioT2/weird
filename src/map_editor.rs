use std::{collections::HashMap, ops::RangeFrom};

use crate::{font::Font, input, surface::Surface, Vec2f, Vec2si};

struct Polygon {
    pub points: Vec<u32>,
    pub floor: f32,
    pub ceiling: f32,
}

pub struct Edge {
    base_position: Vec2f,
    normal: Vec2f,
}

enum EditorState {
    General,
    BuildPolygon {
        path: Vec<u32>,
    },
    DragPoint {
        id: u32,
    }
}

pub struct MapEditor {
    pub scale: f32,

    state: EditorState,
    points: HashMap<u32, Vec2f>,
    polygons: HashMap<u32, Polygon>,
    id_generator: RangeFrom<u32>,
    font: Font,
}

impl MapEditor {
    pub fn new() -> Self {
        Self {
            scale: 1.0,
            state: EditorState::General,
            points: HashMap::new(),
            polygons: HashMap::new(),
            id_generator: 0..,
            font: Font::default(),
        }
    }

    fn insert_point(&mut self, location: Vec2f) -> u32 {
        let id = self.id_generator.next().unwrap();
        self.points.insert(id, location);
        id
    }

    fn erase_point(&mut self, id: u32) {
        _ = self.points.remove(&id)
    }

    pub fn response(&mut self, _input_state: &input::State) {

    } // fn response

    pub fn render(&self, surface: &Surface) {
        surface.draw_bar(0, 0, surface.width as isize, surface.height as isize, 0x000000);

        for (_, polygon) in &self.polygons {
            let ps: Option<Vec<Vec2si>> = polygon.points
                .iter()
                .map(|id| self.points
                    .get(id)
                    .map(|p| Vec2si {
                        x: ((0.5 + p.x / self.scale) * surface.width as f32) as isize,
                        y: ((0.5 - p.y / self.scale) * surface.height as f32) as isize,
                    })
                )
                .collect()
            ;

            if let Some(points) = ps {
                let mut prev_point = points[0];

                for point in &points[1..] {
                    surface.draw_line(prev_point.x, prev_point.y, point.x, point.y, 0x00FF00);
                    prev_point = *point;
                }
                surface.draw_line(prev_point.x, prev_point.y, points[0].x, points[0].y, 0x00FF00);
            }
        }

        for point in &self.points {

        }
    } // fn render
} // impl MapEditor

// file map_editor.rs
