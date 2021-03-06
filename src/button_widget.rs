use conrod_core::widget::button::{Flat, Image, ImageColor};
use conrod_core::{
    self, text, widget, widget_ids, Color, Colorable, FontSize, Labelable, Positionable, Scalar,
    UiCell, Widget,
};
use conrod_core::{image, Sizeable};

use conrod_core::{WidgetCommon, WidgetStyle};

///Image Id to communicate with the CircularImageIds.
///
///In case the image needs to change when being hovered or pressed.
#[derive(Clone, Copy)]
pub struct ImageId {
    ///Image Id for when the image is not doing anything
    pub normal: conrod_core::image::Id,
    ///Image Id for when the image is being hovered
    pub hover: conrod_core::image::Id,
    ///Image Id for when the image is being pressed
    pub press: conrod_core::image::Id,
}

///The structure of a Button.
#[derive(WidgetCommon)]
pub struct CircularButton<'a, S> {
    /// An object that handles some of the dirty work of rendering a GUI. We don't
    /// really have to worry about it.
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    /// Optional label string for the button.
    maybe_label: Option<&'a str>,
    ///Type of Button (Image or Text)
    show: S,
    ///What type of Button, Image or Flat
    /// See the Style struct below.
    style: Style,
}

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

//Here we will making a widget with Circle and Text for the label
//
//This is how we generate one
//This is how we generate it
widget_ids! {
    ///Widget Id for Text(Flat Circle)
    pub struct TextIds{
        circle,
        text,
    }
}

widget_ids! {
    ///Widget Id for Image
    pub struct CircularImageIds{
        circle,
        image,
    }
}

///Representation of Circle with Text
pub struct TextState {
    ids: TextIds,
}

///Representation of Circle with Image
pub struct ImageState {
    ids: CircularImageIds,
}

impl<'a, S> CircularButton<'a, S> {
    ///Making a button context

    ///Specify type of font used
    #[allow(dead_code)]
    pub fn label_font_id(mut self, font_id: conrod_core::text::font::Id) -> Self {
        self.style.label_font_id = Some(Some(font_id));
        self
    }
}

impl<'a> CircularButton<'a, Image> {
    ///Default constructor for an image
    pub fn image(image_id: conrod_core::image::Id) -> Self {
        CircularButton {
            common: widget::CommonBuilder::default(),
            maybe_label: None,
            show: Image {
                image_id,
                hover_image_id: None,
                press_image_id: None,
                color: ImageColor::None,
                src_rect: None,
            },
            style: Style::default(),
        }
    }

    ///The rectangular area of the image
    ///
    ///If not used it will use image's default size
    pub fn source_rectangle(mut self, rect: conrod_core::position::Rect) -> Self {
        self.show.src_rect = Some(rect);
        self
    }

    ///Image's illuminance
    pub fn image_color(mut self, color: conrod_core::color::Color) -> Self {
        self.show.color = ImageColor::Normal(color);
        self
    }

    ///Color will change slightly when button highlighted or clicked
    pub fn image_color_with_feedback(mut self, color: conrod_core::color::Color) -> Self {
        self.show.color = ImageColor::WithFeedback(color);
        self
    }

    ///Image displayed while hovering button
    pub fn hover_image(mut self, image_id: image::Id) -> Self {
        self.show.hover_image_id = Some(image_id);
        self
    }

    ///Image displayed when button is pressed
    pub fn press_image(mut self, image_id: image::Id) -> Self {
        self.show.press_image_id = Some(image_id);
        self
    }
}

impl<'a> CircularButton<'a, Flat> {
    pub fn new() -> Self {
        CircularButton {
            common: widget::CommonBuilder::default(),
            show: Flat {
                hover_color: Some(conrod_core::color::WHITE),
                press_color: Some(conrod_core::color::WHITE),
            },
            style: Style::default(),
            maybe_label: None,
        }
    }
}
impl<'a> Widget for CircularButton<'a, Image> {
    type State = ImageState;
    type Style = Style;
    type Event = bool;

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        ImageState {
            ids: CircularImageIds::new(id_gen),
        }
    }

    fn style(&self) -> Self::Style {
        self.style
    }

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
            maybe_label: _,
            show,
            ..
        } = self;

        // Instantiate image
        let Image {
            image_id,
            hover_image_id: _,
            press_image_id: _,
            color,
            src_rect,
        } = show;

        let (button_color, was_clicked) = {
            let input = ui.widget_input(id);

            //If button was clicked
            let was_clicked = input.clicks().left().next().is_some();

            let color = style.color(&ui.theme);
            let color = input.mouse().map_or(color, |mouse| {
                if mouse.buttons.left().is_down() {
                    color.clicked()
                } else {
                    color.highlighted()
                }
            });

            (color, was_clicked)
        };

        let radius = rect.w() / 2.0;

        //Drawing the circle
        widget::Oval::fill([radius * 2.0, radius * 2.0])
            .resolution(26)
            .middle_of(id)
            .graphics_for(id)
            .color(button_color)
            .set(state.ids.circle, ui);

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

        image.set(state.ids.image, ui);

        was_clicked
    }
}
impl<'a> Widget for CircularButton<'a, Flat> {
    type State = TextState;
    type Style = Style;
    type Event = Option<()>;

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        TextState {
            ids: TextIds::new(id_gen),
        }
    }

    fn style(&self) -> Self::Style {
        self.style
    }

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs {
            id,
            state,
            rect,
            ui,
            style,
            ..
        } = args;

        let (color, event) = {
            let input = ui.widget_input(id);

            //If button was clicked
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
        let radius = rect.w() / 2.0;

        // Drawing the circle
        widget::Oval::fill([radius * 2.0, radius * 2.0])
            .resolution(26)
            .middle_of(id)
            .graphics_for(id)
            .color(color)
            .set(state.ids.circle, ui);

        // Instantiate label
        if let Some(label) = self.maybe_label {
            let label_color = style.label_color(&ui.theme);
            let font_size = style.label_font_size(&ui.theme);
            let font_id = style
                .label_font_id(&ui.theme)
                .or_else(|| ui.fonts.ids().next());

            widget::Text::new(label)
                .and_then(font_id, widget::Text::font_id)
                .middle_of(id)
                .font_size(font_size)
                .graphics_for(id)
                .color(label_color)
                .set(state.ids.text, ui);
        }

        event
    }
}

///Uses CircularButton<Flat> constructor as default constructor
///
///This new() would only happen if the type of button is not specified
impl<'a> Default for CircularButton<'a, Flat> {
    fn default() -> Self {
        Self::new()
    }
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

///Makes a Circle Widget with an ImageId given.
///Takes x and y to position Widget.
pub fn draw_circle_with_image(
    widget: widget::id::Id,
    ui: &mut UiCell,
    image_id: ImageId,
    widget_x_position: f64,
    widget_y_position: f64,
) -> bool {
    CircularButton::image(image_id.normal)
        .x(widget_x_position)
        .y(widget_y_position)
        .w_h(50.0, 50.0)
        .label_color(conrod_core::color::WHITE)
        .label("Airplane Button")
        .set(widget, ui)
}
