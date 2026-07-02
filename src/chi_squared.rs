/// Contains all accepted correlation probabilities for [chi_squared]
pub const CORRELATION_PROBS: [f32; 11] = [
    0.995, 0.975, 0.2, 0.1, 0.05, 0.025, 0.02, 0.01, 0.005, 0.002, 0.001,
];

/// Returns the value of chi^2 with 2 degrees of freedom. (x- and y-axis)
///
/// The mapping is copied from lecture 7, slide 20
///
/// # Accepted correlation probabilities
///
/// They can also be looked up in [CORRELATION_PROBS]
///
/// - 0.995
/// - 0.975
/// - 0.2
/// - 0.1
/// - 0.05
/// - 0.025
/// - 0.02
/// - 0.01
/// - 0.005
/// - 0.002
/// - 0.001
pub fn chi_squared(correlation_prob: f32) -> f32 {
    match correlation_prob {
        0.995 => 0.01,
        0.975 => 0.0506,
        0.2 => 3.219,
        0.1 => 4.605,
        0.05 => 5.991,
        0.025 => 7.378,
        0.02 => 7.824,
        0.01 => 9.21,
        0.005 => 10.597,
        0.002 => 12.429,
        0.001 => 13.816,
        _ => unimplemented!(),
    }
}
