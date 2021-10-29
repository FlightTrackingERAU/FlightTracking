//! Various utilities for commonly needed things.
//!
//! Currently thin contains functions for interpolation, normalization, and mapping one range of
//! values to another

pub fn lerp<T, F>(a: T, b: T, f: F) -> T
where
    T: Copy,
    T: std::ops::Sub<Output = T>,
    T: std::ops::Add<Output = T>,
    T: std::ops::Mul<F, Output = T>,
{
    //Convert the 0-1 range into a value in the right range.
    a + ((b - a) * f)
}

pub fn normalize<T, F>(a: T, b: T, value: T) -> F
where
    T: Copy,
    T: std::ops::Sub<Output = T>,
    T: std::ops::Div<Output = F>,
{
    (value - a) / (b - a)
}

pub fn map<S, D, F>(left_min: S, left_max: S, value: S, right_min: D, right_max: D) -> D
where
    S: Copy,
    S: std::ops::Sub<Output = S>,
    S: std::ops::Div<Output = F>,
    D: Copy,
    D: std::ops::Sub<Output = D>,
    D: std::ops::Add<Output = D>,
    D: std::ops::Mul<F, Output = D>,
{
    //Figure out how 'wide' each range is
    let f: F = normalize(left_min, left_max, value);

    lerp(right_min, right_max, f)
}

pub fn round_up<T>(to_round: T, multiple: T) -> T
where
    T: num::Signed,
    T: From<i32>,
    T: std::ops::Sub,
    T: std::cmp::PartialOrd,
    T: Clone,
{
    let zero = T::from(0);
    if multiple == zero {
        return to_round;
    }

    let remainder = to_round.abs() % multiple.clone();
    if remainder == zero {
        return to_round;
    }

    if to_round < zero {
        -(to_round.abs() - remainder)
    } else {
        to_round + multiple - remainder
    }
}

pub fn round_up_pow2<T>(to_round: T) -> T
where
    T: num::traits::float::Float,
{
    //Find out log2 of `to_round` and round up, then use as exponent for 2 to get result
    T::powf(T::from(2).unwrap(), T::ceil(T::log2(to_round)))
}

/// Takes a latitude in degrees and converts it to a world y coordinate using the mercator
/// projection.
pub fn y_from_latitude(lat_degrees: f64) -> f64 {
    //Math visible at:
    //https://www.desmos.com/calculator/qz3psqkddu
    use std::f64::consts::PI;

    let lat_rads = PI * lat_degrees / 180.0;

    map(PI, -PI, f64::atanh(f64::sin(lat_rads)), 0.0, 1.0)
}

/// Takes a y in world coordinates and converts it to latitude in degrees using the mercator
/// projection.
pub fn latitude_from_y(y: f64) -> f64 {
    use std::f64::consts::PI;

    let output = f64::asin(f64::tanh(map(0.0, 1.0, y, PI, -PI)));
    output * 180.0 / PI
}

/// Takes a latitude in degrees and converts it to a world y coordinate using the mercator
/// projection.
pub fn x_from_longitude(longitude_degrees: f64) -> f64 {
    map(-180.0, 180.0, longitude_degrees, 0.0, 1.0)
}

/// Takes a x in world coordinates and converts it to longitude in degrees using the mercator
/// projection.
pub fn longitude_from_x(x: f64) -> f64 {
    map(0.0, 1.0, x, -180.0, 180.0)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn ish(value: f64, expected: f64) {
        if (value - expected).abs() > 0.00001 {
            panic!("Expected: {}, {} is of range", expected, value);
        }
    }

    fn ish_bounded(value: f64, expected: f64, bound: f64) {
        if (value - expected).abs() > bound {
            panic!("Expected: {}, {} is of range", expected, value);
        }
    }

    #[test]
    fn y_from_latitude_test() {
        ish(y_from_latitude(0.0), 0.5);
        ish(y_from_latitude(85.05113), 0.0);
        ish(y_from_latitude(-85.05113), 1.0);

        //66.5 is the latitude at the top of iceland, between the top two tiles at zoom level 2
        ish_bounded(y_from_latitude(66.5), 0.25, 0.05);
    }

    #[test]
    fn latitude_from_y_test() {
        ish(latitude_from_y(0.5), 0.0);
        ish(latitude_from_y(0.0), 85.05113);
        ish(latitude_from_y(1.0), -85.05113);

        ish_bounded(latitude_from_y(0.25), 66.5, 0.05);
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

/*
pub fn map_clamp<S, D, F, C>(
    left_min: S,
    left_max: S,
    value: S,
    right_min: D,
    right_max: D,
    clamp_fn: C,
) -> D
where
    S: Copy,
    S: std::ops::Sub<Output = S>,
    S: std::ops::Div<Output = F>,
    D: Copy,
    D: std::ops::Sub<Output = D>,
    D: std::ops::Add<Output = D>,
    D: std::ops::Mul<F, Output = D>,
    C: Fn(&D, &D, &D) -> D,
{
    let un_clamped: D = map(left_min, left_max, value, right_min, right_max);
    clamp_fn(&right_max, &right_max, &un_clamped)
}
*/
