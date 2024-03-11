pub mod util;

use std::collections::HashSet;
use util::unordered_pair::UnorderedPair;

/// 2-component vector representation structure
#[derive(Copy, Clone, Debug)]
struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl<T: std::fmt::Display> std::fmt::Display for Vec2<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("<{}, {}>", self.x, self.y))
    }
}

type Vec2f = Vec2<f32>;

/// Sector edge representation structure
#[derive(Copy, Clone, Debug)]
enum Edge {
    /// Wall
    Wall,
    /// Portal to some sector
    Portal(u32),
} // enum Edge

impl std::fmt::Display for Edge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Wall => f.write_str("Wall"),
            Self::Portal(portal) => f.write_fmt(format_args!("Portal({portal})"))
        }
    } // fn fmt
} // impl std::fmt::Display for Edge

/// Sector representation structure
struct Sector {
    pub points: Vec<Vec2f>,
    pub edges: Vec<Edge>,
    pub floor: f32,
    pub ceiling: f32,
} // struct Sector

impl std::fmt::Display for Sector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "\
            points: {:?}\n\
            edges: {:?}\n\
            bounds: [{}; {}]\n\
            ",
            self.points, self.edges, self.floor, self.ceiling,
        ))
    }
}

/// Map representation structure
struct Map {
    pub sectors: Vec<Sector>,
    pub camera_location: Vec2f,
    pub camera_rotation: f32,
} // struct Map

/// Map loading error representation structure
#[derive(Debug)]
enum WmtLoadingError {
    NumberParsingError,
    UnknownLineType(String),
    InvalidPointIndex(u32),
    NotEnoughPointCoordinates,
    NotEnoughCameraParameters,
    NotEnoughSectorVertices,
    NoAdjointSectorForPortal {
        from_sector: u32,
        by_indices: (u32, u32),
    },
    InvalidSectorBounds {
        floor: f32,
        ceiling: f32
    },
    Other(String),
} // enum WmtLoadingError


impl Map {
    /// Map from .wmt file loading function
    /// * `source` - file text
    /// * Returns valid Map or WmtLoadingError
    pub fn load_from_wmt(source: &str) -> Result<Map, WmtLoadingError> {
        struct SectorData {
            pub point_indices: Vec<u32>,
            pub floor: f32,
            pub ceiling: f32,
            pub walls: HashSet<UnorderedPair<u32>>,
        }
        let mut points = Vec::<Vec2f>::new();
        let mut walls = HashSet::<UnorderedPair<u32>>::new();
        let mut sectors = Vec::<SectorData>::new();
        let mut camera_location = Vec2f {
            x: 0.0,
            y: 0.0,
        };
        let mut camera_rotation = 0.0;

        // Parse file data
        for line in source.lines().map(|line| line.trim()) {
            let mut elem = line.split(' ');

            let line_type = match elem.next() {
                Some(s) => s,
                None => continue,
            };

            match line_type {
                // Comment | Empty line
                "#" | "" => {}

                // Point
                "p" | "point" => {
                    let (sx, sy) = elem.next().zip(elem.next()).ok_or(WmtLoadingError::NotEnoughPointCoordinates)?;
                    let (x, y) = sx.parse::<f32>().ok().zip(sy.parse::<f32>().ok()).ok_or(WmtLoadingError::NumberParsingError)?;

                    points.push(Vec2f { x, y });
                }

                // Wall
                "w" | "wall" => {
                    let (sx, sy) = elem.next()
                        .zip(elem.next())
                        .ok_or(WmtLoadingError::NotEnoughPointCoordinates)?;

                    walls.insert(sx.parse::<u32>().ok()
                        .zip(sy.parse::<u32>().ok())
                        .ok_or(WmtLoadingError::NumberParsingError)?
                        .into()
                    );
                }

                // Sector
                "s" | "sector" => {
                    let (sx, sy) = elem.next().zip(elem.next()).ok_or(WmtLoadingError::NotEnoughPointCoordinates)?;
                    let (floor, ceiling) = sx.parse::<f32>().ok().zip(sy.parse::<f32>().ok()).ok_or(WmtLoadingError::NumberParsingError)?;

                    let point_indices: Vec<u32> = elem
                        .map(|index_str| index_str.parse::<u32>().ok())
                        .collect::<Option<Vec<u32>>>().ok_or(WmtLoadingError::NumberParsingError)?;

                    if point_indices.len() < 3 {
                        return Err(WmtLoadingError::NotEnoughSectorVertices);
                    }

                    let walls = point_indices.iter()
                        .zip({
                            let mut iter = point_indices.iter().cycle();
                            iter.next();
                            iter
                        })
                        .map(|(first, second)| {
                            UnorderedPair::new(*first, *second)
                        })
                        .collect::<HashSet<UnorderedPair<u32>>>();

                    sectors.push(SectorData { point_indices, floor, ceiling, walls });
                }

                "c" | "camera" => {
                    let ((scx, scy), sca) = elem.next().zip(elem.next()).zip(elem.next()).ok_or(WmtLoadingError::NotEnoughCameraParameters)?;
                    ((camera_location.x, camera_location.y), camera_rotation) = scx.parse::<f32>().ok()
                        .zip(scy.parse::<f32>().ok())
                        .zip(sca.parse::<f32>().ok())
                        .ok_or(WmtLoadingError::NumberParsingError)?;
                }

                _ => return Err(WmtLoadingError::UnknownLineType(line_type.to_string()))
            }
        }

        // Parse data parsed from file
        let sectors = sectors.iter()
            .enumerate()
            .map(|(sector_index, sector)| -> Result<Sector, WmtLoadingError> {
                let points = sector.point_indices.iter()
                    .map(|index| points
                        .get(*index as usize)
                        .map(|v| *v)
                        .ok_or(WmtLoadingError::InvalidPointIndex(*index))
                    )
                    .collect::<Result<Vec<Vec2f>, WmtLoadingError>>()?;

                let edges = sector.point_indices.iter()
                    .zip({
                        let mut iter = sector.point_indices.iter().cycle();
                        iter.next();
                        iter
                    })
                    .map(|(first, second)| {
                        let pair = UnorderedPair::new(*first, *second);

                        // Find adjoint sector
                        if walls.contains(&pair) {
                            Ok(Edge::Wall)
                        } else {
                            let adjoint = sectors.iter()
                                .enumerate()
                                .find(|(index, sector)| *index != sector_index && sector.walls.contains(&pair))
                                .map(|(index, _)| index as u32)
                                .ok_or(WmtLoadingError::NoAdjointSectorForPortal {
                                    from_sector: sector_index as u32,
                                    by_indices: pair.into()
                                })?;

                            Ok(Edge::Portal(adjoint))
                        }
                    })
                    .collect::<Result<Vec<Edge>, WmtLoadingError>>()?;

                // Validate sector bounds
                if sector.floor > sector.ceiling {
                    return Err(WmtLoadingError::InvalidSectorBounds { floor: sector.floor, ceiling: sector.ceiling })
                }

                Ok(Sector {
                    points,
                    edges,
                    floor: sector.floor,
                    ceiling: sector.ceiling,
                })
            })
            .collect::<Result<Vec<Sector>, WmtLoadingError>>()?;

        Ok(Map { sectors, camera_location, camera_rotation })
    } // fn load_from_wmt
} // impl Map

struct Frustum {
    pub x0: u32,
    pub y0_min: u32,
    pub y0_max: u32,

    pub x1: u32,
    pub y1_min: u32,
    pub y1_max: u32,
}

struct Surface {
    data: *mut u32,
    width: usize,
    height: usize,
}

struct Render {
    lower_bound_buffer: Vec<usize>,
    upper_bound_bufer: Vec<usize>,
}

impl Render {
    pub fn new() -> Render {
        Render {
            lower_bound_buffer: Vec::new(),
            upper_bound_bufer: Vec::new(),
        }
    }


    pub fn next_frame(surface: &Surface, map: &Map) {

    }
}

fn main() {
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    let screen_size = winit::dpi::LogicalSize::new(800, 600);
    let window = winit::window::WindowBuilder::new()
        .with_title("WEIRD")
        .with_resizable(false)
        .with_inner_size(screen_size)
        .build(&event_loop).unwrap()
        ;

    let map_source = include_str!("../maps/test.wmt");
    let map = Map::load_from_wmt(map_source).unwrap();

    let render = Render::new();

    event_loop.run(|event, target| {
        match event {
            winit::event::Event::WindowEvent { window_id, event } => if window.id() == window_id {
                match event {
                    winit::event::WindowEvent::CloseRequested => {
                        target.exit();
                    }
                    winit::event::WindowEvent::RedrawRequested => {
                        window.request_redraw();
                    }
                    _ => {},
                }
            }
            _ => {},
        }
    }).unwrap();
}
