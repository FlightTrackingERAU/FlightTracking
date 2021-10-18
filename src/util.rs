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

/// Takes a latitude in degrees and converts it to a world y coordinate using the mercator
/// projection.
pub fn y_from_latitude(lat_degrees: f64) -> f64 {
    //TODO: Make sure this is the correct function. The inverse looks the same as the main function?
    //https://www.desmos.com/calculator/qz3psqkddu
    use std::f64::consts::PI;

    let lat_rads = PI * lat_degrees / 180.0;

    map(PI, -PI, f64::atanh(f64::sin(lat_rads)), 0.0, 1.0)
}

/// Takes a y in world coordinates and converts it to latitude in degrees using the mercator
/// projection.
pub fn latitude_from_y(y: f64) -> f64 {
    let mercator_latitude_max = 85.05113;
    use std::f64::consts::FRAC_PI_2;

    let input = map(0.0, 1.0, y, -FRAC_PI_2, FRAC_PI_2);

    let _ = map(0.0, 1.0, 0.0, mercator_latitude_max, -mercator_latitude_max);
    let output = f64::asin(f64::tanh(input));
    let output_max = 1.1608753909688045;
    map(
        -output_max,
        output_max,
        output,
        mercator_latitude_max,
        -mercator_latitude_max,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ish(value: f64, expected: f64) {
        if (value - expected).abs() > 0.00001 {
            panic!("{} expected: {} out of range", expected, value);
        }
    }

    #[test]
    fn y_from_latitude_test() {
        ish(y_from_latitude(0.0), 0.5);
        ish(y_from_latitude(85.05113), 0.0);
        ish(y_from_latitude(-85.05113), 1.0);
    }

    #[test]
    fn latitude_from_y_test() {
        ish(latitude_from_y(0.5), 0.0);
        ish(latitude_from_y(0.0), 85.05113);
        ish(latitude_from_y(1.0), -85.05113);
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
