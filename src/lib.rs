use conrod_core::{self, text, widget, Color, FontSize, Scalar};
use conrod_core::{WidgetCommon, WidgetStyle};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

///Empty lib for now
pub struct RandomStruct;

///Custom implementation of a circular button widget
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

///Luke and Troy's part of document
///
///
///
#[derive(Debug, Copy, Clone)]
pub struct TileId {
    pub x: u32,
    pub y: u32,
    pub zoom: u32,
}
pub struct Tile {
    pub id: TileId,
    pub image: image::RgbaImage,
}

#[derive(Debug, Copy, Clone)]
enum CachedTile {
    Pending,
    Cached(conrod_core::image::Id),
}

pub struct TileCache {
    tile_requester: TileRequester,
    hashmaps: Vec<HashMap<(u32, u32), CachedTile>>,
}

pub struct TileRequester {
    tile_rx: UnboundedReceiver<Tile>,
    request_tx: UnboundedSender<TileId>,
    tile_size: Arc<Mutex<Option<u32>>>,
}
