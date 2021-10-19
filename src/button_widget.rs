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
pub struct CircularButton<'a> {
    /// An object that handles some of the dirty work of rendering a GUI. We don't
    /// really have to worry about it.
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    /// Optional label string for the button.
    maybe_label: Option<&'a str>,
    ///What type of Button, Image or Flat
    /// See the Style struct below.
    style: Style,
    /// Whether the button is currently enabled, i.e. whether it responds to
    /// user input.
    enabled: bool,
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
//This is how we generate it
widget_ids! {
    struct TextIds{
        circle,
        text,
    }
}

///Representation of Circle with Text
pub struct TextState {
    ids: TextIds,
}

impl<'a> CircularButton<'a> {
    ///Making a button context
    pub fn new() -> Self {
        CircularButton {
            common: widget::CommonBuilder::default(),
            style: Style::default(),
            maybe_label: None,
            enabled: true,
        }
    }

    ///Specify type of font used
    pub fn label_font_id(mut self, font_id: conrod_core::text::font::Id) -> Self {
        self.style.label_font_id = Some(Some(font_id));
        self
    }

    ///Enabled button
    #[allow(dead_code)]
    pub fn enabled(mut self, flag: bool) -> Self {
        self.enabled = flag;
        self
    }
}

impl<'a> Widget for CircularButton<'a> {
    type State = TextState;
    type Style = Style;
    type Event = Option<()>;

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        TextState {
            ids: TextIds::new(id_gen),
        }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
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

        //Drawing the circle
        widget::Circle::fill(radius)
            .middle_of(id)
            .graphics_for(id)
            .color(color)
            .set(state.ids.circle, ui);

        //Instantiate label
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
                .set(state.ids.text, ui);
        }

        event
    }
}

/// Provide the chainable color() configuration method.
impl<'a> Colorable for CircularButton<'a> {
    fn color(mut self, color: conrod_core::Color) -> Self {
        self.style.color = Some(color);
        self
    }
}

/// Provide the chainable label(), label_color(), and label_font_size()
/// configuration methods.
impl<'a> Labelable<'a> for CircularButton<'a> {
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
