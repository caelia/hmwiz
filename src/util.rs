pub(crate) fn wmean((m, v): (f32, f32), (n, w): (f32, f32)) -> f32 {
    ((m * v) + (n * w)) / (v + w)
}

pub(crate) fn distance(c1: usize, r1: usize, c2: usize, r2: usize) -> usize {
    if r1 == r2 {
        usize::max(c1, c2) - usize::min(c1, c2)
    } else if c1 == c2 {
        usize::max(r1, r2) - usize::min(r1, r2)
    } else {
        panic!("Invalid arguments for distance(): c1: {c1}, r1: {r1}, c2: {c2}, r2: {r2}");
    }
}

pub(crate) fn gravitate(slope: f32, gravity: f32, max_slope: f32) -> f32 {
    assert!(gravity <= 1.0 && gravity >= 0.0);
    let delta = gravity * slope.abs();
    let adj_slope = slope - delta;
    if adj_slope < 0. {
        f32::max(adj_slope, -max_slope)
    } else {
        f32::min(adj_slope, max_slope)
    }
}
