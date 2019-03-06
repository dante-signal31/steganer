/// Module to hide data inside an image.
///
/// It should work with any non loseless image format, currently:
/// * PNG
/// * GIF
/// * BMP
/// * ICO
/// * PNM
use image::DynamicImage;

const HEADER_LENGTH: u8 = 32;

struct ContainerImage {
    image: DynamicImage,
    width: u32,
    height: u32,
}

impl ContainerImage{
    #[must_use]
    pub fn new(file_pathname: &str)-> Self {
        let image = image::open(file_pathname)
            .expect("Something wrong happened opening given image");
        let (width, height) = image.dimensions();
        ContainerImage{image, width, height}
    }

    /// First HEADER_LENGTH pixels of container image hides a u64 with encoded
    /// data length. This functions encodes that u64 in those pixels.
    fn encode_header(&mut self, total_data_size: u64){

    }

    fn encode_bits(&mut self, bits: u32, bits_length: u8, x: u32, y: u32){

    }

    fn encode_data(&mut self, chunk_data: u32, chunk_data_length: u8, position: u64){

    }

    fn get_coordinates(position: u64)-> (u32, u32){

    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use image::{ImageBuffer, GenericImage, ImageDecoder};
    use crate::test_common::TestEnvironment;

    enum test_colors {
        BLACK,
        WHITE
    };

    fn create_test_image(fill_color: test_colors)-> (TestEnvironment, PathBuf) {
        let test_env = TestEnvironment::new();
        let color = match fill_color {
            test_colors::BLACK=> image::Rgb([0, 0, 0]),
            test_colors::WHITE=> image::Rgb([255,255,255]),
        };
        let test_image = ImageBuffer::from_fn(512, 512, |_, _| {color})
        let test_image_path = test_env.path().join("test_image.png");
        test_image.save(test_image_path)
            .except("Something wrong happened saving test image");
        (test_env, test_image_path)
    }

    #[test]
    fn test_encode_bits() {
        let (test_env, test_image_path) = create_test_image(test_colors::BLACK);
        let container = ContainerImage::new(test_image_path.to_str()
            .except("Something worng happened cpnverting test image path to str"));


    }
}
