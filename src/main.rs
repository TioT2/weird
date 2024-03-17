pub mod util;
pub mod timer;
pub mod input;
pub mod math;
pub mod camera;
pub mod map;
pub mod font;
pub mod surface;

use font::Font;
use map::*;
use math::*;
use surface::Surface;
use camera::Camera;

use input::KeyCode;

/// Renderer representation structure
struct Render {
} // struct Render

struct RenderContext<'a> {
    surface: &'a Surface,
    map: &'a Map,
    camera: &'a Camera,
    visit_stack: std::collections::VecDeque<SectorId>,
    floor_buffer: &'a mut [usize],
    ceil_buffer: &'a mut [usize],
    inv_depth_buffer: &'a mut [f32],
}

impl Render {
    /// Render create function
    pub fn new() -> Render {
        Render {
        }
    }

    pub unsafe fn draw_bar_unchecked(&self, surface: &Surface, x0: usize, y0: usize, x1: usize, y1: usize, color: u32) {
        let mut yptr = surface.data.add(y0 * surface.stride + x0);
        let yeptr = yptr.add((y1 - y0) * surface.stride);
        let dx = x1 - x0;

        while yptr != yeptr {
            let mut xptr = yptr;
            let xeptr = xptr.add(dx);

            while xptr != xeptr {
                *xptr = color;
                xptr = xptr.add(1);
            }

            yptr = yptr.add(surface.stride);
        }
    }

    fn clip_line(mut x0: isize, mut y0: isize, mut x1: isize, mut y1: isize, w: usize, h: usize) -> Option<(usize, usize, usize, usize)> {
        // Clip line
        const LOC_INSIDE: u32 = 0;
        const LOC_LEFT: u32 = 1;
        const LOC_RIGHT: u32 = 2;
        const LOC_BOTTOM: u32 = 4;
        const LOC_TOP: u32 = 7;

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
            (y0 - y1, surface.stride.wrapping_neg())
        } else {
            (y1 - y0, surface.stride)
        };
        let (mut dx, sx): (usize, usize) = if x1 < x0 {
            (x0 - x1, 1usize.wrapping_neg())
        } else {
            (x1 - x0, 1usize)
        };

        let mut pptr = surface.data.wrapping_add(y0 * surface.stride + x0);
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

    fn render_sector(context: &mut RenderContext, sector_id: SectorId, screen_x_begin: usize, screen_x_end: usize) {
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
                    // Clip edge if totally invisible
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
                ((p.x / p.y / 2.0 + 0.5) * context.surface.width as f32) as isize
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

            let (color, floor_color, ceil_color) = match context.visit_stack.len() {
                0 => (0xAACCAA, 0xDDFFDD, 0x779977),
                1 => (0xCCAAAA, 0xFFDDDD, 0x997777),
                2 => (0xAAAACC, 0xDDDDFF, 0x777799),
                _ => (0xBBBBBB, 0xEEEEEE, 0x888888),
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

            let (is_portal, next_sector) = match edge_type {
                EdgeType::Portal(next_sector) => (true, Some(next_sector)),
                EdgeType::Wall => (false, None)
            };
            let (neighbour_floor_delta, neighbour_ceiling_delta) = next_sector
                .and_then(|id| context.map.get_sector(*id))
                .map(|neighbour_sector| {
                    ((neighbour_sector.floor - sector.floor).max(0.0), (neighbour_sector.ceiling - sector.ceiling).min(0.0))
                })
                .unwrap_or((0.0, 0.0))
            ;

            for x in xp0..xp1 {
                let pixel_dir = Vec2f {
                    x: x as f32 / context.surface.width as f32 * 2.0 - 1.0,
                    y: 1.0,
                };

                // edge_distance
                let inv_distance = (pixel_dir.x * edge_norm.x + pixel_dir.y * edge_norm.y).abs() * inv_edge_distance;
                let mut ceil_y = ((((context.camera.height - sector.ceiling - neighbour_ceiling_delta) * inv_distance + 1.0) / 2.0 * context.surface.height as f32) as isize).clamp(0, context.surface.height as isize) as usize;
                let mut floor_y = ((((context.camera.height - sector.floor - neighbour_floor_delta) * inv_distance + 1.0) / 2.0 * context.surface.height as f32) as isize).clamp(0, context.surface.height as isize) as usize;

                unsafe {
                    let buf_floor = context.floor_buffer.get_unchecked_mut(x);
                    let buf_ceil = context.ceil_buffer.get_unchecked_mut(x);

                    ceil_y = ceil_y.clamp(*buf_ceil, *buf_floor);
                    floor_y = floor_y.clamp(*buf_ceil, *buf_floor);

                    let pbegin = context.surface.data.add(x);

                    let mut pptr = pbegin.add(context.surface.stride * *buf_ceil);
                    let pceil = pbegin.add(context.surface.stride * ceil_y);
                    let pfloor = pbegin.add(context.surface.stride * floor_y);
                    let pend = pbegin.add(context.surface.stride * *buf_floor);

                    *buf_ceil = ceil_y;
                    *buf_floor = floor_y;

                    *context.inv_depth_buffer.get_unchecked_mut(x) = inv_distance;

                    while pptr < pceil {
                        *pptr = ceil_color;
                        pptr = pptr.add(context.surface.stride);
                    }

                    if is_portal {
                        pptr = pfloor;
                    } else {
                        while pptr < pfloor {
                            *pptr = color;
                            pptr = pptr.add(context.surface.stride);
                        }
                    }

                    while pptr < pend {
                        *pptr = floor_color;
                        pptr = pptr.add(context.surface.stride);
                    }
                }
            }

            if let EdgeType::Portal(portal_sector_id) = edge_type {
                context.visit_stack.push_back(sector_id);

                if !context.visit_stack.contains(portal_sector_id) {
                    if xp1 - xp0 > 0 {
                        Self::render_sector(context, *portal_sector_id, xp0, xp1);
                    }
                }

                context.visit_stack.pop_back();
            };

        } // 'edge_loop
    } // fn render_sector

    /// Next frame rendering function
    /// `surface` - surface to render frame to
    /// `map` - map to render
    /// `sector_id` - id of sector to start rendering from
    pub fn render(&mut self, surface: &Surface, map: &Map, camera: &Camera, sector_id: SectorId) {
        // Render only if sector actually exists
        if map.get_sector(sector_id).is_some() {
            let mut context = RenderContext {
                surface,
                map,
                camera,
                visit_stack: std::collections::VecDeque::new(),
                floor_buffer: &mut {
                    let mut buffer = Vec::with_capacity(surface.width);
                    buffer.resize(surface.width, surface.height);
                    buffer
                },
                ceil_buffer: &mut {
                    let mut buffer = Vec::with_capacity(surface.width);
                    buffer.resize(surface.width, 0);
                    buffer
                },
                inv_depth_buffer: &mut {
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
    #[allow(unused)]
    pub fn render_minimap(&mut self, surface: &Surface, map: &Map, camera: &Camera, camera_sector: SectorId) {
        unsafe {
            self.draw_bar_unchecked(surface, 0, 0, surface.width, surface.height, 0x000000);
        }

        let render_sector = |sector: &Sector, color_scale: u32| {
            for (edge, edge_type) in sector.edges.iter().zip(sector.edge_types.iter()) {
                // Calculate edge projection
                let p0 = camera.to_space(edge.p0);
                let p1 = camera.to_space(edge.p1);

                // Project edge to pixel space and render, actually
                let edge_color = match edge_type {
                    EdgeType::Wall => 0x001100,
                    EdgeType::Portal(_) => 0x110000,
                } * color_scale;

                /*unsafe*/ {
                    self.draw_line(surface,
                        surface.width  as isize / 2 + (p0.x * 8.0) as isize,
                        surface.height as isize / 2 - (p0.y * 8.0) as isize,
                        surface.width  as isize / 2 + (p1.x * 8.0) as isize,
                        surface.height as isize / 2 - (p1.y * 8.0) as isize,
                        edge_color,
                    );
                }
            }
        };

        for (sector_id, sector) in map.iter_indexed_sectors() {
            if sector_id != camera_sector {
                render_sector(sector, 6);
            }
        }

        if let Some(sector) = map.get_sector(camera_sector) {
            render_sector(sector, 15);
        }

        // Render player
        let (x0, y0) = (surface.width / 2, surface.height / 2);

        unsafe {
            self.draw_bar_unchecked(surface, x0 - 1, y0 - 1, x0 + 2, y0 + 2, 0xFFFFFF);
            self.draw_line_unchecked(surface, x0, y0, x0, y0 - 5, 0xFFFFFF);
        }
    } // impl fn render_minimap
} // impl Render

/// Main program function
fn main() {
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    let screen_size = winit::dpi::LogicalSize::<u32>::new(700, 600);
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

    let map_name = std::env::args()
        .nth(1)
        .and_then(|arg| {
            std::fs::read_to_string(arg).ok()
        })
        .unwrap_or(include_str!("../maps/default.wmt").to_string());

    let map = Map::load_from_wmt(map_name.as_str()).unwrap();
    let mut camera = Camera::new();

    camera.set_location(map.camera_location, 0.5, map.camera_rotation);
    let mut camera_sector_id = map.find_sector(camera.location).unwrap();

    let mut render = Render::new();

    let mut timer = timer::Timer::new();
    let mut input = input::Input::new();

    let font = Font::default();

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

                        let mut mut_buffer = match surface.buffer_mut() {
                            Ok(buffer) => buffer,
                            Err(_) => break 'redraw,
                        };

                        if input.get_state().is_key_clicked(KeyCode::F11) {
                            if window.fullscreen().is_some() {
                                window.set_fullscreen(None);
                            } else {
                                if let Some(monitor) = window.current_monitor() {
                                    let mut best_index: Option<usize> = None;
                                    let mut best_count: Option<u32> = None;
                                    for (index, count) in monitor.video_modes()
                                        .enumerate()
                                        .map(|(index, mode)|
                                            (index, (mode.bit_depth() == 32) as u32 + ((mode.refresh_rate_millihertz() == 60000) as u32 + (mode.size() == winit::dpi::PhysicalSize::new(640, 480)) as u32 * 2))
                                        ) {
                                        if Some(count) > best_count {
                                            best_count = Some(count);
                                            best_index = Some(index);
                                        }
                                    }

                                    if let Some(index) = best_index {
                                        window.set_fullscreen(Some(winit::window::Fullscreen::Exclusive(monitor.video_modes().nth(index).unwrap())));
                                    }
                                }
                            }
                        }

                        'input_control: {
                            let input = input.get_state();
                            let dt = timer.get_delta_time();

                            let ox = (input.is_key_pressed(KeyCode::KeyA) as i32 - input.is_key_pressed(KeyCode::KeyD) as i32) as f32;
                            let oy = (input.is_key_pressed(KeyCode::KeyW) as i32 - input.is_key_pressed(KeyCode::KeyS) as i32) as f32;
                            let oz = (input.is_key_pressed(KeyCode::KeyR) as i32 - input.is_key_pressed(KeyCode::KeyF) as i32) as f32;

                            if ox == 0.0 && oy == 0.0 && oz == 0.0 {
                                break 'input_control;
                            }

                            let new_location = Vec2f {
                                x: camera.location.x + camera.direction.x * oy * dt * 3.0,
                                y: camera.location.y + camera.direction.y * oy * dt * 3.0,
                            };

                            if let Some(new_camera_sector_id) = map.find_sector_from_old(new_location, camera_sector_id) {
                                camera_sector_id = new_camera_sector_id;
                                let camera_sector = map.get_sector(camera_sector_id).unwrap();
                                let new_height = (camera.height + oz * dt * 3.0).clamp(camera_sector.floor, camera_sector.ceiling);
                                camera.set_location(new_location, new_height, camera.rotation + ox * dt * 2.0);
                            }
                        }

                        // Render main frame
                        render.render(&Surface {
                            data: mut_buffer.as_mut_ptr(),
                            width: surface_size.width as usize,
                            stride: surface_size.width as usize,
                            height: surface_size.height as usize,
                        }, &map, &camera, camera_sector_id);

                        let minimap_surface = Surface {
                            data: mut_buffer.as_mut_ptr(),
                            width: surface_size.width as usize / 6,
                            stride: surface_size.width as usize,
                            height: surface_size.height as usize / 6,
                        };

                        // Render minimap on subframe
                        // TODO: Fix minimap itself & it's style
                        // render.render_minimap(&minimap_surface, &map, &camera, camera_sector);

                        let font_size = font.get_letter_size();
                        font.put_string(&minimap_surface, 4, (font_size.height + 1) * 0 + 4, format!("FPS: {}", timer.get_fps()).as_str(), 0xFFFFFF);
                        font.put_string(&minimap_surface, 4, (font_size.height + 1) * 1 + 4, format!("X: {}", camera.location.x).as_str(), 0xFFFFFF);
                        font.put_string(&minimap_surface, 4, (font_size.height + 1) * 2 + 4, format!("Y: {}", camera.location.y).as_str(), 0xFFFFFF);

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
} // fn main

// file main.rs