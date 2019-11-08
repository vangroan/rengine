const PERMUTATION_LIST: [u8; 255] = [
    12, 208, 115, 24, 3, 23, 151, 244, 253, 75, 118, 17, 249, 18, 191, 179, 195, 148, 235, 92, 120,
    40, 103, 226, 15, 101, 209, 194, 218, 204, 109, 182, 143, 42, 147, 79, 163, 52, 90, 213, 185,
    176, 96, 78, 130, 181, 137, 72, 227, 104, 223, 43, 129, 217, 60, 63, 168, 155, 188, 189, 239,
    100, 237, 8, 134, 122, 175, 222, 33, 66, 162, 69, 86, 170, 41, 127, 153, 67, 112, 212, 54, 205,
    10, 236, 14, 225, 197, 45, 68, 31, 2, 207, 144, 156, 184, 242, 230, 196, 126, 198, 117, 252,
    57, 5, 119, 71, 46, 13, 74, 106, 84, 233, 7, 216, 128, 0, 97, 88, 29, 121, 21, 95, 232, 210, 6,
    44, 231, 4, 245, 34, 56, 221, 51, 248, 201, 238, 234, 73, 102, 203, 82, 250, 254, 25, 180, 174,
    158, 94, 87, 28, 27, 160, 70, 229, 111, 186, 65, 146, 36, 133, 219, 80, 149, 89, 61, 116, 200,
    224, 30, 241, 131, 199, 32, 108, 164, 240, 113, 35, 47, 169, 50, 178, 91, 123, 150, 62, 114, 9,
    165, 85, 83, 107, 183, 136, 177, 49, 211, 167, 132, 16, 139, 37, 105, 215, 59, 53, 220, 228,
    11, 172, 20, 214, 1, 247, 76, 251, 22, 193, 140, 135, 125, 161, 142, 93, 159, 81, 77, 99, 171,
    173, 110, 166, 141, 145, 152, 187, 19, 154, 138, 202, 48, 64, 243, 192, 98, 58, 157, 26, 124,
    38, 39, 206, 190, 246, 55,
];

/// Generates a sample of value noise at the given
/// three-dimensional position.
pub fn sample_value_noise(position: f32, octaves: u8) -> f32 {
    assert!(octaves > 0, "Octaves must be greater than 0");

    // Each octave is added to the sum output.
    let mut sum: f32 = 0.0;

    for oct in 0..octaves {
        let frequency = 2_f32.powf(f32::from(oct));
        let amplitude = 1.0 / frequency;

        // Points either side of the given position.
        let pos_a = (position.floor()) * frequency;
        let pos_b = (position.floor() + 1.0) * frequency;
        let pos = position * frequency;

        // Shifted by the iteration so that each octave
        // has distinct random values.
        let a = (pos_a as usize % PERMUTATION_LIST.len()) as f32 / 255.0;
        let b = (pos_b as usize % PERMUTATION_LIST.len()) as f32 / 255.0;

        // Higher frequency octaves have lower applitudes.
        let val_a = a * amplitude;
        let val_b = b * amplitude;

        // Linear interpolation
        // (b - a * t) + a
        let diff = val_b - val_a;
        let t = (pos - pos_a) / diff;
        sum += val_a + diff * t;
    }

    sum
}
