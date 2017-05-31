extern crate lerp;

use self::lerp::Lerp;

fn clamp(val: f32, min: f32, max: f32) -> f32 {
    if val < min {
        min
    } else if val > max {
        max
    } else {
        val
    }
}

pub fn red_weighted(rgb: &[u8], red_multiplier: f32) -> [u8; 3] {
    let average = ((rgb[0] / 3) + (rgb[1] / 3) + (rgb[2] / 3)) as f32;
    let weight = (rgb[0] as f32 - average) as f32 / 255f32;

    return [
        clamp(average.lerp(rgb[0] as f32, weight * red_multiplier), 0f32, 255f32) as u8,
        average.lerp(rgb[1] as f32, weight * 0.5f32) as u8,
        average.lerp(rgb[2] as f32, weight * 0.5f32) as u8
    ];
}

pub fn red_averages(rgb: &[u8]) -> [u8; 3] {
    let average = (rgb[0] / 3) + (rgb[1] / 3) + (rgb[2] / 3);
    return [rgb[0], average, average];
}
