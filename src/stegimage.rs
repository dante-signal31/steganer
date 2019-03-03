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
    use image::{ImageBuffer, GenericImage};

    fn create_test_image()-> GenericImage{
        let test_image = ImageBuffer::new(512, 512);

    }
}
