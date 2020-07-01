use std::time::Instant;

use image::ImageBuffer;
use noise::{NoiseFn, OpenSimplex, ScaleBias, ScalePoint, Seedable};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // explicit type required
    let two: f64 = 2.0;

    let octave_1 = two.powf(5.0);
    let simplex_1 = OpenSimplex::new().set_seed(2345678);
    let scale_point_1 = ScalePoint::new(&simplex_1)
        .set_x_scale(1.0 / octave_1)
        .set_y_scale(1.0 / octave_1);
    let scale_bias_1 = ScaleBias::new(&scale_point_1).set_scale(255.0 / 2.0);
    // Simplex noise output is between -0.5 and 0.5
    // .set_bias((255.0 / 2.0) / 2.0);

    let octave_2 = two.powf(3.0);
    let simplex_2 = OpenSimplex::new().set_seed(1945668);
    let scale_point_2 = ScalePoint::new(&simplex_2)
        .set_x_scale(1.0 / octave_2)
        .set_y_scale(1.0 / octave_2);
    let scale_bias_2 = ScaleBias::new(&scale_point_2).set_scale(255.0 / 3.0);
    // .set_bias((255.0 / 3.0) / 2.0);

    let add_3 = noise::Add::new(&scale_bias_1, &scale_bias_2);
    // let scale_bias_3 = ScaleBias::new(&add_3).set_scale(255.0);

    let (width, height) = (128, 128);

    let start = Instant::now();
    let img = ImageBuffer::from_fn(width, height, |x, y| {
        // println!("{}", scale_bias_1.get([x as f64, y as f64]));
        // image::Rgb([(value_1 * 255.0) as u8, (value_2 * 255.0) as u8, 0_u8])
        let value = add_3.get([x as f64, y as f64]);
        image::Luma([value as u8])
    });
    println!("time elapsed: {}ms", start.elapsed().as_millis());

    img.save("./noise3.png")?;

    Ok(())
}
