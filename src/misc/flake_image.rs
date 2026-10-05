use crate::prelude::{Color, FilterType};
use image::{DynamicImage, ImageBuffer, Rgba, RgbaImage};

/// # Image
///
/// Contains pixel data for a `width * height` image in RGBA format.
/// Data is stored as `4 * w * h` 8-bit r, g, b, a values.
#[derive(Clone, Default, Debug)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub filter_type: FilterType,
}

impl Image {
    /// Creates a new black/transparent image filled with zeros (RGBA 8-bit).
    pub fn new(width: u32, height: u32, filter_type: FilterType) -> Self {
        Self {
            width,
            height,
            data: vec![0; 4 * width as usize * height as usize],
            filter_type,
        }
    }

    /// Wraps an existing Vec<u8> RGBA buffer.
    pub fn from_data(width: u32, height: u32, filter_type: FilterType, data: Vec<u8>) -> Self {
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

    /// Creates an image from a collection of Colors.
    ///
    /// Color channels are expected to be normalized to [0.0, 1.0]
    /// and are converted to 8-bit [0, 255] values.
    pub fn from_pixels(
        width: u32,
        height: u32,
        filter_type: FilterType,
        pixels: Vec<Color>,
    ) -> Self {
        debug_assert_eq!(
            pixels.len(),
            width as usize * height as usize,
            "Pixel buffer size does not match width * height"
        );

        let mut data = Vec::with_capacity(pixels.len() * 4);

        for color in pixels {
            let [r, g, b, a] = color.to_array();

            data.extend_from_slice(&[
                (r.clamp(0.0, 1.0) * 255.0) as u8,
                (g.clamp(0.0, 1.0) * 255.0) as u8,
                (b.clamp(0.0, 1.0) * 255.0) as u8,
                (a.clamp(0.0, 1.0) * 255.0) as u8,
            ]);
        }

        Self {
            width,
            height,
            data,
            filter_type,
        }
    }

    /// Returns image with filter type.
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
    ///
    /// Color channels are expected to be normalized to [0.0, 1.0].
    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x < self.width && y < self.height {
            let idx = self.pixel_index(x, y);
            let [r, g, b, a] = color.to_array();

            self.data[idx] = (r.clamp(0.0, 1.0) * 255.0) as u8;
            self.data[idx + 1] = (g.clamp(0.0, 1.0) * 255.0) as u8;
            self.data[idx + 2] = (b.clamp(0.0, 1.0) * 255.0) as u8;
            self.data[idx + 3] = (a.clamp(0.0, 1.0) * 255.0) as u8;
        }
    }

    /// Gets the RGBA color [r, g, b, a] at (x, y).
    /// Returns None if out of bounds.
    ///
    /// Color channels are returned normalized to [0.0, 1.0].
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<Color> {
        if x < self.width && y < self.height {
            let idx = self.pixel_index(x, y);

            Some(Color::new(
                self.data[idx] as f32 / 255.0,
                self.data[idx + 1] as f32 / 255.0,
                self.data[idx + 2] as f32 / 255.0,
                self.data[idx + 3] as f32 / 255.0,
            ))
        } else {
            None
        }
    }

    /// Clears the image using a single RGBA color.
    ///
    /// Color channels are expected to be normalized to [0.0, 1.0].
    pub fn clear(&mut self, color: [f32; 4]) {
        let color = [
            (color[0].clamp(0.0, 1.0) * 255.0) as u8,
            (color[1].clamp(0.0, 1.0) * 255.0) as u8,
            (color[2].clamp(0.0, 1.0) * 255.0) as u8,
            (color[3].clamp(0.0, 1.0) * 255.0) as u8,
        ];

        for chunk in self.data.chunks_exact_mut(4) {
            chunk.copy_from_slice(&color);
        }
    }

    /// Converts this struct directly into `image::DynamicImage`.
    pub fn to_dynamic(&self) -> DynamicImage {
        let buffer = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(
            self.width,
            self.height,
            self.data.clone(),
        )
            .expect("Failed to create ImageBuffer from Image data");

        DynamicImage::ImageRgba8(buffer)
    }

    /// Creates an `Image` from an `image::DynamicImage`.
    ///
    /// Pixel data is stored directly as normalized 8-bit RGBA.
    pub fn from_dynamic(dynamic: &DynamicImage, filter_type: FilterType) -> Self {
        let rgba = dynamic.to_rgba8();
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

    /// Converts this image to an 8-bit RGBA ImageBuffer.
    ///
    /// This clones the underlying data because the returned ImageBuffer owns it.
    pub fn to_rgba8_buffer(&self) -> RgbaImage {
        ImageBuffer::from_raw(self.width, self.height, self.data.clone())
            .expect("Failed to create RGBA8 ImageBuffer")
    }
}