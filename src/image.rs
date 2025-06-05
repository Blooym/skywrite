use std::io::Cursor;

use image::{GenericImageView, ImageFormat, imageops::FilterType};

pub fn resize_to_aspect_ratio(
    image_bytes: &[u8],
    target_aspect: f32,
    format: ImageFormat,
    filter_type: FilterType,
) -> anyhow::Result<Vec<u8>> {
    let img = image::load_from_memory(image_bytes)?;
    let (width, height) = img.dimensions();
    let current_aspect = width as f32 / height as f32;

    // Convert to target ratio.
    let (new_width, new_height) = if current_aspect > target_aspect {
        ((height as f32 * target_aspect) as u32, height)
    } else if current_aspect < target_aspect {
        (width, (width as f32 / target_aspect) as u32)
    } else {
        (width, height)
    };

    let resized_img = img.resize_exact(new_width, new_height, filter_type);
    let mut output_bytes = Vec::new();
    resized_img.write_to(&mut Cursor::new(&mut output_bytes), format)?;

    Ok(output_bytes)
}
