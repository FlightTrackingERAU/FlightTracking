#[macro_use]
extern crate conrod_core;
extern crate conrod_glium;
#[macro_use]
extern crate conrod_winit;
extern crate find_folder;
extern crate glium;
use button_widget::CircularButton;
use conrod_core::{
    widget::{self, button::ImageIds},
    Colorable, Labelable, Positionable, Sizeable, Widget,
};
use conrod_core::{
    text::Font, widget, widget_ids, Colorable, Labelable, Positionable, Sizeable, Widget,
};
use glam::DVec2;
use glium::Surface;
use tile_cache::TileCache;

use crate::button_widget::CircularButton;

mod map;
mod map_renderer;
mod tile_cache;
mod tile_requester;

mod button_widget;
mod support;
mod util;

mod button_widget;

const HEIGHT: u32 = 1000;
const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

const MAX_ZOOM_LEVEL: u32 = 20;

widget_ids!(pub struct Ids { fps_logger, text, viewport, map_images[], squares[], tiles[], square_text[], circular_button });
fn main() {
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
    widget_ids!(struct Ids {circle_button});
    let mut ids = Ids::new(ui.widget_id_generator());

    // Add the NotoSans font from the file
    let assets = find_folder::Search::KidsThenParents(3, 5)
        .for_folder("assets")
        .unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
    let regular = ui.fonts.insert_from_file(font_path).unwrap();

    /*
    let image_id = Image {
        normal: image_map.insert(airplane_icon),
        hover: image_map.insert(load_image(&display, image_path.join("airplane-icon.png"))),
        press: image_map.insert(load_image(&display, image_path.join("airplane-icon.png"))),
    };
    */

    // A type used for converting `conrod_core::render::Primitives` into `Command`s that can be used
    // for drawing to the glium `Surface`.
    let mut ids = Ids::new(ui.widget_id_generator());

    let mut image_map: conrod_core::image::Map<glium::Texture2d> = conrod_core::image::Map::new();

    let noto_sans_ttf = include_bytes!("../assets/fonts/NotoSans/NotoSans-Regular.ttf");
    let font = Font::from_bytes(noto_sans_ttf).expect("Failed to decode font");
    ui.fonts.insert(font);

    let mut renderer = conrod_glium::Renderer::new(&display).unwrap();

    let mut last_time = std::time::Instant::now();
    let mut frame_time_ms = 0.0;

    let runtime = tokio::runtime::Runtime::new().expect("Unable to create Tokio runtime!");

    let mut tile_cache = TileCache::new(&runtime);

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
                    // should_update_ui = false;

                    // Set the widgets.
                    let ui = &mut ui.set_widgets();

                    if let Some(_clicks) = CircularButton::new()
                        .color(conrod_core::color::WHITE)
                        .middle()
                        .w_h(256.0, 256.0)
                        .label_color(conrod_core::color::WHITE)
                        .label("Circular Button")
                        .set(ids.circle_button, ui)
                    {
                        println!("Dr. T is awesome, That is why he will curve the Test");
                    map_renderer::draw(
                        &mut tile_cache,
                        &viewer,
                        &display,
                        &mut image_map,
                        &mut ids,
                        ui,
                    );

                    let frame_time_str = format!(
                        "FT: {:.2}, FPS: {}",
                        frame_time_ms,
                        (1000.0 / frame_time_ms) as u32
                    );

                    widget::Text::new(frame_time_str.as_str())
                        .top_left()
                        .color(conrod_core::color::WHITE)
                        .justify(conrod_core::text::Justify::Right)
                        .font_size(12)
                        .set(ids.fps_logger, ui);
                    // Request redraw if the `Ui` has changed.
                    display.gl_window().window().request_redraw();
                }
            }
            glium::glutin::event::Event::RedrawRequested(_) => {
                //render and swap buffers
                let primitives = ui.draw();

                renderer.fill(&display, primitives, &image_map);
                let mut target = display.draw();
                target.clear_color(0.21, 0.32, 0.4, 1.0);
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

fn load_image<P>(display: &glium::Display, path: P) -> glium::texture::Texture2d
where
    P: AsRef<std::path::Path>,
{
    let path = path.as_ref();
    let rgba_image = image::open(&std::path::Path::new(&path)).unwrap().to_rgba();
    let image_dimensions = rgba_image.dimensions();
    let raw_image = glium::texture::RawImage2d::from_raw_rgba_reversed(
        &rgba_image.into_raw(),
        image_dimensions,
    );
    let texture = glium::texture::Texture2d::new(display, raw_image).unwrap();
    texture
}
