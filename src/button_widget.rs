use conrod_core::text::rt::Rect;
use conrod_core::widget::button::{Flat, FlatIds, Image, ImageColor, ImageIds};
use conrod_core::{
    self, text, widget, widget::button, widget_ids, Color, Colorable, FontSize, Labelable, Point,
    Positionable, Scalar, Widget,
};
use conrod_core::{image, Sizeable};
use conrod_core::{Borderable, UiCell};

use conrod_core::position;

///Circular Button Implementation.
#[derive(WidgetCommon)]
pub struct CircularButton<'a, S> {
    /// An object that handles some of the dirty work of rendering a GUI. We don't
    /// really have to worry about it.
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    /// Optional label string for the button.
    maybe_label: Option<&'a str>,
    ///What type of Button, Image or Flat
    show: S,
    /// See the Style struct below.
    style: Style,
    /// Whether the button is currently enabled, i.e. whether it responds to
    /// user input.
    enabled: bool,
}

#[derive(Copy, Clone)]
enum Interaction {
    Idle,
    Hover,
    Press,
}

// We use `#[derive(WidgetStyle)] to vastly simplify the definition and implementation of the
// widget's associated `Style` type. This generates an implementation that automatically
// retrieves defaults from the provided theme in the following order:
//
// 1. If the field is `None`, falls back to the style stored within the `Theme`.
// 2. If there are no style defaults for the widget in the `Theme`, or if the
//    default field is also `None`, falls back to the expression specified within
//    the field's `#[conrod(default = "expr")]` attribute.

/// Represents the unique styling for our CircularButton widget.
#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle)]
pub struct Style {
    /// Color of the Button's pressable area.
    #[conrod(default = "theme.shape_color")]
    pub color: Option<Color>,
    /// Width of the border surrounding the button
    #[conrod(default = "theme.border_width")]
    pub border: Option<Scalar>,
    /// The color of the border.
    #[conrod(default = "theme.border_color")]
    pub border_color: Option<Color>,
    /// The color of the Button's label.
    #[conrod(default = "theme.label_color")]
    pub label_color: Option<Color>,
    /// The font size of the Button's label.
    #[conrod(default = "theme.font_size_medium")]
    pub label_font_size: Option<FontSize>,
    /// The ID of the font used to display the label.
    #[conrod(default = "theme.font_id")]
    pub label_font_id: Option<Option<text::font::Id>>,
    /// The label's typographic alignment over the *x* axis.
    #[conrod(default = "text::Justify::Center")]
    pub label_justify: Option<text::Justify>,
}

impl<'a, S> CircularButton<'a, S> {
    /// Create a button context to be built upon.
    pub fn new(show: S) -> Self {
        CircularButton {
            common: widget::CommonBuilder::default(),
            style: Style::default(),
            show,
            maybe_label: None,
            enabled: true,
        }
    }

    /// Specify the font used for displaying the label.
    pub fn label_font_id(mut self, font_id: text::font::Id) -> Self {
        self.style.label_font_id = Some(Some(font_id));
        self
    }

    /// Align the label to the left of the `Button`'s surface.
    pub fn left_justify_label(mut self) -> Self {
        self.style.label_justify = Some(text::Justify::Left);
        self
    }

    /// Align the label to the mid-left of the `Button`'s surface.
    ///
    /// This is the default label alignment.
    pub fn center_justify_label(mut self) -> Self {
        self.style.label_justify = Some(text::Justify::Center);
        self
    }

    /// Align the label to the mid-left of the `Button`'s surface.
    pub fn right_justify_label(mut self) -> Self {
        self.style.label_justify = Some(text::Justify::Right);
        self
    }

    /// If true, will allow user inputs.  If false, will disallow user inputs.  Like
    /// other Conrod configs, this returns self for chainability. Allow dead code
    /// because we never call this in the example.
    #[allow(dead_cde)]
    pub fn enabled(mut self, flag: bool) -> Self {
        self.enabled = flag;
        self
    }
}

impl<'a> CircularButton<'a, Image> {
    ///Build button with Image given
    pub fn image(image_id: image::Id) -> Self {
        let image = Image {
            image_id,
            hover_image_id: None,
            press_image_id: None,
            src_rect: None,
            color: ImageColor::None,
        };

        Self::new(image)
    }

    ///Area that image will appear on
    ///
    ///On a rectangular area
    pub fn source_rectangle(mut self, rect: conrod_core::position::Rect) -> Self {
        self.show.src_rect = Some(rect);
        self
    }

    pub fn image_color(mut self, color: Color) -> Self {
        self.show.color = ImageColor::Normal(color);
        self
    }

    ///Color of the button when highlited or clicked
    pub fn image_color_with_feedback(mut self, color: Color) -> Self {
        self.show.color = ImageColor::WithFeedback(color);
        self
    }

    pub fn hover_image(mut self, id: image::Id) -> Self {
        self.show.hover_image_id = Some(id);
        self
    }

    pub fn press_image(mut self, id: image::Id) -> Self {
        self.show.press_image_id = Some(id);
        self
    }
}

impl<'a> CircularButton<'a, Flat> {
    ///Flat colored Circular Button widget
    pub fn new_flat() -> Self {
        Self::new(Flat::default())
    }

    ///Override the default style
    pub fn with_style(mut self, s: Style) -> Self {
        self.style = s;
        self
    }

    ///Hover color
    pub fn hover_color(mut self, color: Color) -> Self {
        self.show.hover_color = Some(color);
        self
    }

    ///Press Color
    pub fn press_color(mut self, color: Color) -> Self {
        self.show.press_color = Some(color);
        self
    }
}

impl<'a> Widget for CircularButton<'a, Flat> {
    /// The State struct that we defined above.
    type State = FlatIds;
    /// The Style struct that we defined using the `widget_style!` macro.
    type Style = Style;
    /// The event produced by instantiating the widget.
    ///
    /// `Some` when clicked, otherwise `None`.
    type Event = Option<()>;

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        FlatIds::new(id_gen)
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    /// This method is optional to implement. By default, the bounding rectangle of the widget
    /// is used.
    fn is_over(&self) -> widget::IsOverFn {
        use conrod_core::graph::Container;
        use conrod_core::Theme;
        fn is_over_widget(widget: &Container, _: Point, _: &Theme) -> widget::IsOver {
            let unique = widget.state_and_style::<FlatIds, Style>().unwrap();
            unique.state.rectangle.into()
        }
        is_over_widget
    }

    /// Update the state of the button by handling any input that has occurred since the last
    /// update.
    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs {
            id,
            state,
            rect,
            ui,
            style,
            ..
        } = args;

        let CircularButton {
            common,
            maybe_label,
            ..
        } = self;

        let (color, event) = {
            let input = ui.widget_input(id);

            // If the button was clicked, produce `Some` event.
            let event = input.clicks().left().next().map(|_| ());

            let color = style.color(&ui.theme);
            let color = input.mouse().map_or(color, |mouse| {
                if mouse.buttons.left().is_down() {
                    color.clicked()
                } else {
                    color.highlighted()
                }
            });

            (color, event)
        };

        // Finally, we'll describe how we want our widget drawn by simply instantiating the
        // necessary primitive graphics widgets.
        //
        // Conrod will automatically determine whether or not any changes have occurred and
        // whether or not any widgets need to be re-drawn.
        //
        // The primitive graphics widgets are special in that their unique state is used within
        // conrod's backend to do the actual drawing. This allows us to build up more complex
        // widgets by using these simple primitives with our familiar layout, coloring, etc
        // methods.
        //
        // If you notice that conrod is missing some sort of primitive graphics that you
        // require, please file an issue or open a PR so we can add it! :)

        // First, we'll draw the **Circle** with a radius that is half our given width.
        let radius = rect.w() / 2.0;
        widget::Circle::fill(radius)
            .middle_of(id)
            .graphics_for(id)
            .color(color)
            .set(state.label, ui);

        // Now we'll instantiate our label using the **Text** widget.
        if let Some(ref label) = self.maybe_label {
            let label_color = style.label_color(&ui.theme);
            let font_size = style.label_font_size(&ui.theme);
            let font_id = style.label_font_id(&ui.theme).or(ui.fonts.ids().next());
            widget::Text::new(label)
                .and_then(font_id, widget::Text::font_id)
                .middle_of(id)
                .font_size(font_size)
                .graphics_for(id)
                .color(label_color)
                .set(state.label, ui);
        }

        event
    }
}

impl<'a> Widget for CircularButton<'a, Image> {
    type State = ImageIds;
    type Style = conrod_core::widget::button::Style;
    type Event = Option<()>;

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        ImageIds::new(id_gen)
    }

    fn style(&self) -> Self::Style {
        self.style()
    }

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let conrod_core::widget::UpdateArgs {
            id,
            state,
            style,
            rect,
            ui,
            ..
        } = args;
        let CircularButton {
            maybe_label, show, ..
        } = self;

        let (interaction, times_triggered) = interaction_and_times_triggered(id, ui);

        let Image {
            image_id,
            hover_image_id,
            press_image_id,
            color,
            src_rect,
        } = show;

        let image_id = match interaction {
            Interaction::Idle => image_id,
            Interaction::Hover => hover_image_id.unwrap_or(image_id),
            Interaction::Press => press_image_id.or(hover_image_id).unwrap_or(image_id),
        };

        let (x, y, w, h) = rect.x_y_w_h();

        let mut image = widget::Image::new(image_id)
            .x_y(x, y)
            .w_h(w, h)
            .parent(id)
            .graphics_for(id);

        image.src_rect = src_rect;
        image.style.maybe_color = match color {
            ImageColor::Normal(color) => Some(Some(color)),
            ImageColor::WithFeedback(color) => ui
                .widget_input(id)
                .mouse()
                .map(|mouse| {
                    if mouse.buttons.left().is_down() {
                        Some(color.clicked())
                    } else {
                        Some(color.highlighted())
                    }
                })
                .or(Some(Some(color))),
            ImageColor::None => None,
        };

        image.set(state.image, ui);

        Some(())
    }
}
fn interaction_and_times_triggered(button_id: widget::Id, ui: &UiCell) -> (Interaction, u16) {
    let input = ui.widget_input(button_id);
    let mouse_interaction = input.mouse().map_or(Interaction::Idle, |mouse| {
        if mouse.buttons.left().is_down() {
            if ui.global_input().current.widget_under_mouse == Some(button_id) {
                Interaction::Press
            } else {
                Interaction::Idle
            }
        } else {
            Interaction::Hover
        }
    });
    let interaction = match mouse_interaction {
        Interaction::Idle | Interaction::Hover => {
            let is_touch_press = ui
                .global_input()
                .current
                .touch
                .values()
                .any(|t| t.start.widget == Some(button_id) && t.widget == Some(button_id));
            if is_touch_press {
                Interaction::Press
            } else {
                mouse_interaction
            }
        }
        Interaction::Press => Interaction::Press,
    };
    let times_triggered = (input.clicks().left().count() + input.taps().count()) as u16;
    (interaction, times_triggered)
}

/// Provide the chainable color() configuration method.
impl<'a, S> Colorable for CircularButton<'a, S> {
    fn color(mut self, color: conrod_core::Color) -> Self {
        self.style.color = Some(color);
        self
    }
}

/// Provide the chainable label(), label_color(), and label_font_size()
/// configuration methods.
impl<'a, S> Labelable<'a> for CircularButton<'a, S> {
    fn label(mut self, text: &'a str) -> Self {
        self.maybe_label = Some(text);
        self
    }
    fn label_color(mut self, color: conrod_core::Color) -> Self {
        self.style.label_color = Some(color);
        self
    }
    fn label_font_size(mut self, size: conrod_core::FontSize) -> Self {
        self.style.label_font_size = Some(size);
        self
    }
}
