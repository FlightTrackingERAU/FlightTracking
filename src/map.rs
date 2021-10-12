use glam::DVec2;
use itertools::Itertools;
use std::convert::TryInto;
use std::ops::Range;

/// Representation of tile zoom levels.
/// Unsigned value that indicated exponential zoom.
/// 0 = Whole world is visible
pub type TileZoomLevel = u32;

/// The coordinates of a 2d map tile with the mercator projection
pub type TileCoordinate = (u32, u32);

pub struct TileView {
    /// The center of the view in degrees of longitude, and latitude
    ///
    /// Note: This format is [x (longitude), y (latitude)] NOT [latitude, longitude]
    center: DVec2,

    /// The size of each pixel in terms of degrees of latitude
    ///
    /// We use this format instead of lets say storing the coordinates of opposite corners so that
    /// the window can be resized and the center will stay in the center, and the zoom level will
    /// remain the same
    px_to_longitude: f64,
}

impl TileView {
    pub fn new(latitude: f64, longitude: f64, zoom: f64, window_width: u32) -> Self {
        Self {
            center: DVec2::new(longitude, latitude),
            px_to_longitude: px_to_longitude_from_zoom(zoom, window_width),
        }
    }

    /// Returns what zoom is visible based on the size of a tile.
    ///
    /// The zoom level is always rounded up so that pixels on a tile are always smaller physical pixels
    /// (no low quality interpolation needed)
    pub fn tile_zoom_level(&self, tile_size: u32) -> TileZoomLevel {
        //px = (360 / 2^zoom) / tile_size -> from px_to_latitude_from_zoom
        //px * tile_size = 360/ 2^zoom
        //px * tile_size * 2^zoom = 360
        //2^zoom = 360 / (px * tile_size)
        //zoom = log_2(360 / px * tile_size)

        let zoom = f64::log2(360f64 / (self.px_to_longitude * tile_size as f64)).ceil();
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
        self.px_to_longitude = px_to_longitude_from_zoom(zoom, window_width);
    }

    pub fn tile_iter(
        &self,
        tile_size: u32,
        screen_width: u32,
        screen_height: u32,
    ) -> TileViewIterator {
        let tile_zoom = self.tile_zoom_level(tile_size);
        let tile_size = tile_size as f64;
        let max_tile = 2u32.pow(tile_zoom);

        let tiles_wide = (screen_width as f64 / tile_size as f64).ceil() as u32;
        let tiles_high = (screen_height as f64 / tile_size as f64).ceil() as u32;

        // Each pixel is twice as wide as it is tall because of the mercator projection spanning
        // 360 degrees wide but only 180 degrees tall
        let px_to_latitude = self.px_to_longitude / 2.0;
        let px_to_longitude = self.px_to_longitude;
        //TODO: Use real trig below to make up for the * 2 above to make lines of latitude near the
        //poles map correctly

        //Compute the number of degrees (longitude, latitude) that spans the distance from the top
        //left of the screen to the certes of the screen
        let half_screen_size = DVec2::new(
            screen_width as f64 * px_to_longitude,
            screen_height as f64 * px_to_latitude,
        ) / 2.0;

        //Calculate where the top left and bottom right of our viewport is units of degrees
        let adjusted_half_screen_size = DVec2::new(half_screen_size.x, -half_screen_size.y);
        let mut top_left = self.center - adjusted_half_screen_size;
        let bottom_right = self.center + adjusted_half_screen_size;

        if top_left.x < -180.0 {
            top_left.x += 360.0;
        }

        let min = DVec2::new(-180.0, 90.0);
        let max = DVec2::new(180.0, -90.0);

        let dest_min = DVec2::new(0.0, 0.0);
        let dest_max = DVec2::new(max_tile as f64, max_tile as f64);

        //Next map the degree coordinates to tile coordinates (0..max_tile)
        let top_left_tiles = crate::util::map(min, max, top_left, dest_min, dest_max);
        let bottom_right_tiles = crate::util::map(min, max, bottom_right, dest_min, dest_max);

        //Floor and ceil to render all tiles that are even partially visible
        let first_x = top_left_tiles.x.floor() as u32;
        let first_y = top_left_tiles.y.floor() as u32;

        let num_tiles_x = (bottom_right_tiles.x.ceil() - top_left_tiles.x.floor()) as u32;

        if num_tiles_x != 0 {
            assert_eq!(num_tiles_x, tiles_wide);
        } 

        //We have all the values to make the iterator
        TileViewIterator {
            product: (first_x..(first_x + tiles_wide))
                .cartesian_product(first_y..first_y + tiles_high),
            max_tile,
        }
    }
}

pub fn modulo_floor(val: f64, modulo: f64) -> f64 {
    return val - (val.rem_euclid(modulo));
}

pub fn modulo_ceil(val: f64, modulo: f64) -> f64 {
    if val % modulo == 0.0 {
        val
    } else {
        val + modulo - val.rem_euclid(modulo)
    }
}

/// Converts a zoom level and the current window size to a `px_to_latitude` value.
fn px_to_longitude_from_zoom(zoom: f64, window_width: u32) -> f64 {
    //Use zoom to calculate how wide the window is in terms of degrees of longitude
    let latitude_width: f64 = 360f64 / 2f64.powf(zoom);

    // Divide by the number of pixels to get the number of degrees per pixel
    latitude_width / window_width as f64
}

/// Walks the positions of all the tiles currently in view, returning their coordinates for
/// rendering
pub struct TileViewIterator {
    product: itertools::Product<Range<u32>, Range<u32>>,
    max_tile: u32,
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
        let expected: Vec<TileCoordinate> = TileViewIterator { product, max_tile }.collect();
        assert_eq!(real, expected);
    }

    #[test]
    fn px_to_latitude_from_zoom_test() {
        //Zoom level 0 is the entire world and if we have one pixel then width then each pixel should
        //be the entire world
        assert_eq!(px_to_longitude_from_zoom(0.0, 1), 360.0);

        // At zoom=1 half of the world is visible horizontally,
        // and if we have 10 pixels then each pixel should be 18 degrees
        assert_eq!(px_to_longitude_from_zoom(1.0, 10), 18.0);
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
