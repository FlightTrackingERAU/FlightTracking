use conrod_core::{widget, widget_ids, Colorable, Labelable, Point, Positionable, Widget};

///This is how we implement widget
#[derive(WidgetCommon)]
pub struct Circular_Button<'a, S> {
    ///Does some rendering work in back
    ///It doesn't really matter to know
    #[conrod(common_builder)]
    common: widget::CommonBuilder,

    ///label for string
    label: Option<&'a str>,

    show: S,

    ///Style using the Style Structure
    style: Style,

    ///If botton is pressed or not
    enabled: bool,
}
///Circular Button with an Image

///Unique style for a Circular Botton
#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle)]
pub struct Style {
    ///Color of button
    #[conrod(default = "theme.shape_color")]
    pub color: Option<conrod_core::Color>,

    ///Color of label
    #[conrod(default = "theme.label_color")]
    pub label_color: Option<conrod_core::Color>,

    #[conrod(default = "theme.font_size_medium")]
    pub label_font_size: Option<conrod_core::FontSize>,

    /// Specify a unique font for the label.
    #[conrod(default = "theme.font_id")]
    pub label_font_id: Option<Option<conrod_core::text::font::Id>>,
}

#[derive(Copy, Clone, Default, PartialEq, Debug)]
pub struct Flat {
    /// Allows specifying a color to use when the mouse hovers over the button.
    ///
    /// By default, this is `color.highlighted()` where `color` is the button's regular color.
    pub hover_color: Option<conrod_core::Color>,
    /// Allows specifying a color to use when the mouse presses the button.
    ///
    /// By default, this is `color.clicked()` where `color` is the button's regular color.
    pub press_color: Option<conrod_core::Color>,
}
widget_ids! {
    /// Identifiers for a "flat" button.
    pub struct FlatIds {
        circle,
        label,
    }
}

///Identifier for Image
widget_ids! {  //Widget of a circle with text, Later Circle with Image
    struct ImageIds{
        circle,
        image,
    }
}

impl<'a> Circular_Button<'a, Flat> {
    /// Begin building a flat-colored `Button` widget.
    pub fn new_flat() -> Self {
        Self::new(Flat::default())
    }

    /// Override the default button style
    pub fn with_style(mut self, s: Style) -> Self {
        self.style = s;
        self
    }

    /// Specify a color to use when the mouse hovers over the button.
    ///
    /// By default, this is `color.highlighted()` where `color` is the button's regular color.
    pub fn hover_color(mut self, color: conrod_core::Color) -> Self {
        self.show.hover_color = Some(color);
        self
    }

    /// Specify a color to use when the mouse presses the button.
    ///
    /// By default, this is `color.clicked()` where `color` is the button's regular color.
    pub fn press_color(mut self, color: conrod_core::Color) -> Self {
        self.show.press_color = Some(color);
        self
    }
}

#[derive(Copy, Clone)]
pub struct Image {
    ///Image Id
    pub image_id: conrod_core::image::Id,

    ///Image displayed when mouse hovers button
    pub hover_image_id: Option<conrod_core::image::Id>,

    ///Image displayed when image is pressed
    pub press_image_id: Option<conrod_core::image::Id>,

    pub color: widget::button::ImageColor,

    pub src_rect: Option<conrod_core::position::Rect>,
}

///What state we going to use (Text or Image)

impl<'a> Circular_Button<'a, Image> {
    ///Build button with Image
    pub fn image(image_id: conrod_core::image::Id) -> Self {
        let image = Image {
            image_id,
            hover_image_id: None,
            press_image_id: None,
            src_rect: None,
            color: widget::button::ImageColor::None,
        };
        Self::new(image)
    }

    ///The rectangular are of the image wish to displayed
    ///
    ///by default it would use the normal size of image
    pub fn source_rectangle(mut self, rect: conrod_core::position::Rect) -> Self {
        self.show.src_rect = Some(rect);
        self
    }

    pub fn image_color(mut self, color: conrod_core::color::Color) -> Self {
        self.show.color = conrod_core::widget::button::ImageColor::Normal(color);
        self
    }

    ///Color will change slightly when button is clicked or highlated
    pub fn image_color_with_feedback(mut self, color: conrod_core::color::Color) -> Self {
        self.show.color = conrod_core::widget::button::ImageColor::WithFeedback(color);
        self
    }

    pub fn hover_image(mut self, id: conrod_core::image::Id) -> Self {
        self.show.hover_image_id = Some(id);
        self
    }

    pub fn press_image(mut self, id: conrod_core::image::Id) -> Self {
        self.show.press_image_id = Some(id);
        self
    }
}

impl<'a, S> Circular_Button<'a, S> {
    ///Create blank button
    pub fn new(show: S) -> Self {
        Circular_Button {
            common: widget::CommonBuilder::default(),
            style: Style::default(),
            show,
            label: None,
            enabled: true,
        }
    }

    ///Choose type of Font to use
    pub fn label_font_id(mut self, font_type: conrod_core::text::font::Id) -> Self {
        self.style.label_font_id = Some(Some(font_type));
        self
    }

    #[allow(dead_code)]
    pub fn enabled(mut self, condition: bool) -> Self {
        self.enabled = condition;
        self
    }
}

///Now implement Widget for Own Made Buttons
///Conrod_core has more explanation in their website
impl<'a> Widget for Circular_Button<'a, Flat> {
    ///Stated defined before
    type State = FlatIds;
    ///Style defined before
    type Style = Style;

    ///Event when widget is initialized
    ///
    ///Some when clicked, None when its NOT clicked
    type Event = Option<()>;

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        FlatIds::new(id_gen)
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    ///This function determines if there is a point over this widget
    ///Or if any other widgets can be used to represent this widget

    fn is_over(&self) -> widget::IsOverFn {
        use conrod_core::graph::Container;
        use conrod_core::Theme;

        fn is_over_widget(widget: &Container, _: Point, _: &Theme) -> widget::IsOver {
            let unique = widget.state_and_style::<FlatIds, Style>().unwrap();
            unique.state.circle.into()
        }

        is_over_widget
    }
    /// Update the state of the button by handling any input that has occurred since the last
    /// update. Like height, radius, anything can change
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

        let radius = rect.w() / 2.0;
        widget::Circle::fill(radius)
            .middle_of(id)
            .graphics_for(id)
            .color(color)
            .set(state.circle, ui);

        // Now we'll instantiate our label using the **Text** widget.
        if let Some(ref label) = self.label {
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

///Widget for Image
impl<'a> Widget for Circular_Button<'a, Image> {
    type State = ImageIds;
    type Style = Style;
    type Event = Option<()>;

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        ImageIds::new(id_gen)
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs {
            id,
            state,
            style,
            rect,
            ui,
            ..
        } = args;

        let Circular_Button {
            common,
            label,
            show,
            style,
            enabled,
        } = self;

        let Image {
            image_id,
            hover_image_id,
            press_image_id,
            color,
            src_rect,
        } = show;
    }
}
///Chainable Color
impl<'a, S> Colorable for Circular_Button<'a, S> {
    fn color(mut self, color: conrod_core::Color) -> Self {
        self.style.color = Some(color);
        self
    }
}
///Chainable label
impl<'a> Labelable<'a> for Circular_Button<'a> {
    fn label(mut self, text: &'a str) -> Self {
        self.label = Some(text);
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

fn interaction_and_times_triggered(
    button_id: widget::Id,
    ui: &conrod_core::ui::UiCell,
) -> (Interaction, u16) {
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

#[cfg(test)]
pub mod tests {
    use super::*;
}
