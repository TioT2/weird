pub mod util;
pub mod timer;
pub mod input;
pub mod math;
pub mod camera;
pub mod map;
pub mod font;
pub mod surface;
pub mod map_editor;

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

            let neighbour_bounds = match edge_type {
                EdgeType::Portal(next_sector_id) => context.map
                    .get_sector(*next_sector_id)
                    .map(|neighbour_sector| (neighbour_sector.floor, neighbour_sector.ceiling))
                ,
                EdgeType::Wall => None,
            };

            for x in xp0..xp1 {
                let pixel_dir = Vec2f {
                    x: x as f32 / context.surface.width as f32 * 2.0 - 1.0,
                    y: 1.0,
                };

                // edge_distance
                let inv_distance = (pixel_dir.x * edge_norm.x + pixel_dir.y * edge_norm.y).abs() * inv_edge_distance;

                let to_screen_height = |height: f32| -> usize {
                    ((((context.camera.height - height) * inv_distance + 1.0) / 2.0 * context.surface.height as f32) as isize).clamp(0, context.surface.height as isize) as usize
                };

                let mut ceil_y = to_screen_height(sector.ceiling);
                let mut floor_y = to_screen_height(sector.floor);

                unsafe {
                    let buf_floor = context.floor_buffer.get_unchecked_mut(x);
                    let buf_ceil = context.ceil_buffer.get_unchecked_mut(x);

                    ceil_y = ceil_y.clamp(*buf_ceil, *buf_floor);
                    floor_y = floor_y.clamp(*buf_ceil, *buf_floor);

                    let p_base = context.surface.data.add(x);
                    let p_ceil = p_base.add(context.surface.stride * ceil_y);
                    let p_floor = p_base.add(context.surface.stride * floor_y);
                    let p_end = p_base.add(context.surface.stride * *buf_floor);

                    let mut p_current = p_base.add(context.surface.stride * *buf_ceil);

                    *context.inv_depth_buffer.get_unchecked_mut(x) = inv_distance;

                    // Ceiling
                    while p_current < p_ceil {
                        *p_current = ceil_color;
                        p_current = p_current.add(context.surface.stride);
                    }

                    if let Some((neighbour_floor, neighbour_ceiling)) = neighbour_bounds {
                        // Render neighbour borders
                        let neighbour_ceil_y = to_screen_height(neighbour_ceiling).clamp(ceil_y, floor_y);
                        let neighbour_floor_y = to_screen_height(neighbour_floor).clamp(ceil_y, floor_y);

                        let p_neighbour_ceil = p_base.add(context.surface.stride * neighbour_ceil_y);
                        let p_neighbour_floor = p_base.add(context.surface.stride * neighbour_floor_y);

                        // Upper wall
                        while p_current < p_neighbour_ceil {
                            *p_current = color;
                            p_current = p_current.add(context.surface.stride);
                        }

                        // Skip portal
                        p_current = p_neighbour_floor;

                        // Set hints for inner rendering
                        *buf_ceil = neighbour_ceil_y;
                        *buf_floor = neighbour_floor_y;

                        // Lower wall
                        while p_current < p_floor {
                            *p_current = color;
                            p_current = p_current.add(context.surface.stride);
                        }

                        /*
                        struct Edge {
                            lower_texture: TextureId,
                            middle_texture: Option<TextureId>,
                            upper_texture: TextureId,
                        }

                        struct Sector {
                            ceiling_height: f32,
                            floor_height: f32,
                            ceiling_texture: TextureId,
                            floor_texture: TextureId,
                            light: u8,

                            edges: Vec<Edge>,
                        }
                         */
                    } else {
                        // Middle block
                        while p_current < p_floor {
                            *p_current = color;
                            p_current = p_current.add(context.surface.stride);
                        }

                        // Don't actually need it
                        // *buf_ceil = ceil_y;
                        // *buf_floor = floor_y;
                    }

                    *context.inv_depth_buffer.get_unchecked_mut(x) = inv_distance;

                    // Floor
                    while p_current < p_end {
                        *p_current = floor_color;
                        p_current = p_current.add(context.surface.stride);
                    }
                }
            }

            // Deferred neighbour rendering
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
                    surface.draw_line(
                        surface.width  as isize / 2 + (p0.x * 6.0) as isize,
                        surface.height as isize / 2 - (p0.y * 6.0) as isize,
                        surface.width  as isize / 2 + (p1.x * 6.0) as isize,
                        surface.height as isize / 2 - (p1.y * 6.0) as isize,
                        edge_color,
                    );
                }
            }
        };

        if let Some(sector) = map.get_sector(camera_sector) {
            let adjacent_sectors = sector.edge_types
                .iter()
                .filter_map(|edge| {
                    if let EdgeType::Portal(sector_id) = *edge {
                        Some(sector_id)
                    } else {
                        None
                    }
                })
                .filter_map(|id| {
                    map.get_sector(id)
                })
            ;

            for sector in adjacent_sectors {
                render_sector(sector, 6);
            }
            render_sector(sector, 15);
        }

        // Render player
        let (x0, y0) = ((surface.width / 2) as isize, (surface.height / 2) as isize);

        surface.draw_bar( x0 - 1, y0 - 1, x0 + 2, y0 + 2, 0xFFFFFF);
        surface.draw_line( x0, y0, x0, y0 - 5, 0xFFFFFF);
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
            winit::event::Event::DeviceEvent { device_id, event } => {
                match event {
                    winit::event::DeviceEvent::Button { button, state } => {
                        let key_code = match button {
                            0 => Some(KeyCode::F30),
                            1 => Some(KeyCode::F31),
                            2 => Some(KeyCode::F32),
                            _ => None,
                        };
                        if let Some(key_code) = key_code {
                            input.on_key_state_change(key_code, state == winit::event::ElementState::Pressed);
                        }
                    }
                    _ => {}
                }
            }
            winit::event::Event::WindowEvent { window_id, event } => if window.id() == window_id {
                match event {
                    winit::event::WindowEvent::CloseRequested => {
                        target.exit();
                    }
                    winit::event::WindowEvent::KeyboardInput { event, .. } => if let winit::keyboard::PhysicalKey::Code(code) = event.physical_key {
                        input.on_key_state_change(code, event.state == winit::event::ElementState::Pressed);
                    }
                    winit::event::WindowEvent::CursorMoved { device_id, position } => {
                        let motion: winit::dpi::LogicalPosition<f32> = position.to_logical(window.scale_factor());

                        input.on_mouse_move(Vec2f {
                            x: motion.x,
                            y: motion.y,
                        });
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
                                // Find perfect suitable videomode
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

                            let ox = (input.is_key_pressed(KeyCode::KeyA) as i32 - input.is_key_pressed(KeyCode::KeyD) as i32) as f32 +
                                input.get_mouse_motion().x * (input.is_key_pressed(KeyCode::F30)) as i32 as f32;
                            let oy = (input.is_key_pressed(KeyCode::KeyW) as i32 - input.is_key_pressed(KeyCode::KeyS) as i32) as f32;
                            let oz = (input.is_key_pressed(KeyCode::KeyR) as i32 - input.is_key_pressed(KeyCode::KeyF) as i32) as f32;
                            let strafe = input.is_key_pressed(KeyCode::AltLeft);

                            if ox == 0.0 && oy == 0.0 && oz == 0.0 {
                                break 'input_control;
                            }

                            let mut camera_location_delta = Vec2f {
                                x: camera.direction.x * oy * 3.0,
                                y: camera.direction.y * oy * 3.0,
                            };
                            let mut new_rotation = camera.rotation;
                            if strafe {
                                camera_location_delta.x -= camera.right.x * ox * 3.0;
                                camera_location_delta.y -= camera.right.y * ox * 3.0;
                            } else {
                                new_rotation += ox * 2.0 * dt;
                            }
                            let new_height = camera.height + oz * 3.0 * dt;

                            let new_location = Vec2f {
                                x: camera.location.x + camera_location_delta.x * dt,
                                y: camera.location.y + camera_location_delta.y * dt,
                            };
                            let new_test_location = Vec2f {
                                x: camera.location.x + camera_location_delta.x * 0.01,
                                y: camera.location.y + camera_location_delta.y * 0.01,
                            };

                            if let Some(new_camera_sector_id) = map.find_adjacent_sector(new_test_location, camera_sector_id) {
                                let new_camera_sector = map.get_sector(new_camera_sector_id).unwrap();

                                if camera_sector_id == new_camera_sector_id {
                                    camera.set_location(
                                        new_location,
                                        new_height.clamp(new_camera_sector.floor, new_camera_sector.ceiling),
                                        new_rotation,
                                    );
                                } else {
                                    if new_height >= new_camera_sector.floor && new_height <= new_camera_sector.ceiling {
                                        camera.set_location(
                                            new_location,
                                            new_height,
                                            new_rotation,
                                        );
                                        camera_sector_id = new_camera_sector_id;
                                    }
                                }
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
                            width: surface_size.width as usize / 3,
                            stride: surface_size.width as usize,
                            height: surface_size.height as usize / 3,
                        };

                        // Render minimap on subframe
                        // TODO: Fix minimap itself & it's style
                        render.render_minimap(&minimap_surface, &map, &camera, camera_sector_id);

                        let font_size = font.get_letter_size();
                        font.put_string(&minimap_surface, 4, (font_size.height + 1) * 0 + 4, format!("FPS: {}", timer.get_fps()).as_str(), 0xFFFFFF);
                        font.put_string(&minimap_surface, 4, (font_size.height + 1) * 1 + 4, format!("X: {}", camera.location.x).as_str(), 0xFFFFFF);
                        font.put_string(&minimap_surface, 4, (font_size.height + 1) * 2 + 4, format!("Y: {}", camera.location.y).as_str(), 0xFFFFFF);
                        font.put_string(&minimap_surface, 4, (font_size.height + 1) * 3 + 4, format!("H: {}", camera.height    ).as_str(), 0xFFFFFF);

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
