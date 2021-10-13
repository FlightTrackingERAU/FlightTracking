#[macro_use]
extern crate conrod_core;
extern crate conrod_glium;
#[macro_use]
extern crate conrod_winit;
extern crate find_folder;
extern crate glium;
use button_widget::Circular_Button;
use conrod_core::{widget, Colorable, Labelable, Positionable, Sizeable, Widget};
use glium::Surface;

mod button_widget;
mod support;

const WIDTH: u32 = 400;
const HEIGHT: u32 = 400;

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
    widget_ids!(struct Ids {circle_button});
    let ids = Ids::new(ui.widget_id_generator());

    // Add the NotoSans font from the file
    let assets = find_folder::Search::KidsThenParents(3, 5)
        .for_folder("assets")
        .unwrap();
    let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");

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

                    for _clicks in Circular_Button::new()
                        .color(conrod_core::color::WHITE)
                        .middle()
                        .w_h(256.0, 256.0)
                        .label_font_id(regular)
                        .label_color(conrod_core::color::WHITE)
                        .label("Circular Button")
                        // Add the widget to the conrod_core::Ui. This schedules the widget it to be
                        // drawn when we call Ui::draw.
                        .set(ids.circle_button, ui)
                    {
                        println!("Dr.T is awesome");
                    }

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
