//! Various utilities for commonly needed things.
//!
//! Currently thin contains functions for interpolation, normalization, and mapping one rang of
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

pub fn map_clamp<S, D, F, C>(left_min: S, left_max: S, value: S, right_min: D, right_max: D, clamp_fn: C) -> D
where
    S: Copy,
    S: std::ops::Sub<Output = S>,
    S: std::ops::Div<Output = F>,
    D: Copy,
    D: std::ops::Sub<Output = D>,
    D: std::ops::Add<Output = D>,
    D: std::ops::Mul<F, Output = D>,
    C: Fn(&D, &D, &D) -> D
{
    let un_clamped: D = map(left_min, left_max, value, right_min, right_max);
    clamp_fn(&right_max, &right_max, &un_clamped)
}
