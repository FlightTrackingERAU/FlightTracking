use glam::{DVec2, IVec2};
use itertools::Itertools;
use std::convert::TryInto;
use std::ops::Range;

/// Representation of tile zoom levels.
/// Unsigned value that indicated exponential zoom.
/// 0 = Whole world is visible
pub type TileZoomLevel = u32;

pub type TileCoordinate = (u32, u32);

pub struct TileView {
    /// The center of the view [0..1] for both x and y
    ///
    /// (0, 0) is the north pole near alaska and (1, 1) is the south pole left of the anti meridian.
    /// Using this system, the coordinates map neatly onto tile coordinates that use a spherical
    /// mercator projection
    center: DVec2,

    /// The size of each pixel in terms of the world units used by `center`
    ///
    /// A size of one means that each pixel contains all the tiles in the whole world.
    /// We use this format instead of lets say storing the coordinates of opposite corners so that
    /// the window can be resized and the center will stay in the center, and the zoom level will
    /// remain the same
    pixel_size: f64,
}

impl TileView {
    pub fn new(latitude: f64, longitude: f64, zoom: f64, window_width: u32) -> Self {
        let x = crate::util::map(-180.0, 180.0, longitude, 0.0, 1.0);
        //TODO: Convert latitude properly, accounting for mercator stretching near the poles
        let y = crate::util::map(90.0, -90.0, latitude, 0.0, 1.0);
        println!("lng: {}, lat {}, to [{}, {}]", longitude, latitude, x, y);
        Self {
            center: DVec2::new(x, y),
            pixel_size: pixel_size_from_zoom(zoom, window_width),
        }
    }

    /// Returns what zoom is visible based on the size of a tile.
    ///
    /// The zoom level is always rounded up so that pixels on a tile are always smaller physical pixels
    /// (no low quality interpolation needed)
    pub fn tile_zoom_level(&self, tile_size: u32) -> TileZoomLevel {
        //px = (1 / 2^zoom) / tile_size -> from pixel_size_from_zoom
        //px * tile_size = 1 / 2^zoom
        //px * tile_size * 2^zoom = 1
        //2^zoom = 1 / (px * tile_size)
        //zoom = log_2(1 / px * tile_size)

        let zoom = f64::log2(1f64 / (self.pixel_size * tile_size as f64)).ceil();
        //Convert to i64 first so that we can use try from here
        //Somehow there is no impl TryInto<i64> for f64 or TryInto<u32> for f64
        (zoom as i64)
            .try_into()
            .expect("Zoom level too large for u32")
    }

    /// Sets the `zoom` for the entire tile viewport based on the current `window_width`.
    /// The value returned by [`tile_zoom_level`] will always at least as big as `zoom` for a
    /// window larger then the tile size, because more tiles are needed to span the entire window
    pub fn set_zoom(&mut self, zoom: f64, window_width: u32) {
        self.pixel_size = pixel_size_from_zoom(zoom, window_width);
    }

    pub fn tile_iter(
        &self,
        tile_size: u32,
        screen_width: u32,
        screen_height: u32,
    ) -> TileViewIterator {
        let tile_zoom = self.tile_zoom_level(tile_size);
        let max_tile = 2u32.pow(tile_zoom);

        //Tile size is the size of a tile in pixels based on the current zoom level
        //We know how large each pixel should be in world coordinates, and how big the tile should
        //be in world coordinates. Use one to calculate the other

        //Units are world units (aka 1/(tile units))
        let tile_length = 1.0 / max_tile as f64;
        let tile_size_world = DVec2::new(tile_length, tile_length);

        //`self.pixel_size` units are (world/pixel), so inv is (pixel/world)
        let inv_pixel_size = DVec2::new(1.0, 1.0) / self.pixel_size;

        //If we multiply tile_size_world with inv_pixel_size the units are:
        //(pixel/world) * (world/1) -> pixel
        let tile_size = tile_size_world * inv_pixel_size;

        //Compute the size of half the screen in terms of world coordinates
        let half_screen_size = DVec2::new(
            screen_width as f64 * self.pixel_size,
            screen_height as f64 * self.pixel_size,
        ) / 2.0;

        //Calculate where the top left and bottom right of our viewport is world coordinates
        let adjusted_half_screen_size = DVec2::new(half_screen_size.x, half_screen_size.y);
        let mut top_left = self.center - adjusted_half_screen_size;
        let mut bottom_right = self.center + adjusted_half_screen_size;

        top_left.x = top_left.x.rem_euclid(1.0);
        top_left.y = top_left.y.rem_euclid(1.0);

        bottom_right.x = bottom_right.x.rem_euclid(1.0);
        bottom_right.y = bottom_right.y.rem_euclid(1.0);

        println!("top {}, bottom {}", top_left, bottom_right);

        let dest_max = DVec2::new(max_tile as f64, max_tile as f64);

        //Next map world coordinates to tile coordinates (0..1) to (0..max_tile)
        let top_left_tiles = top_left * dest_max;
        let bottom_right_tiles = bottom_right * dest_max;

        //Floor and ceil to render all tiles that are even partially visible
        let first_offset = top_left_tiles % DVec2::new(1.0, 1.0);

        let first_x = (top_left_tiles.x - first_offset.x) as u32;
        let first_y = (top_left_tiles.y - first_offset.y) as u32;
        println!("First offset: {}, size {}", first_offset, tile_size);

        let (tiles_wide, tiles_high) = {
            if top_left.x < bottom_right.x {
                let diff = bottom_right_tiles - top_left_tiles;
                (diff.x.ceil() as u32 + 1, diff.y.ceil() as u32 + 2)
            } else {
                panic!("Wraparound x not implemented");
            }
        };

        //We have all the values to make the iterator
        TileViewIterator {
            product: (first_x..(first_x + tiles_wide))
                .cartesian_product(first_y..first_y + tiles_high),
            max_tile,
            tile_offset: DVec2::new(-first_offset.x, first_offset.y) * tile_size,
            tile_size,
            tiles_horizontally: tiles_wide,
            tiles_vertically: tiles_high,
        }
    }
}

/// Rounds a number down to the nearest multiple of `modulo`
pub fn modulo_floor(val: f64, modulo: f64) -> f64 {
    val - (val.rem_euclid(modulo))
}

/// Rounds a number up to the nearest multiple of `modulo`
pub fn modulo_ceil(val: f64, modulo: f64) -> f64 {
    if val % modulo == 0.0 {
        val
    } else {
        val + modulo - val.rem_euclid(modulo)
    }
}

/// Converts a zoom level and the current window size to a `pixel_size` value.
fn pixel_size_from_zoom(zoom: f64, window_width: u32) -> f64 {
    //Use zoom to calculate how wide the window is in world units (zoom level 0 = whole world)
    let window_size: f64 = 1.0 / 2f64.powf(zoom);

    // Divide by the number of pixels to get the number of degrees per pixel
    window_size / window_width as f64
}

/// Walks the positions of all the tiles currently in view, returning their coordinates for
/// rendering
pub struct TileViewIterator {
    product: itertools::Product<Range<u32>, Range<u32>>,
    max_tile: u32,

    /// The pixel offset between the top left corner of the viewpoint and the top left corner of the
    /// topmost, leftmost tile
    pub tile_offset: DVec2,

    /// The size of a tile in pixels based on the current zoom
    pub tile_size: DVec2,

    /// The number of tiles to render horizontally
    pub tiles_horizontally: u32,

    /// The number of tiles to render vertically
    pub tiles_vertically: u32,
}

impl Iterator for TileViewIterator {
    type Item = TileCoordinate;

    fn next(&mut self) -> Option<Self::Item> {
        match self.product.next() {
            Some(next) => Some((next.0 % self.max_tile, next.1 % self.max_tile)),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct IsSameTiles {
        view: TileView,
        tile_size: u32,
        screen_width: u32,
        screen_height: u32,
        x_start: u32,
        x_len: u32,
        y_start: u32,
        y_len: u32,
    }

    fn is_same_tiles(data: IsSameTiles) {
        let real_iter = data
            .view
            .tile_iter(data.tile_size, data.screen_width, data.screen_height);

        let real: Vec<TileCoordinate> = real_iter.collect();
        let max_tile = 2u32.pow(data.view.tile_zoom_level(data.tile_size));

        let a = data.x_start..(data.x_start + data.x_len);
        let b = data.y_start..(data.y_start + data.y_len);

        let product = a.cartesian_product(b);
        let expected: Vec<TileCoordinate> = TileViewIterator {
            product,
            max_tile,
            tile_offset: DVec2::new(0.0, 0.0),
            tile_size: DVec2::new(0.0, 0.0),
            tiles_horizontally: data.x_len,
            tiles_vertically: data.y_start,
        }
        .collect();
        assert_eq!(real, expected);
    }

    #[test]
    fn pixel_size_from_zoom_test() {
        //Zoom level 0 is the entire world and if we have one pixel then width then each pixel should
        //be the entire world
        assert_eq!(pixel_size_from_zoom(0.0, 1), 1.0);

        // At zoom=1 half of the world is visible horizontally,
        // and if we have 10 pixels then each pixel should be 1/2 * 1/10 == 0.05
        assert_eq!(pixel_size_from_zoom(1.0, 10), 0.05);
    }

    #[test]
    fn tile_view() {
        let mut view = TileView::new(0.0, 0.0, 0.0, 1000);
        //Center the world and use zoom level 0 - the whole world is visible.
        //With a screen 1000 pixels wide, we need at least 1000 pixels wide of tiles to look nice.
        //Because our virtual tiles are 256x256, then we need 4 of them to to fill the screen (1024
        //pixels). Therefore use zoom level 2 which includes 2^2 = 4 tiles
        assert_eq!(view.tile_zoom_level(256), 2);

        assert_eq!(view.tile_zoom_level(512), 1);

        //If we are using zoom level 0.5 and the window is 500 pixels wide we need 2 tiles for the
        //entire window, and the entire window is at zoom level 2, meaning each tile should be at
        //zoom level 3
        view.set_zoom(2.0, 500);
        assert_eq!(view.tile_zoom_level(256), 3);
        assert_eq!(view.tile_zoom_level(128), 4);
    }

    #[test]
    fn tile_view_it() {
        let mut it = TileViewIterator {
            product: (0..2).cartesian_product(0..2),
            max_tile: 2,
            tile_offset: DVec2::new(0.0, 0.0),
            tile_size: DVec2::new(0.0, 0.0),
            tiles_horizontally: 0,
            tiles_vertically: 0,
        };
        assert_eq!(it.next(), Some((0, 0)));
        assert_eq!(it.next(), Some((0, 1)));
        assert_eq!(it.next(), Some((1, 0)));
        assert_eq!(it.next(), Some((1, 1)));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn tile_it_1() {
        let screen_width = 500;
        let screen_height = 500;
        //Use a tiny bit of zoom to force zoom level 2 to be chosen
        let view = TileView::new(0.0, 0.0, 0.001, screen_width);
        is_same_tiles(IsSameTiles {
            view,
            tile_size: 256,
            screen_width,
            screen_height,
            x_start: 0,
            x_len: 2,
            y_start: 0,
            y_len: 2,
        });
    }

    #[test]
    fn tile_it_2() {
        let screen_width = 500;
        let screen_height = 500;

        //Use a tiny bit of zoom to force zoom level 2 to be chosen for each tile
        let view = TileView::new(0.0, 0.0, 1.01, screen_width);
        is_same_tiles(IsSameTiles {
            view,
            tile_size: 256,
            screen_width,
            screen_height,
            x_start: 1,
            x_len: 2,
            y_start: 1,
            y_len: 2,
        });
    }

    #[test]
    fn tile_it_3() {
        let screen_width = 750;
        let screen_height = 500;

        //Use a tiny bit of zoom to force zoom level 2 to be chosen for each tile
        let view = TileView::new(83.0, -178.0, 4.001, screen_width);
        is_same_tiles(IsSameTiles {
            view,
            tile_size: 256,
            screen_width,
            screen_height,
            x_start: 62,
            x_len: 3,
            y_start: 1,
            y_len: 2,
        });
    }

    #[test]
    fn test_modulo_floor() {
        assert_eq!(modulo_floor(4.5, 2.0), 4.0);
        assert_eq!(modulo_floor(55.0, 10.0), 50.0);
        assert_eq!(modulo_floor(4.5, 2.0), 4.0);
        assert_eq!(modulo_floor(-4.5, 2.0), -6.0);
    }

    #[test]
    fn test_modulo_ceil() {
        assert_eq!(modulo_ceil(4.5, 2.0), 6.0);
        assert_eq!(modulo_ceil(55.0, 10.0), 60.0);
        assert_eq!(modulo_ceil(4.5, 1.5), 4.5);
    }
}
