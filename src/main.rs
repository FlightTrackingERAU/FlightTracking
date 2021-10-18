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
use glium::Surface;

mod button_widget;
mod support;

const WIDTH: u32 = 1000;
const HEIGHT: u32 = 1000;

fn main() {
    // Create our UI's event loop
    let event_loop = glium::glutin::event_loop::EventLoop::new();

    // Build the window
    let window = glium::glutin::window::WindowBuilder::new()
        .with_title("Conrod Window")
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(WIDTH, HEIGHT));

    let context = glium::glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_multisampling(4);

    let display = glium::Display::new(window, context, &event_loop).unwrap();

    // Construct our "UI" to hold our widgets/primitives
    let mut ui = conrod_core::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

    // Generate our widget identifiers
    widget_ids!(struct Ids {button, airplane_icon, background});
    let mut ids = Ids::new(ui.widget_id_generator());

    struct Image {
        normal: conrod_core::image::Id,
        hover: conrod_core::image::Id,
        press: conrod_core::image::Id,
    }

    let mut image_map: conrod_core::image::Map<glium::Texture2d> = conrod_core::image::Map::new();

    // Add the NotoSans font from the file
    let assets = find_folder::Search::KidsThenParents(3, 5)
        .for_folder("assets")
        .unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");

    let image_path = assets.join("images");
    let airplane_icon = load_image(&display, image_path.join("airplane-icon.png"));

    /*
    let image_id = Image {
        normal: image_map.insert(airplane_icon),
        hover: image_map.insert(load_image(&display, image_path.join("airplane-icon.png"))),
        press: image_map.insert(load_image(&display, image_path.join("airplane-icon.png"))),
    };
    */

    // A type used for converting `conrod_core::render::Primitives` into `Command`s that can be used
    // for drawing to the glium `Surface`.
    let mut renderer = conrod_glium::Renderer::new(&display).unwrap();

    let regular = ui.fonts.insert_from_file(font_path).unwrap();

    // The image map describing each of our widget->image mappings (in our case, none).
    let image_map = conrod_core::image::Map::<glium::texture::Texture2d>::new();

    let mut should_update_ui = true;
    event_loop.run(move |event, _, control_flow| {
        // Break from the loop upon `Escape` or closed window.
        match &event {
            glium::glutin::event::Event::WindowEvent { event, .. } => match event {
                // Break from the loop upon `Escape`.
                glium::glutin::event::WindowEvent::CloseRequested
                | glium::glutin::event::WindowEvent::KeyboardInput {
                    input:
                        glium::glutin::event::KeyboardInput {
                            virtual_keycode: Some(glium::glutin::event::VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = glium::glutin::event_loop::ControlFlow::Exit,
                _ => {}
            },
            _ => {}
        }

        // Use the `winit` backend feature to convert the winit event to a conrod one.
        if let Some(event) = support::convert_event(&event, &display.gl_window().window()) {
            ui.handle_event(event);
            should_update_ui = true;
        }

        match &event {
            glium::glutin::event::Event::MainEventsCleared => {
                if should_update_ui {
                    should_update_ui = false;

                    // Set the widgets.
                    let ui = &mut ui.set_widgets();

                    //CircularButton::image(image_id.normal)
                    //    .hover_image(image_id.hover)
                    //    .press_image(image_id.press)
                    //    .color(conrod_core::color::WHITE)
                    //    .w_h(200.0, 200.0)
                    //    .middle_of(ids.background)
                    //    .set(ids.circular_button, ui);
                    //
                    CircularButton::new_flat()
                        .color(conrod_core::color::WHITE)
                        .middle()
                        .w_h(200.0, 200.0)
                        .label_color(conrod_core::color::BLACK)
                        .label("Button")
                        .set(ids.button, ui);

                    // Add the widget to the conrod_core::Ui. This schedules the widget it to be
                    //rawn when we call Ui::draw.

                    // Request redraw if the `Ui` has changed.
                    display.gl_window().window().request_redraw();
                }
            }
            glium::glutin::event::Event::RedrawRequested(_) => {
                // Draw the `Ui` if it has changed.
                let primitives = ui.draw();

                renderer.fill(&display, primitives, &image_map);
                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                renderer.draw(&display, &mut target, &image_map).unwrap();
                target.finish().unwrap();
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
