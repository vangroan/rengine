/// Given a voxel coordinate, return three deterministic pseudo-random numbers
/// between 0.0 and 1.0.
///
/// Useful for random values that need to remain the same for a given voxel
/// position. Example use-case would be randomizing mesh or texture
/// regeneration.
pub fn wiggle(i: i32, j: i32, k: i32) -> [f32; 3] {
    // Cast to larger type so we have room to shift.
    let (i, j, k) = (i as u64, j as u64, k as u64);

    // This function is meant to work with map coordinates, the most
    // common coordinates are around the origin (0, 0, 0), making the
    // output also close to (0, 0, 0).
    //
    // Subtracting from a large number to increase range of output.
    let (i, j, k) = (
        ::std::u64::MAX - i,
        ::std::u64::MAX - j,
        ::std::u64::MAX - k,
    );

    // Combine all axes together, so output along a single
    // a single axes is still randomised.
    //
    // Otherwise a axis' output would remain the same, even
    // as the other coordinates change.
    let (i, j, k) = (
        (i << 30) ^ (j << 14) ^ k,
        (j << 31) ^ (k << 15) ^ i,
        (k << 32) ^ (i << 16) ^ j,
    );

    // The number 65,535 is chosen because it is small enough to offer good
    // modulo wrap-around, but also gives good spread between 0.0 and 1.0.
    [
        (xorshift(i) % 65_535) as f32 / 65_535_f32,
        (xorshift(j) % 65_535) as f32 / 65_535_f32,
        (xorshift(k) % 65_535) as f32 / 65_535_f32,
    ]
}

/// Simple Xor-Shift pseudo random number implementation.
fn xorshift(n: u64) -> u64 {
    let mut x = n;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    x
}
