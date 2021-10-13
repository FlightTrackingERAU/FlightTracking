use conrod_core::{widget, widget_ids, Colorable, Labelable, Point, Positionable, Widget};

///This is how we implement widget
#[derive(WidgetCommon)]
pub struct Circular_Button<'a> {
    ///Does some rendering work in back
    ///It doesn't really matter to know
    #[conrod(common_builder)]
    common: widget::CommonBuilder,

    ///label for string
    label: Option<&'a str>,

    ///Style using the Style Structure
    style: Style,

    ///If botton is pressed or not
    enabled: bool,
}
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

widget_ids! {  //Widget of a circle with text, Later Circle with Image
    struct Ids{
        circle,
        text,
    }
}

///What state we going to use (Text or Image)
pub struct State {
    ids: Ids,
}

impl<'a> Circular_Button<'a> {
    ///Create blank button
    pub fn new() -> Self {
        Circular_Button {
            common: widget::CommonBuilder::default(),
            style: Style::default(),
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
impl<'a> Widget for Circular_Button<'a> {
    ///Stated defined before
    type State = State;
    ///Style defined before
    type Style = Style;

    ///Event when widget is initialized
    ///
    ///Some when clicked, None when its NOT clicked
    type Event = Option<()>;

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            ids: Ids::new(id_gen),
        }
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
            let unique = widget.state_and_style::<State, Style>().unwrap();
            unique.state.ids.circle.into()
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
            .set(state.ids.circle, ui);

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
                .set(state.ids.text, ui);
        }

        event
    }
}
///Chainable Color
impl<'a> Colorable for Circular_Button<'a> {
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

#[cfg(test)]
pub mod tests {
    use super::*;
}
