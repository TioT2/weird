pub mod util;
pub mod timer;
pub mod input;
pub mod math;
pub mod camera;


use math::*;
use camera::Camera;

use std::collections::HashSet;
use input::KeyCode;
use util::unordered_pair::UnorderedPair;

/// Sector edge representation structure
#[derive(Copy, Clone, Debug, PartialEq)]
enum EdgeType {
    /// Wall
    Wall,
    /// Portal to some sector
    Portal(SectorId),
} // enum Edge

#[derive(Copy, Clone, Debug)]
struct Edge {
    pub p0: Vec2f,
    pub p1: Vec2f,
    pub d: Vec2f,
    pub d_cross_p0: f32,
}

impl std::fmt::Display for EdgeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Wall => f.write_str("Wall"),
            Self::Portal(portal) => f.write_fmt(format_args!("Portal({})", portal.as_u32()))
        }
    } // fn fmt
} // impl std::fmt::Display for Edge

/// Sector representation structure
struct Sector {
    pub points: Vec<Vec2f>,
    pub edge_types: Vec<EdgeType>,
    pub edges: Vec<Edge>,
    pub floor: f32,
    pub ceiling: f32,
} // struct Sector

impl Sector {
    /// Sector with loop of walls representation structure
    pub fn wall_loop(points: &[Vec2f]) -> Self {
        let mut sector = Self {
            points: points.into(),
            edge_types: {
                let mut edges = Vec::<EdgeType>::with_capacity(points.len());
                edges.resize(points.len(), EdgeType::Wall);
                edges
            },
            edges: Vec::new(),
            floor: 0.0,
            ceiling: 1.0,
        };
        sector.edges = sector.build_edges();
        sector
    } // fn wall_loop

    pub fn build_edges(&self) -> Vec<Edge> {
        let mut edge_lines = Vec::<Edge>::with_capacity(self.points.len());

        for (left, right) in self.points.iter().zip({
            let pit = self.points.iter();
            let mut pit = pit.cycle();
            pit.next();
            pit
        }) {
            let d = Vec2f {
                x: right.x - left.x,
                y: right.y - left.y,
            };

            edge_lines.push(Edge {
                p0: *left,
                p1: *right,
                d_cross_p0: d.x * left.y - d.y * left.x,
                d,
            });
        }

        edge_lines
    }

    /// Check for point being located in sector
    pub fn contains(&self, point: Vec2f) -> bool {
        let mut sign_collector: u8 = 0;

        for line in &self.edges {
            let coef = line.d.x * point.y - line.d.y * point.x - line.d_cross_p0;

            unsafe {
                sign_collector |= 1 << (std::mem::transmute::<f32, u32>(coef) >> 31);
            }
        }

        unsafe {
            std::mem::transmute::<u8, bool>(sign_collector ^ (sign_collector >> 1))
        }
    } // pub fn is_in
}

impl std::fmt::Display for Sector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "\
            points: {:?}\n\
            edges: {:?}\n\
            bounds: [{}; {}]\n\
            ",
            self.points, self.edge_types, self.floor, self.ceiling,
        ))
    }
}

/// Projection information representation structure
pub struct ProjectionInfo {
    pub projection_width: f32,
    pub projection_height: f32,
}

/// Map representation structure
struct Map {
    sectors: Vec<Sector>,
    pub camera_location: Vec2f,
    pub camera_rotation: f32,
} // struct Map

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
struct SectorId(u32);

impl SectorId {
    pub fn new(index: u32) -> Self {
        Self(index)
    }

    pub fn as_u32(self) -> u32 {
        self.0
    }
}

impl Map {
    pub fn find_sector(&self, location: Vec2f) -> Option<SectorId> {
        self.sectors
            .iter()
            .enumerate()
            .find(|(_, sector)| sector.contains(location))
            .map(|(index, _)| SectorId::new(index as u32))
    }

    pub fn get_sector(&self, id: SectorId) -> Option<&Sector> {
        self.sectors.get(id.as_u32() as usize)
    }
}

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
                            Ok(EdgeType::Wall)
                        } else {
                            let adjoint = sectors.iter()
                                .enumerate()
                                .find(|(index, sector)| *index != sector_index && sector.walls.contains(&pair))
                                .map(|(index, _)| index as u32)
                                .ok_or(WmtLoadingError::NoAdjointSectorForPortal {
                                    from_sector: sector_index as u32,
                                    by_indices: pair.into()
                                })?;

                            Ok(EdgeType::Portal(SectorId::new(adjoint)))
                        }
                    })
                    .collect::<Result<Vec<EdgeType>, WmtLoadingError>>()?;

                // Validate sector bounds
                if sector.floor > sector.ceiling {
                    return Err(WmtLoadingError::InvalidSectorBounds { floor: sector.floor, ceiling: sector.ceiling })
                }

                let mut sector = Sector {
                    points,
                    edge_types: edges,
                    edges: Vec::new(),
                    floor: sector.floor,
                    ceiling: sector.ceiling,
                };
                sector.edges = sector.build_edges();

                Ok(sector)
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

#[derive(Copy, Clone)]
struct Surface {
    data: *mut u32,
    width: usize,
    height: usize,
}

/// Renderer representation structure
struct Render {
} // struct Render

struct SectorRenderContext<'a> {
    surface: &'a Surface,
    map: &'a Map,
    camera: &'a Camera,
    visit_stack: std::collections::VecDeque<SectorId>,
    floor_buffer: Vec<usize>,
    ceil_buffer: Vec<usize>,
    inv_depth_buffer: Vec<f32>,
}

impl Render {
    /// Render create function
    pub fn new() -> Render {
        Render {
        }
    }

    pub unsafe fn draw_bar_unchecked(&self, surface: &Surface, x0: usize, y0: usize, x1: usize, y1: usize, color: u32) {
        let mut yptr = surface.data.add(y0 * surface.width + x0);
        let yeptr = yptr.add((y1 - y0) * surface.width);
        let dx = x1 - x0;

        while yptr != yeptr {
            let mut xptr = yptr;
            let xeptr = xptr.add(dx);

            while xptr != xeptr {
                *xptr = color;
                xptr = xptr.add(1);
            }

            yptr = yptr.add(surface.width);
        }
    }

    fn clip_line(mut x0: isize, mut y0: isize, mut x1: isize, mut y1: isize, w: usize, h: usize) -> Option<(usize, usize, usize, usize)> {
        // Clip line
        const LOC_INSIDE: u32 = 0;
        const LOC_LEFT: u32 = 1;
        const LOC_RIGHT: u32 = 2;
        const LOC_BOTTOM: u32 = 4;
        const LOC_TOP: u32 = 8;

        let w = w as isize - 1;
        let h = h as isize - 1;

        let get_point_code = |x: isize, y: isize| {
            let mut code = LOC_INSIDE;

            if x < 0 { code |= LOC_LEFT; }
            if x > w { code |= LOC_RIGHT; }
            if y < 0 { code |= LOC_TOP; }
            if y > h { code |= LOC_BOTTOM; }

            code
        };

        let mut code_0 = get_point_code(x0, y0);
        let mut code_1 = get_point_code(x1, y1);

        if 'intersection_loop: loop {
            if (code_0 | code_1) == LOC_INSIDE {
                break 'intersection_loop true;
            }

            if (code_0 & code_1) != LOC_INSIDE {
                break 'intersection_loop false;
            }

            let out_code = if code_0 != LOC_INSIDE {
                code_0
            } else {
                code_1
            };

            let dx = x1 - x0;
            let dy = y1 - y0;

            let px;
            let py;

            if out_code & LOC_TOP != 0 {
                px = x0 - dx * y0 / dy;
                py = 0;
            } else if out_code & LOC_BOTTOM != 0 {
                px = x0 + dx * (h - y0) / dy;
                py = h;
            } else if out_code & LOC_LEFT != 0 {
                px = 0;
                py = y0 - dy * x0 / dx;
            } else if out_code & LOC_RIGHT != 0 {
                px = w;
                py = y0 + dy * (w - x0) / dx;
            } else {
                break 'intersection_loop false;
            }

            if out_code == code_0 {
                x0 = px;
                y0 = py;
                code_0 = get_point_code(x0, y0);
            } else {
                x1 = px;
                y1 = py;
                code_1 = get_point_code(x1, y1);
            }
        } {
            // Clipped values are safe to transmute into unsigned
            unsafe {
                Some((
                    std::mem::transmute::<isize, usize>(x0),
                    std::mem::transmute::<isize, usize>(y0),
                    std::mem::transmute::<isize, usize>(x1),
                    std::mem::transmute::<isize, usize>(y1),
                ))
            }
        } else {
            None
        }
    }

    pub fn draw_line(&self, surface: &Surface, x0: isize, y0: isize, x1: isize, y1: isize, color: u32) {
        if let Some((x0, y0, x1, y1)) = Self::clip_line(x0, y0, x1, y1, surface.width, surface.height) {
            unsafe {
                self.draw_line_unchecked(surface, x0, y0, x1, y1, color);
            }
        }
    }

    pub unsafe fn draw_line_unchecked(&self, surface: &Surface, x0: usize, y0: usize, x1: usize, y1: usize, color: u32) {
        let surface = *surface;

        let (mut dy, sy): (usize, usize) = if y1 < y0 {
            (y0 - y1, surface.width.wrapping_neg())
        } else {
            (y1 - y0, surface.width)
        };
        let (mut dx, sx): (usize, usize) = if x1 < x0 {
            (x0 - x1, 1usize.wrapping_neg())
        } else {
            (x1 - x0, 1usize)
        };

        let mut pptr = surface.data.wrapping_add(y0 * surface.width + x0);
        pptr.write(color);

        if dx >= dy {
            let ie = 2 * dy;
            let mut f = ie.wrapping_sub(dx);
            let ine = ie.wrapping_sub(2 * dx);

            while dx != 0 {
                pptr = pptr.wrapping_add(sx);
                pptr.write(color);
                dx -= 1;
                if f < std::mem::transmute(isize::MIN) {
                    pptr = pptr.wrapping_add(sy);
                    f = f.wrapping_add(ine);
                } else {
                    f = f.wrapping_add(ie);
                }
            }
        } else {
            let ie = 2 * dx;
            let mut f = ie.wrapping_sub(dy);
            let ine = ie.wrapping_sub(2 * dy);

            while dy != 0 {
                pptr = pptr.wrapping_add(sy);
                pptr.write(color);
                dy -= 1;

                if f < std::mem::transmute(isize::MIN) {
                    pptr = pptr.wrapping_add(sx);
                    f = f.wrapping_add(ine);
                } else {
                    f = f.wrapping_add(ie);
                }
            }
        }
    }

    fn render_sector(context: &mut SectorRenderContext, sector_id: SectorId, screen_x_begin: usize, screen_x_end: usize) {
        let sector = match context.map.get_sector(sector_id) {
            Some(sector) => sector,
            None => return,
        };

        'edge_loop: for (edge, edge_type) in sector.edges.iter().zip(sector.edge_types.iter()) {
            let mut p0 = context.camera.to_space(edge.p0);
            let mut p1 = context.camera.to_space(edge.p1);

            if p0.x > p1.x {
                let tmp = p1;
                p1 = p0;
                p0 = tmp;
            }

            // Check for x or y visibility and clamp'em if not
            if p0.y <= 0.0 {
                if p1.y <= 0.0 {
                    continue 'edge_loop;
                } else {
                    p0 = Vec2f {
                        x: p0.x - p0.y * (p1.x - p0.x) / (p1.y - p0.y),
                        y: 0.01,
                    };
                }
            } else if p1.y <= 0.0 {
                p1 = Vec2f {
                    x: p0.x - p0.y * (p1.x - p0.x) / (p1.y - p0.y),
                    y: 0.01
                };
            }


            let to_screen_x = |p: Vec2f| -> isize {
                (((p.x / p.y) + 1.0) / 2.0 * context.surface.width as f32) as isize
            };


            let (xp0, xp1) = unsafe {
                let x0 = std::mem::transmute::<isize, usize>(to_screen_x(p0).clamp(screen_x_begin as isize, screen_x_end as isize));
                let x1 = std::mem::transmute::<isize, usize>(to_screen_x(p1).clamp(screen_x_begin as isize, screen_x_end as isize));

                if x0 > x1 {
                    (x1, x0)
                } else {
                    (x0, x1)
                }
            };

            let (color, floor_color) = match context.visit_stack.len() {
                0 => (0x00FF00, 0x008800),
                1 => (0xFF0000, 0x880000),
                2 => (0x0000FF, 0x000088),
                _ => (0x333333, 0x333333),
            };

            // Edge normal and distance form user to edge
            let (edge_norm, inv_edge_distance) = {
                let edge_norm_unorm = Vec2f {
                    x: p1.y - p0.y,
                    y: p0.x - p1.x,
                };

                let edge_line_inv_norm = 1.0 / (edge_norm_unorm.x * edge_norm_unorm.x + edge_norm_unorm.y * edge_norm_unorm.y).sqrt();
                let edge_norm = Vec2f {
                    x: edge_norm_unorm.x * edge_line_inv_norm,
                    y: edge_norm_unorm.y * edge_line_inv_norm,
                };

                (edge_norm, 1.0 / (edge_norm.x * p0.x + edge_norm.y * p0.y).abs())
            };

            let is_portal = *edge_type != EdgeType::Wall;

            for x in xp0..xp1 {
                let pixel_dir = Vec2f {
                    x: x as f32 / context.surface.width as f32 * 2.0 - 1.0,
                    y: 1.0,
                };

                // edge_distance
                let inv_distance = (pixel_dir.x * edge_norm.x + pixel_dir.y * edge_norm.y).abs() * inv_edge_distance;

                let y = ((inv_distance * context.surface.height as f32) as isize).clamp(0, (context.surface.height / 2) as isize) as usize;

                unsafe {
                    let buf_floor = context.floor_buffer.get_unchecked_mut(x);
                    let buf_ceil = context.ceil_buffer.get_unchecked_mut(x);

                    let ceil_y = context.surface.height / 2 - y;
                    let floor_y = context.surface.height / 2 + y;

                    let pbegin = context.surface.data.add(x);
                    let mut pptr = pbegin.add(context.surface.width * *buf_ceil);
                    let pceil = pbegin.add(context.surface.width * ceil_y);
                    let pfloor = pbegin.add(context.surface.width * floor_y);
                    let pend = pbegin.add(context.surface.width * *buf_floor);

                    *buf_floor = floor_y;
                    *buf_ceil = ceil_y;
                    *context.inv_depth_buffer.get_unchecked_mut(x) = inv_distance;

                    while pptr < pceil {
                        *pptr = 0xDDDDDD;
                        pptr = pptr.add(context.surface.width);
                    }

                    if is_portal {
                        pptr = pfloor;
                    } else {
                        while pptr < pfloor {
                            *pptr = color;
                            pptr = pptr.add(context.surface.width);
                        }
                    }

                    while pptr < pend {
                        *pptr = floor_color;
                        pptr = pptr.add(context.surface.width);
                    }
                }
            }

            if let EdgeType::Portal(portal_sector_id) = edge_type {
                context.visit_stack.push_back(sector_id);

                if !context.visit_stack.contains(portal_sector_id) {
                    Self::render_sector(context, *portal_sector_id, xp0, xp1);
                }

                context.visit_stack.pop_back();
            };
        } // 'edge_loop
    }

    /// Next frame rendering function
    /// `surface` - surface to render frame to
    /// `map` - map to render
    pub fn render(&mut self, surface: &Surface, map: &Map, camera: &Camera) {
        if let Some(sector_id) = map.find_sector(camera.location) {
            let mut context = SectorRenderContext {
                surface,
                map,
                camera,
                visit_stack: std::collections::VecDeque::new(),
                floor_buffer: {
                    let mut buffer = Vec::with_capacity(surface.width);
                    buffer.resize(surface.width, surface.height);
                    buffer
                },
                ceil_buffer: {
                    let mut buffer = Vec::with_capacity(surface.width);
                    buffer.resize(surface.width, 0);
                    buffer
                },
                inv_depth_buffer: {
                    let mut buffer = Vec::with_capacity(surface.width);
                    buffer.resize(surface.width, 0.0);
                    buffer
                },
            };

            Self::render_sector(&mut context, sector_id, 0, surface.width);
        }
    } // fn next_frame

    /// Next frame rendering function
    /// `surface` - surface to render frame to
    /// `map` - map to render
    pub fn render_minimap(&mut self, surface: &Surface, map: &Map, camera: &Camera) {
        let draw_sector = |sector: &Sector| {
            for (edge, edge_type) in sector.edges.iter().zip(sector.edge_types.iter()) {
                // Calculate edge projection
                let p0 = camera.to_space(edge.p0);
                let p1 = camera.to_space(edge.p1);

                // Project edge to pixel space and render, actually
                let edge_color = match edge_type {
                    EdgeType::Wall => 0x00FF00,
                    EdgeType::Portal(_) => 0xFF0000,
                };

                /*unsafe*/ {
                    self.draw_line(surface,
                        (p0.x * 30.0) as isize + surface.width  as isize / 2,
                        (-p0.y * 30.0) as isize + surface.height as isize / 2,
                        (p1.x * 30.0) as isize + surface.width  as isize / 2,
                        (-p1.y * 30.0) as isize + surface.height as isize / 2,
                        edge_color,
                    );
                }
            }
        };

        for sector in &map.sectors {
            draw_sector(sector);
        }

        // Render player
        let (x0, y0) = (surface.width as isize / 2, surface.height as isize / 2);
        let cfov2 = 1000isize;//((camera.fov / 2.0).cos() * 1000.0) as isize;
        let sfov2 = 1000isize;//((camera.fov / 2.0).sin() * 1000.0) as isize;

        self.draw_line(surface, x0, y0, x0 + sfov2, y0 - cfov2, 0xFF0000);
        self.draw_line(surface, x0, y0, x0 - sfov2, y0 - cfov2, 0xFF0000);
    } // impl fn render_minimap
} // impl Render

/// Main program function
fn main() {
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    let screen_size = winit::dpi::LogicalSize::<u32>::new(800, 600);
    let window = winit::window::WindowBuilder::new()
        .with_title("WEIRD")
        .with_resizable(true)
        .with_inner_size(screen_size)
        .build(&event_loop).unwrap()
        ;

    let window_context = softbuffer::Context::new(&window).unwrap();

    let mut surface_size = screen_size.clone();
    let mut surface = softbuffer::Surface::new(&window_context, &window).unwrap();
    _ = surface.resize(surface_size.width.try_into().unwrap(), surface_size.height.try_into().unwrap());

    let map_source = include_str!("../maps/test.wmt");
    let map = Map::load_from_wmt(map_source).unwrap();
    let mut camera = Camera::new();

    camera.set_location(map.camera_location, map.camera_rotation, 0.5);

    let mut render = Render::new();

    let mut timer = timer::Timer::new();
    let mut frame_index = 0;
    let mut input = input::Input::new();

    event_loop.run(|event, target| {
        match event {
            winit::event::Event::WindowEvent { window_id, event } => if window.id() == window_id {
                match event {
                    winit::event::WindowEvent::CloseRequested => {
                        target.exit();
                    }
                    winit::event::WindowEvent::KeyboardInput { event, .. } => if let winit::keyboard::PhysicalKey::Code(code) = event.physical_key {
                        input.on_key_state_change(code, event.state == winit::event::ElementState::Pressed);
                    }
                    winit::event::WindowEvent::Resized(size) => {
                        surface_size = size.to_logical(window.scale_factor());
                        if let Some((width, height)) = surface_size.width.try_into().ok().zip(surface_size.height.try_into().ok()) {
                            _ = surface.resize(width, height);
                        }
                    }
                    winit::event::WindowEvent::RedrawRequested => 'redraw: {
                        timer.response();
                        if timer.get_fps() > 1.0 {
                            if frame_index % (timer.get_fps().ceil() as u32) == 1 {
                                println!("FPS: {}", timer.get_fps());
                            }
                        } else {
                            println!("Less, than 1 FPS");
                        }
                        frame_index += 1;

                        let mut mut_buffer = match surface.buffer_mut() {
                            Ok(buffer) => buffer,
                            Err(_) => break 'redraw,
                        };

                        'input_control: {
                            let input = input.get_state();
                            let dt = timer.get_delta_time().max(1.0 / 500.0);

                            let ox = (input.is_key_pressed(KeyCode::KeyA) as i32 - input.is_key_pressed(KeyCode::KeyD) as i32) as f32;
                            let oy = (input.is_key_pressed(KeyCode::KeyW) as i32 - input.is_key_pressed(KeyCode::KeyS) as i32) as f32;

                            if ox == 0.0 && oy == 0.0 {
                                break 'input_control;
                            }

                            let new_location = Vec2f {
                                x: camera.location.x + camera.rotation.cos() * oy * dt * 3.0,
                                y: camera.location.y + camera.rotation.sin() * oy * dt * 3.0,
                            };
                            if map.find_sector(new_location).is_some() {
                                camera.set_location(new_location, 0.5, camera.rotation + ox * dt * 2.0);
                            }
                        }

                        render.render(&Surface {
                            data: mut_buffer.as_mut_ptr(),
                            width: surface_size.width as usize,
                            height: surface_size.height as usize,
                        }, &map, &camera);

                        _ = mut_buffer.present();

                        input.clear_changed();

                        window.request_redraw();
                    }
                    _ => {},
                }
            }
            _ => {},
        }
    }).unwrap();
}
