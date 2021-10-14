use conrod_core::{widget, widget_ids, Colorable, Positionable, Sizeable, Widget};
use glam::DVec2;
use glium::Surface;
use maptiler_cloud::{Maptiler, TileRequest};

mod map;
mod map_renderer;

mod support;
mod util;

const WIDTH: u32 = 128 * 4;
const HEIGHT: u32 = 128 * 4;
const ZOOM: u32 = 2;

widget_ids!(pub struct Ids { fps_logger, text, viewport, map_images[], squares[], square_text[] });

fn main() {
    // Create our Tokio runtime
    let runtime = tokio::runtime::Runtime::new().unwrap();

    // Create our Maptiler Cloud session to get our map tiles
    let maptiler = Maptiler::new("VrgC04XoV1a84R5VkUnL");

    // Create our UI's event loop
    let event_loop = glium::glutin::event_loop::EventLoop::new();
    let window = glium::glutin::window::WindowBuilder::new()
        .with_title("Conrod Window")
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(WIDTH, HEIGHT));

    let context = glium::glutin::ContextBuilder::new()
        .with_vsync(false)
        .with_multisampling(4);

    let display = glium::Display::new(window, context, &event_loop).unwrap();

    let mut ui = conrod_core::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

    // Generate our widget identifiers
    let mut ids = Ids::new(ui.widget_id_generator());
    ids.map_images.resize(64, &mut ui.widget_id_generator());

    // Load all of our map tiles dymammically
    let map_tiles = runtime.block_on(load_map(&maptiler, &display, ZOOM));

    // All tiles are the same size
    let (w, h) = (
        map_tiles.get(0).unwrap().get(0).unwrap().get_width(),
        map_tiles
            .get(0)
            .unwrap()
            .get(0)
            .unwrap()
            .get_height()
            .unwrap(),
    );

    let mut image_map = conrod_core::image::Map::new();

    let mut image_ids = Vec::new();

    // Create images id's for each tile
    let mut tile_index = 0;
    for tile_row in map_tiles {
        let mut row_image_ids = Vec::new();

        for tile in tile_row {
            let ids = (
                image_map.insert(tile),
                *ids.map_images.get(tile_index).unwrap(),
            );

            row_image_ids.push(ids);

            tile_index += 1;
        }

        image_ids.push(row_image_ids);
    }

    let assets = find_folder::Search::KidsThenParents(3, 5)
        .for_folder("assets")
        .unwrap();

    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    ui.fonts.insert_from_file(font_path).unwrap();

    let mut renderer = conrod_glium::Renderer::new(&display).unwrap();

    let mut last_time = std::time::Instant::now();
    let mut frame_time_ms = 0.0;

    let mut should_update_ui = true;
    let mut viewer = map::TileView::new(0.0, 0.0, 2.0, 1080 / 2);
    let mut last_cursor_pos: Option<DVec2> = None;
    let mut left_pressed = false;

    event_loop.run(move |event, _, control_flow| {
        use glium::glutin::event::{
            ElementState, Event, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent,
        };
        // Break from the loop upon `Escape` or closed window.
        if let Event::WindowEvent { event, .. } = &event {
            match event {
                // Break from the loop upon `Escape`.
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    input:
                        glium::glutin::event::KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = glium::glutin::event_loop::ControlFlow::Exit,
                WindowEvent::MouseWheel { delta, .. } => {
                    let zoom_change = match delta {
                        MouseScrollDelta::LineDelta(_x, y) => *y as f64,
                        MouseScrollDelta::PixelDelta(data) => data.y / 100.0,
                    };
                    let zoom_change = (-zoom_change / 6.0).clamp(-0.5, 0.5);
                    viewer.multiply_zoom(1.0 + zoom_change);
                }
                WindowEvent::CursorMoved { position, .. } => {
                    let position = DVec2::new(position.x, position.y);
                    if let Some(last) = last_cursor_pos {
                        let delta = (last - position).clamp_length_max(300.0);
                        if left_pressed {
                            viewer.move_camera_pixels(delta);
                        }
                    }

                    last_cursor_pos = Some(position);
                }
                WindowEvent::MouseInput { button, state, .. } => {
                    if matches!(button, MouseButton::Left) {
                        left_pressed = matches!(state, ElementState::Pressed);
                    }
                }
                _ => {}
            }
        }

        // Use the `winit` backend feature to convert the winit event to a conrod one.
        if let Some(event) = support::convert_event(&event, display.gl_window().window()) {
            ui.handle_event(event);
            should_update_ui = true;
        }

        match &event {
            glium::glutin::event::Event::MainEventsCleared => {
                if should_update_ui {
                    //should_update_ui = false;

                    // Set the widgets.
                    let ui = &mut ui.set_widgets();

                    // Render each tile
                    for (row, tile_row) in image_ids.iter().enumerate() {
                        for (col, (image_id, widget_id)) in tile_row.iter().enumerate() {
                            widget::Image::new(*image_id)
                                .w_h(w as f64, h as f64)
                                .x((row * 128) as f64 - 200.0) // I am not sure what these strange constants are
                                .y(((4 - col) * 128) as f64 - 300.0)
                                .set(*widget_id, ui);
                        }
                    }

                    let frame_time_str = format!(
                        "FT: {:.2}, FPS: {}",
                        frame_time_ms,
                        (1000.0 / frame_time_ms) as u32
                    );
                    widget::Text::new(frame_time_str.as_str())
                        .top_right()
                        .color(conrod_core::color::WHITE)
                        .justify(conrod_core::text::Justify::Right)
                        .font_size(12)
                        .set(ids.fps_logger, ui);

                    map_renderer::draw(&viewer, &mut ids, ui);

                    // Request redraw if the `Ui` has changed.
                    display.gl_window().window().request_redraw();
                }
            }
            glium::glutin::event::Event::RedrawRequested(_) => {
                //render and swap buffers
                let primitives = ui.draw();

                renderer.fill(&display, primitives, &image_map);
                let mut target = display.draw();
                target.clear_color(1.0, 0.0, 1.0, 1.0);
                renderer.draw(&display, &mut target, &image_map).unwrap();
                target.finish().unwrap();

                //Time calculations
                let now = std::time::Instant::now();
                frame_time_ms = (now - last_time).as_nanos() as f64 / 1_000_000.0;
                last_time = now;
            }
            _ => {}
        }
    })
}

// Loads all tiles in the map at a given zoom level
async fn load_map(
    maptiler: &Maptiler,
    display: &glium::Display,
    zoom: u32,
) -> Vec<Vec<glium::texture::Texture2d>> {
    let tiles_across = 1 << zoom;
    let mut tiles = Vec::new();

    for tile_row in 0..tiles_across {
        let mut tile_row_vec = Vec::new();

        for tile_col in 0..tiles_across {
            let tile = load_map_tile(maptiler, display, tile_row, tile_col, zoom).await;

            tile_row_vec.push(tile);
        }

        tiles.push(tile_row_vec);
    }

    tiles
}

// Loads a single tile from the Maptiler Cloud API
async fn load_map_tile(
    maptiler: &Maptiler,
    display: &glium::Display,
    x: u32,
    y: u32,
    zoom: u32,
) -> glium::texture::Texture2d {
    let tile_request = TileRequest::new(maptiler_cloud::TileSet::Satellite, x, y, zoom).unwrap();

    let jpeg_bytes = maptiler.request(tile_request).await.unwrap();

    let rgba_image = image::load_from_memory(&jpeg_bytes)
        .unwrap()
        .resize(128, 128, image::FilterType::Nearest)
        .into_rgba();
    let image_dimensions = rgba_image.dimensions();
    let raw_image = glium::texture::RawImage2d::from_raw_rgba_reversed(
        &rgba_image.into_raw(),
        image_dimensions,
    );
    glium::texture::Texture2d::new(display, raw_image).unwrap()
}
