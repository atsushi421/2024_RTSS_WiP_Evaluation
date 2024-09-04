pub fn approx_eq(a: f64, b: f64) -> bool {
    const EPSILON: f64 = 1e-6;
    (a - b).abs() < EPSILON
}
