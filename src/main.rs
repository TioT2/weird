pub mod util;
pub mod timer;
pub mod input;

use std::{collections::HashSet, ops::Range};
use input::KeyCode;
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
} // impl std::fmt::Display for Vec2

type Vec2f = Vec2<f32>;

/// Sector edge representation structure
#[derive(Copy, Clone, Debug)]
enum EdgeType {
    /// Wall
    Wall,
    /// Portal to some sector
    Portal(u32),
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
            Self::Portal(portal) => f.write_fmt(format_args!("Portal({portal})"))
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

                            Ok(EdgeType::Portal(adjoint))
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
    lower_bound_buffer: Vec<usize>,
    upper_bound_bufer: Vec<usize>,
} // struct Render

struct Camera {
    pub location: Vec2f,
    pub rotation: f32,
    pub fov: f32,
}

impl Render {
    /// Render create function
    pub fn new() -> Render {
        Render {
            lower_bound_buffer: Vec::new(),
            upper_bound_bufer: Vec::new(),
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

    /// Next frame rendering function
    /// `surface` - surface to render frame to
    /// `map` - map to render
    pub fn next_frame(&mut self, surface: &Surface, map: &Map, camera: &Camera) {
        let direction = Vec2f {
            x: camera.rotation.cos(),
            y: camera.rotation.sin(),
        };
        let right = Vec2f {
            x: direction.y,
            y: -direction.x,
        };
        let location = camera.location;
        let location_dot_direction = location.x * direction.x + location.y * direction.y;
        let location_dot_right = location.x * right.x + location.y * right.y;

        let to_camera_space = |p: Vec2f| -> Vec2f {
            Vec2f {
                x: p.x * right.x     + p.y * right.y     - location_dot_right,
                y: p.x * direction.x + p.y * direction.y - location_dot_direction,
            }
        };

        struct RenderContext<'a> {
            direction: Vec2f,
            location_dot_direction: f32,
            location_dot_right: f32,
            right: Vec2f,

            surface: &'a Surface,
            map: &'a Map,
            camera: &'a Camera,
            visit_stack: std::collections::VecDeque<u32>,
        }

        impl<'a> RenderContext<'a> {
            pub fn to_camera_space(&self, p: Vec2f) -> Vec2f {
                Vec2f {
                    x: p.x * self.right.x     + p.y * self.right.y     - self.location_dot_right,
                    y: p.x * self.direction.x + p.y * self.direction.y - self.location_dot_direction,
                }
            }
        }

        fn render_sector(context: &mut RenderContext, sector_id: u32, screen_x_begin: usize, screen_x_end: usize) {
            let sector = match context.map.sectors.get(sector_id as usize) {
                Some(sector) => sector,
                None => return,
            };

            'edge_loop: for (edge, edge_type) in sector.edges.iter().zip(sector.edge_types.iter()) {
                let mut p0 = context.to_camera_space(edge.p0);
                let mut p1 = context.to_camera_space(edge.p1);

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
                    let x0 = std::mem::transmute::<isize, usize>(to_screen_x(p0).clamp(0, context.surface.width as isize));
                    let x1 = std::mem::transmute::<isize, usize>(to_screen_x(p1).clamp(0, context.surface.width as isize));

                    if x0 > x1 {
                        (x1, x0)
                    } else {
                        (x0, x1)
                    }
                };

                let color = match edge_type {
                    EdgeType::Portal(portal_id) => {
                        context.visit_stack.push_front(sector_id);
                        if !context.visit_stack.contains(portal_id) {
                            render_sector(context, *portal_id, xp0, xp1);
                        }
                        continue;
                    },
                    EdgeType::Wall => 0x002200,
                };


                let (edge_norm, edge_neg_base_dot_norm) = {
                    let edge_norm_unorm = Vec2f {
                        x: p1.y - p0.y,
                        y: p0.x - p1.x,
                    };

                    let edge_line_inv_norm = 1.0 / (edge_norm_unorm.x * edge_norm_unorm.x + edge_norm_unorm.y * edge_norm_unorm.y).sqrt();
                    let edge_norm = Vec2f {
                        x: edge_norm_unorm.x * edge_line_inv_norm,
                        y: edge_norm_unorm.y * edge_line_inv_norm,
                    };

                    (edge_norm, -(edge_norm.x * p0.x + edge_norm.y * p0.y))
                };
                let edge_distance = edge_neg_base_dot_norm.abs();

                for x in xp0..xp1 {
                    let pixel_dir = Vec2f {
                        x: x as f32 / context.surface.width as f32 * 2.0 - 1.0,
                        y: 1.0,
                    };

                    // edge_distance
                    let distance = edge_distance / (pixel_dir.x * edge_norm.x + pixel_dir.y * edge_norm.y).abs();

                    let mut y = ((0.33 / distance * context.surface.height as f32) as isize).clamp(0, (context.surface.height / 2) as isize) as usize;

                    unsafe {
                        let mut pup = context.surface.data.add(context.surface.width * context.surface.height / 2 + x);
                        let mut pdown = context.surface.data.add(context.surface.width * context.surface.height / 2 + x);

                        while y != 0 {
                            *pup = color;
                            *pdown = color;

                            pup = pup.add(context.surface.width);
                            pdown = pdown.sub(context.surface.width);
                            y -= 1;
                        }
                    }
                }
            } // 'edge_loop
        }

        'rendering: {
            let main_sector = match map.sectors.iter().enumerate().find(|(_, sector)| sector.contains(location)) {
                Some(sector) => sector.0,
                None => break 'rendering,
            };
            let mut context = RenderContext {
                direction,
                location_dot_direction,
                location_dot_right,
                right,

                surface,
                map,
                camera,
                visit_stack: std::collections::VecDeque::new(),
            };

            render_sector(&mut context, main_sector as u32, 0, surface.width);
        }

        let draw_sector = |sector: &Sector| {
            for (edge, edge_type) in sector.edges.iter().zip(sector.edge_types.iter()) {
                // Calculate edge projection
                let p0 = to_camera_space(edge.p0);
                let p1 = to_camera_space(edge.p1);

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
    } // fn next_frame
} // impl Render

/// Main program function
fn main() {
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    let screen_size = winit::dpi::LogicalSize::<u32>::new(800, 600);
    let window = winit::window::WindowBuilder::new()
        .with_title("WEIRD")
        .with_resizable(false)
        .with_inner_size(screen_size)
        .build(&event_loop).unwrap()
        ;

    let window_context = softbuffer::Context::new(&window).unwrap();
    let mut surface = softbuffer::Surface::new(&window_context, &window).unwrap();
    let surface_size = screen_size.clone();
    _ = surface.resize(surface_size.width.try_into().unwrap(), surface_size.height.try_into().unwrap());

    let map_source = include_str!("../maps/test.wmt");
    let map = Map::load_from_wmt(map_source).unwrap();
    let mut camera = Camera {
        location: map.camera_location,
        rotation: map.camera_rotation,
        fov: std::f32::consts::PI * (2.0 / 3.0),
    };

    let mut render = Render::new();

    let mut timer = timer::Timer::new();
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
                    winit::event::WindowEvent::RedrawRequested => 'redraw: {
                        timer.response();

                        let mut mut_buffer = match surface.buffer_mut() {
                            Ok(buffer) => buffer,
                            Err(_) => break 'redraw,
                        };

                        'input_control: {
                            let input = input.get_state();

                            let ox = (input.is_key_pressed(KeyCode::KeyA) as i32 - input.is_key_pressed(KeyCode::KeyD) as i32) as f32;
                            let oy = (input.is_key_pressed(KeyCode::KeyW) as i32 - input.is_key_pressed(KeyCode::KeyS) as i32) as f32;

                            if ox == 0.0 && oy == 0.0 {
                                break 'input_control;
                            }

                            camera.rotation += ox * timer.get_delta_time() * 2.0;
                            camera.location.x += camera.rotation.cos() * oy * timer.get_delta_time() * 3.0;
                            camera.location.y += camera.rotation.sin() * oy * timer.get_delta_time() * 3.0;
                        }

                        mut_buffer.fill(0x000000);
                        mut_buffer[0] = 0x00FF00;
                        render.next_frame(&Surface {
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
