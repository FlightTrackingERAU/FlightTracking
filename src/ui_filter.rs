use conrod_core::Sizeable;
use conrod_core::{
    self, position, text, widget, widget_ids, Color, Colorable, FontSize, Labelable, Positionable,
    Scalar, UiCell, Widget,
};

use conrod_core::{WidgetCommon, WidgetStyle};

///Custom made widget for the FilterButton
#[derive(WidgetCommon)]
pub struct FilterButton<'a> {
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
/// Represents the unique styling for our widget.
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

    #[conrod(default = "position::Relative::Align(position::Align::Middle)")]
    pub label_x: Option<position::Relative>,
    /// The position of the title bar's `Label` widget over the *y* axis.
    #[conrod(default = "position::Relative::Align(position::Align::Middle)")]
    pub label_y: Option<position::Relative>,
}

widget_ids! {
    pub struct FilterWidgetIds{
        start_circle,
        rectangle,
        end_circle,
        label,
    }
}

///Declaration of the Filter State
pub struct FilterWidgetState {
    ids: FilterWidgetIds,
}

impl<'a> FilterButton<'a> {
    pub fn new() -> Self {
        FilterButton {
            common: widget::CommonBuilder::default(),
            maybe_label: None,
            style: Style::default(),
            enabled: true,
        }
    }

    #[allow(dead_code)]
    pub fn label_font_id(mut self, font_id: conrod_core::text::font::Id) -> Self {
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

    /// Specify the label's position relatively to `Button` along the *x* axis.
    pub fn label_x(mut self, x: position::Relative) -> Self {
        self.style.label_x = Some(x);
        self
    }

    /// Specify the label's position relatively to `Button` along the *y* axis.
    pub fn label_y(mut self, y: position::Relative) -> Self {
        self.style.label_y = Some(y);
        self
    }

    ///Enabled button
    #[allow(dead_code)]
    pub fn enabled(mut self, flag: bool) -> Self {
        self.enabled = flag;
        self
    }
}

impl<'a> Default for FilterButton<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Widget for FilterButton<'a> {
    type State = FilterWidgetState;
    type Style = Style;
    type Event = Option<()>;

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        FilterWidgetState {
            ids: FilterWidgetIds::new(id_gen),
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

        let FilterButton { maybe_label, .. } = self;

        let (button_color, event) = {
            let input = ui.widget_input(id);

            //If button was clicked
            let event = input.clicks().left().next().map(|_| ());

            let color = style.color(&ui.theme);
            let color = input.mouse().map_or(color, |mouse| {
                if mouse.buttons.left().is_down() {
                    conrod_core::color::DARK_GREY
                } else {
                    color.highlighted()
                }
            });

            (color, event)
        };

        let radius = rect.h() / 2.0;

        //Drawing the circles
        widget::Circle::fill(radius)
            .x(rect.x() - rect.w() / 2.0)
            .y(rect.y())
            .graphics_for(id)
            .color(button_color)
            .set(state.ids.start_circle, ui);

        widget::Circle::fill(radius)
            .x(rect.x() + rect.w() / 2.0)
            .y(rect.y())
            .graphics_for(id)
            .color(button_color)
            .set(state.ids.end_circle, ui);

        let rect_fill = [rect.w(); 2];
        widget::Rectangle::fill(rect_fill)
            .w_h(rect.w(), rect.h())
            .middle_of(id)
            .graphics_for(id)
            .color(button_color)
            .set(state.ids.rectangle, ui);

        //Makes the label
        if let Some(l) = maybe_label {
            label(id, state.ids.label, l, style, ui)
        }
        event
    }
}
impl<'a> Colorable for FilterButton<'a> {
    fn color(mut self, color: conrod_core::Color) -> Self {
        self.style.color = Some(color);
        self
    }
}

/// Provide the chainable label(), label_color(), and label_font_size()
/// configuration methods.
impl<'a> Labelable<'a> for FilterButton<'a> {
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
//Function to make the initiation of a label
fn label(button_id: widget::Id, label_id: widget::Id, label: &str, style: &Style, ui: &mut UiCell) {
    let color = style.label_color(&ui.theme);
    let font_size = style.label_font_size(&ui.theme);
    let x = style.label_x(&ui.theme);
    let y = style.label_y(&ui.theme);
    let justify = style.label_justify(&ui.theme);
    let font_id = style
        .label_font_id(&ui.theme)
        .or_else(|| ui.fonts.ids().next());
    widget::Text::new(label)
        .and_then(font_id, widget::Text::font_id)
        .x_position_relative_to(button_id, x)
        .y_position_relative_to(button_id, y)
        .justify(justify)
        .parent(button_id)
        .graphics_for(button_id)
        .color(color)
        .font_size(font_size)
        .set(label_id, ui);
}

pub fn draw(
    widget_id: widget::id::Id,
    ui: &mut UiCell,
    label: String,
    widget_x_position: f64,
    widget_y_position: f64,
) {
    if let Some(_clicks) = FilterButton::new()
        .x(widget_x_position)
        .y(widget_y_position)
        .w_h(150.0, 30.0)
        .label_font_size(10)
        .label_color(conrod_core::color::BLACK)
        .label(label.as_str())
        .set(widget_id, ui)
    {
        println!("{:?}", ui.xy_of(widget_id));
    }
}
