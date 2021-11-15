use std::time::{Duration, Instant};

use conrod_core::{text::Font, widget, widget_ids, Colorable, Positionable, Sizeable, Widget};
use glam::DVec2;
use glium::Surface;

mod airports;
mod button_widget;
mod map;
mod map_renderer;
mod plane_renderer;
mod request_plane;
mod support;
mod tile;
mod ui_filter;
mod util;

pub use airports::*;
pub use button_widget::*;
pub use map::*;
pub use map_renderer::*;
pub use plane_renderer::*;
pub use request_plane::*;
use statrs::statistics::OrderStatistics;
pub use tile::*;
pub use ui_filter::*;
pub use util::*;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;
const MAX_ZOOM_LEVEL: u32 = 20;

widget_ids!(pub struct Ids {
    debug_menu[],
    text,
    viewport,
    map_images[],
    satellite_tiles[],
    tiles[],
    weather_tiles[],
    weather_button,
    airplane_button,
    debug_button,
    bench_button,
    latitude_lines[],
    latitude_text[],
    longitude_lines[],
    longitude_text[],
    filer_button[],
    airports[],
    planes[],
    square
});

use std::fmt::Write;
pub use util::MAP_PERF_DATA;

/// The app's "main" function. Our real main inside `main.rs` calls this function
pub fn run_app() {
    // Create our UI's event loop
    let event_loop = glium::glutin::event_loop::EventLoop::new();
    let window = glium::glutin::window::WindowBuilder::new()
        .with_title("Flight Tracker")
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(WIDTH, HEIGHT));

    let context = glium::glutin::ContextBuilder::new()
        .with_vsync(false)
        .with_multisampling(4);

    let display = glium::Display::new(window, context, &event_loop).unwrap();

    let mut map_ui = conrod_core::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();
    let mut overlay_ui = conrod_core::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

    // Generate our widget identifiers
    let mut map_ids = Ids::new(map_ui.widget_id_generator());
    let mut overlay_ids = Ids::new(overlay_ui.widget_id_generator());

    let mut image_map: conrod_core::image::Map<glium::Texture2d> = conrod_core::image::Map::new();

    // Making airplane image ids
    let airplane_image_bytes = include_bytes!("../assets/images/airplane-icon.png");
    let airplane_id = return_image_essentials(&display, airplane_image_bytes, &mut image_map);

    let weather_image_bytes = include_bytes!("../assets/images/weather-icon.png");
    let weather_id = return_image_essentials(&display, weather_image_bytes, &mut image_map);

    let gear_icon_bytes = include_bytes!("../assets/images/gear-icon.png");
    let gear_id = return_image_essentials(&display, gear_icon_bytes, &mut image_map);

    let bench_icon_bytes = include_bytes!("../assets/images/bench-icon.png");
    let bench_id = return_image_essentials(&display, bench_icon_bytes, &mut image_map);

    let noto_sans_ttf = include_bytes!("../assets/fonts/NotoSans/NotoSans-Regular.ttf");
    let noto_sans = Font::from_bytes(noto_sans_ttf).expect("Failed to decode font");
    let _noto_sans = overlay_ui.fonts.insert(noto_sans);

    let b612_ttf = include_bytes!("../assets/fonts/B612Mono/B612Mono-Regular.ttf");
    let b612 = Font::from_bytes(b612_ttf).expect("Failed to decode font");
    let b612_overlay = overlay_ui.fonts.insert(b612.clone());
    let b612_map = map_ui.fonts.insert(b612);

    let mut map_renderer = conrod_glium::Renderer::new(&display).unwrap();
    let mut overlay_renderer = conrod_glium::Renderer::new(&display).unwrap();
    let mut plane_renderer = PlaneRenderer::new(&display);

    let mut last_time = std::time::Instant::now();
    let mut frame_time_ms = 0.0;

    let runtime = tokio::runtime::Runtime::new().expect("Unable to create Tokio runtime!");

    let mut pipelines = tile::pipelines(&runtime);
    let mut plane_requester = PlaneRequester::new(&runtime);

    let airports_bin = include_bytes!("../assets/data/airports.bin");
    let airports = airports_from_bytes(airports_bin).expect("Failed to load airports");

    let mut should_update_ui = true;
    let mut viewer = map::TileView::new(29.18796, -81.04923, 8.0, 1080.0 / 2.0);
    let mut last_cursor_pos: Option<DVec2> = None;
    let mut left_pressed = false;

    let mut weather_enabled = true;
    let mut debug_enabled = true;

    let mut last_fps_print = Instant::now();
    let mut frame_counter = 0;
    let mut frame_times: Option<(Vec<f64>, Instant)> = None;

    overlay_ids
        .filer_button
        .resize(4, &mut overlay_ui.widget_id_generator());

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
            map_ui.handle_event(event.clone());
            overlay_ui.handle_event(event);
            should_update_ui = true;
        }

        match &event {
            glium::glutin::event::Event::MainEventsCleared => {
                if should_update_ui {
                    // should_update_ui = false;

                    // Set the widgets.
                    let mut map_ui = map_ui.set_widgets();
                    let map_ui = &mut map_ui;
                    let mut overlay_ui = overlay_ui.set_widgets();
                    let overlay_ui = &mut overlay_ui;

                    //========== Draw Map ==========
                    {
                        let map_state = map_renderer::MapRendererState {
                            tile_cache: &mut pipelines,
                            view: &viewer,
                            display: &display,
                            image_map: &mut image_map,
                            ids: &mut map_ids,
                            weather_enabled,
                        };
                        map_renderer::draw(map_state, map_ui, b612_map);
                    }

                    //========== Draw Airports ==========

                    airports::airport_renderer::draw(
                        &airports,
                        &viewer,
                        &display,
                        &mut map_ids,
                        map_ui,
                    );

                    //========== Draw Debug Data ==========

                    let perf_data = crate::take_profile_data();

                    if debug_enabled {
                        let _scope_debug_view = crate::profile_scope("Render Debug Information");
                        let mut perf_data: Vec<_> = perf_data.into_iter().collect();
                        perf_data.sort_unstable_by(|a, b| a.0.cmp(b.0));

                        //========== Draw Debug Text ==========
                        let map_data = {
                            let mut guard = MAP_PERF_DATA.lock();
                            guard.snapshot()
                        };

                        let debug_lines = 4 + map_data.backend_request_secs.len() + perf_data.len();

                        let mut i = 0;
                        let mut buf: util::StringFormatter<512> = util::StringFormatter::new();
                        overlay_ids
                            .debug_menu
                            .resize(debug_lines, &mut overlay_ui.widget_id_generator());

                        let mut draw_text = |args: std::fmt::Arguments<'_>| {
                            buf.clear();
                            buf.write_fmt(args).unwrap();
                            let gui_text = widget::Text::new(buf.as_str())
                                .color(conrod_core::color::WHITE)
                                .left_justify()
                                .font_size(8)
                                .font_id(b612_overlay);

                            let width = gui_text.get_w(overlay_ui).unwrap();
                            let x = -overlay_ui.win_w / 2.0 + width / 2.0 + 4.0;
                            let y = overlay_ui.win_h / 2.0 - 8.0 - i as f64 * 11.0;
                            gui_text
                                .x_y(x, y)
                                .set(overlay_ids.debug_menu[i], overlay_ui);
                            i += 1;
                            assert!(i <= debug_lines);
                        };

                        draw_text(format_args!(
                            "FT: {:.2}, FPS: {}",
                            frame_time_ms,
                            (1000.0 / frame_time_ms) as u32
                        ));
                        draw_text(format_args!(
                            "Zoom: {}, Tiles: {}",
                            map_data.zoom, map_data.tiles_rendered
                        ));
                        draw_text(format_args!(
                            "Decode: {:.2}ms, Upload: {:.2}ms",
                            map_data.tile_decode_time.as_secs_f64() * 1000.0,
                            map_data.tile_upload_time.as_secs_f64() * 1000.0
                        ));

                        for (backend_name, time) in map_data.backend_request_secs {
                            draw_text(format_args!("  {} {:?}", backend_name, time,));
                        }
                        for (name, data) in perf_data {
                            let samples = data.get_samples();
                            if samples.len() == 1 {
                                draw_text(format_args!("{}: {:?}", name, samples[0]));
                            } else {
                                let avg: Duration =
                                    samples.iter().sum::<Duration>() / samples.len() as u32;
                                draw_text(format_args!(
                                    "{}: {} times, {:?} avg",
                                    name,
                                    samples.len(),
                                    avg
                                ));
                            };
                        }
                    }
                    //========== Draw Buttons ==========
                    let scope_render_buttons = crate::profile_scope("Render Buttons");

                    let widget_x_position = (overlay_ui.win_w / 2.0) * 0.95 - 25.0;
                    let widget_y_position = (overlay_ui.win_h / 2.0) * 0.90;

                    if button_widget::draw_circle_with_image(
                        overlay_ids.weather_button,
                        overlay_ui,
                        weather_id,
                        widget_x_position,
                        widget_y_position - 70.0,
                    ) {
                        weather_enabled = !weather_enabled;
                    }

                    if button_widget::draw_circle_with_image(
                        overlay_ids.debug_button,
                        overlay_ui,
                        gear_id,
                        widget_x_position,
                        widget_y_position - 140.0,
                    ) {
                        debug_enabled = !debug_enabled;
                    }

                    button_widget::draw_circle_with_image(
                        overlay_ids.airplane_button,
                        overlay_ui,
                        airplane_id,
                        widget_x_position,
                        widget_y_position,
                    );

                    if button_widget::draw_circle_with_image(
                        overlay_ids.bench_button,
                        overlay_ui,
                        bench_id,
                        widget_x_position,
                        widget_y_position - 210.0,
                    ) {
                        let now = Instant::now();
                        match frame_times.take() {
                            Some((vec, start)) => {
                                println!("Captured {} samples over {:?}", vec.len(), now - start);
                                let mut data = statrs::statistics::Data::new(vec);
                                println!("  1st  percentile: {:.2}ms", data.percentile(1));
                                println!("  5th  percentile: {:.2}ms", data.percentile(5));
                                println!("  Mean FT:         {:.2}ms", data.percentile(50));
                                println!("  95th percentile: {:.2}ms", data.percentile(95));
                                println!("  99th percentile: {:.2}ms", data.percentile(99));
                                frame_times = None;
                            }
                            None => {
                                frame_times = Some((Vec::new(), now));
                                println!("Starting frame profiler");
                            }
                        }
                    }

                    ui_filter::draw(
                        overlay_ids.filer_button[0],
                        overlay_ui,
                        String::from("American Airlines"),
                        widget_x_position - 130.0,
                        widget_y_position,
                    );

                    ui_filter::draw(
                        overlay_ids.filer_button[1],
                        overlay_ui,
                        String::from("Spirit"),
                        widget_x_position - 130.0,
                        widget_y_position - 40.0,
                    );

                    ui_filter::draw(
                        overlay_ids.filer_button[2],
                        overlay_ui,
                        String::from("Southwest"),
                        widget_x_position - 130.0,
                        widget_y_position - 80.0,
                    );

                    ui_filter::draw(
                        overlay_ids.filer_button[3],
                        overlay_ui,
                        String::from("United"),
                        widget_x_position - 130.0,
                        widget_y_position - 120.0,
                    );
                    scope_render_buttons.end();

                    frame_counter += 1;
                    let now = Instant::now();
                    if now - last_fps_print >= Duration::from_secs(1) {
                        let _ = frame_counter;
                        //println!("FPS: {}", frame_counter);
                        last_fps_print = now;
                        frame_counter = 0;
                    }

                    //Time calculations
                    let now = std::time::Instant::now();
                    frame_time_ms = (now - last_time).as_nanos() as f64 / 1_000_000.0;
                    if let Some((vec, _)) = &mut frame_times {
                        vec.push(frame_time_ms);
                    }
                    last_time = now;

                    display.gl_window().window().request_redraw();
                }
            }
            glium::glutin::event::Event::RedrawRequested(_) => {
                //render and swap buffers
                let map_primitives = map_ui.draw();
                let overlay_primitives = overlay_ui.draw();

                let mut target = display.draw();
                target.clear_color(0.21, 0.32, 0.4, 1.0);

                map_renderer.fill(&display, map_primitives, &image_map);
                map_renderer
                    .draw(&display, &mut target, &image_map)
                    .unwrap();

                //=========Draw Planes============

                plane_renderer.draw(&display, &mut target, &mut plane_requester, &viewer);

                //=========Draw Overlay===========

                overlay_renderer.fill(&display, overlay_primitives, &image_map);
                overlay_renderer
                    .draw(&display, &mut target, &image_map)
                    .unwrap();

                target.finish().unwrap();
            }
            _ => {}
        }
    })
}

// Function to return the Id for images
// Must convert image path to bytes
fn return_image_essentials(
    display: &glium::Display,
    bytes: &[u8],
    image_map: &mut conrod_core::image::Map<glium::Texture2d>,
) -> ImageId {
    let image_2d = load_image(display, bytes);

    ImageId {
        normal: image_map.insert(image_2d),
        hover: image_map.insert(load_image(display, bytes)),
        press: image_map.insert(load_image(display, bytes)),
    }
}

// Load an image from our assets folder as a texture we can draw to the screen.
fn load_image(display: &glium::Display, bytes: &[u8]) -> glium::texture::Texture2d {
    let rgba_image = image::load_from_memory(bytes).unwrap().to_rgba();
    let image_dimensions = rgba_image.dimensions();
    let raw_image = glium::texture::RawImage2d::from_raw_rgba_reversed(
        &rgba_image.into_raw(),
        image_dimensions,
    );
    glium::texture::Texture2d::new(display, raw_image).unwrap()
}
