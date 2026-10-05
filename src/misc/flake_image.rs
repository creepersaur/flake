use crate::prelude::{Color, FilterType};
use image::{DynamicImage, ImageBuffer, Rgba, RgbaImage};

/// # Image
///
/// Contains pixel data for a `width * height` image in RGBA format.
/// Where data is a Vec of `4 * w * h` float r, g, b, a values.
#[derive(Clone, Default, Debug)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub data: Vec<f32>,
    pub filter_type: FilterType,
}

impl Image {
    /// Creates a new black/transparent image filled with zeros (RGBA 32-bit float).
    pub fn new(width: u32, height: u32, filter_type: FilterType) -> Self {
        Self {
            width,
            height,
            data: vec![0.0; 4 * width as usize * height as usize],
            filter_type,
        }
    }

    /// Wraps an existing Vec<f32> RGBA buffer.
    pub fn from_data(width: u32, height: u32, filter_type: FilterType, data: Vec<f32>) -> Self {
        debug_assert_eq!(
            data.len(),
            4 * width as usize * height as usize,
            "Data buffer size does not match width * height * 4"
        );
        Self {
            width,
            height,
            data,
            filter_type,
        }
    }

    /// Returns image with filter type
    pub fn with_filter_type(mut self, filter_type: FilterType) -> Self {
        self.filter_type = filter_type;
        self
    }

    /// Calculates the starting 1D array index for pixel (x, y).
    #[inline]
    fn pixel_index(&self, x: u32, y: u32) -> usize {
        ((y * self.width + x) * 4) as usize
    }

    /// Sets the RGBA color [r, g, b, a] for pixel at (x, y).
    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x < self.width && y < self.height {
            let idx = self.pixel_index(x, y);
            self.data[idx..idx + 4].copy_from_slice(&color.to_array());
        }
    }

    /// Gets the RGBA color [r, g, b, a] at (x, y). Returns None if out of bounds.
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<Color> {
        if x < self.width && y < self.height {
            let idx = self.pixel_index(x, y);
            Some(Color::new(
                self.data[idx],
                self.data[idx + 1],
                self.data[idx + 2],
                self.data[idx + 3],
            ))
        } else {
            None
        }
    }

    /// Clears the image using a single RGBA color.
    pub fn clear(&mut self, color: [f32; 4]) {
        for chunk in self.data.chunks_exact_mut(4) {
            chunk.copy_from_slice(&color);
        }
    }

    /// Converts this struct directly into `image::DynamicImage` (as `ImageRgba32F`).
    pub fn to_dynamic(&self) -> DynamicImage {
        let buffer = ImageBuffer::<Rgba<f32>, Vec<f32>>::from_raw(
            self.width,
            self.height,
            self.data.clone(),
        )
        .expect("Failed to create ImageBuffer from Image data");

        DynamicImage::ImageRgba32F(buffer)
    }

    /// Creates an `Image` from an `image::DynamicImage` (normalizes u8/u16 channels to f32 [0.0, 1.0]).
    pub fn from_dynamic(dynamic: &DynamicImage, filter_type: FilterType) -> Self {
        let rgba = dynamic.to_rgba32f();
        let width = rgba.width();
        let height = rgba.height();
        let data = rgba.into_raw();

        Self {
            width,
            height,
            data,
            filter_type,
        }
    }

    /// Converts normalized [0.0, 1.0] float data to an 8-bit RGBA ImageBuffer for saving to disk or GPU uploading.
    pub fn to_rgba8_buffer(&self) -> RgbaImage {
        let mut buffer = ImageBuffer::new(self.width, self.height);
        for (x, y, pixel) in buffer.enumerate_pixels_mut() {
            if let Some(Color { r, g, b, a }) = self.get_pixel(x, y) {
                *pixel = Rgba([
                    (r.clamp(0.0, 1.0) * 255.0) as u8,
                    (g.clamp(0.0, 1.0) * 255.0) as u8,
                    (b.clamp(0.0, 1.0) * 255.0) as u8,
                    (a.clamp(0.0, 1.0) * 255.0) as u8,
                ]);
            }
        }
        buffer
    }
}
