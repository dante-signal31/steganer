/// Module to hide data inside an image.
///
/// It should work with any non loseless image format, currently:
/// * PNG
/// * GIF
/// * BMP
/// * ICO
/// * PNM
use image::{DynamicImage, GenericImage, GenericImageView};

use crate::bytetools::{mask, u24_to_bytes};

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

//    /// First HEADER_LENGTH pixels of container image hides a u64 with encoded
//    /// data length. This functions encodes that u64 in those pixels.
//    fn encode_header(&mut self, total_data_size: u64){
//
//    }

    /// Encode given bits at pixel defined by x and y coordinates.
    ///
    /// # Parameters:
    /// * bits: Data to be hidden.
    /// * bits_length: How many bits at bits parameter are actually data to be hidden.
    /// * x: X coordinate of pixel where data is going to be hidden.
    /// * y: Y coordinate of pixel where data is going to be hidden.
    fn encode_bits(&mut self, bits: u32, bits_length: u8, x: u32, y: u32){
        // We don't know if host image is going to have an alpha channel or not. So
        // we must implement both cases.
        if let Some(contained_image) = self.image.as_mut_rgba8() {
            let pixel = contained_image.get_pixel_mut(x, y);
            let modified_pixel_bytes = ContainerImage::overwrite_pixel(&pixel.data[..3], bits, bits_length);
            *pixel = image::Rgba([modified_pixel_bytes[0],
                modified_pixel_bytes[1],
                modified_pixel_bytes[2],
                pixel[3]]); // We keep original Alpha channel.

        } else {
            let contained_image = self.image.as_mut_rgb8()
                .expect("Something wrong happened when accessing to inner image to encode data");
            let pixel = contained_image.get_pixel_mut(x, y);
            let modified_pixel_bytes = ContainerImage::overwrite_pixel(&pixel.data[..3], bits, bits_length);
            *pixel = image::Rgb([modified_pixel_bytes[0],
                modified_pixel_bytes[1],
                modified_pixel_bytes[2]]);
        }
    }

    /// Called by self.encode_bits() to get which value should have host pixel after data hidding.
    fn overwrite_pixel(rgb: &[u8], bits: u32, bits_length: u8)-> [u8; 3]{
        let original_pixel_value: u32 = ((rgb[0] as u32) << 16) + ((rgb[1] as u32) << 8) + (rgb[2] as u32);
        let modified_pixel_value = (original_pixel_value & mask(bits_length, true)) + bits;
        let modified_pixel_bytes = u24_to_bytes(modified_pixel_value);
        modified_pixel_bytes
    }

    /// Decode bits hidden into given pixel defined by x and y coordinates.
    ///
    /// # Parameters:
    /// * x: X coordinate of pixel where data is going to be hidden.
    /// * y: Y coordinate of pixel where data is going to be hidden.
    /// * bits_length: How many bits at pixel are actually hiden data.
    ///
    /// # Returns:
    /// * Recovered bits are returned into a u32.
    fn decode_bits(&self, x: u32, y: u32, bits_length: u8)-> u32{
       0
    }

//    fn encode_data(&mut self, chunk_data: u32, chunk_data_length: u8, position: u64){
//
//    }

//    fn get_coordinates(position: u64)-> (u32, u32){
//
//    }

    fn get_image(&self)-> &DynamicImage {
        &self.image
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use image::{ImageBuffer, GenericImageView, ImageDecoder};
    use crate::test_common::TestEnvironment;

    enum TestColors {
        BLACK,
        WHITE
    }

    fn create_test_image(fill_color: TestColors) -> (TestEnvironment, PathBuf) {
        let test_env = TestEnvironment::new();
        let test_image_path = match fill_color {
            TestColors::BLACK=> save_image_filled(&test_env, [0, 0, 0]),
            TestColors::WHITE=> save_image_filled(&test_env, [255, 255, 255]),
        };
        (test_env, test_image_path)
    }

    fn create_test_image_with_custom_color(fill_color: u32)-> (TestEnvironment, PathBuf){
        let test_env = TestEnvironment::new();
        let test_image_path = save_image_filled(&test_env, u24_to_bytes(fill_color));
        (test_env, test_image_path)
    }

    fn save_image_filled(test_env: &TestEnvironment, color: [u8; 3])-> PathBuf{
        let color = image::Rgb(color);
        let test_image = ImageBuffer::from_fn(512, 512, |_, _| {color});
        let test_image_path = test_env.path().join("test_image.png");
        test_image.save(&test_image_path)
            .expect("Something wrong happened saving test image");
        test_image_path
    }

    #[test]
    fn test_encode_less_than_8_bits() {
        let test_bits: u32 = 0b_10110;
        let test_bits_length: u8 = 5;
        let (test_env, test_image_path) = create_test_image(TestColors::BLACK);
        let mut container = ContainerImage::new(test_image_path.to_str()
            .expect("Something wrong happened converting test image path to str"));
        container.encode_bits(test_bits, test_bits_length, 0, 0);
        let pixel = container.get_image().get_pixel(0,0);
        assert_eq!(pixel.data[2], test_bits as u8,
                   "Error encoding less than 8 bits. Expected {} but encoded {}",
                   test_bits, pixel.data[2]);
    }

    #[test]
    fn test_encode_up_to_16_bits() {
        let mut test_bits: u32 = 0;
        let expected_upper_byte: u8 = 0b_00110100;
        let expected_lower_byte: u8 = 0b_00010110;
        test_bits = test_bits + (expected_upper_byte as u32);
        test_bits = test_bits << 8;
        test_bits = test_bits + (expected_lower_byte as u32);
        let test_bits_length: u8 = 14;
        let (test_env, test_image_path) = create_test_image(TestColors::BLACK);
        let mut container = ContainerImage::new(test_image_path.to_str()
            .expect("Something wrong happened converting test image path to str"));
        container.encode_bits(test_bits, test_bits_length, 0, 0);
        let mut pixel = container.get_image().get_pixel(0,0);
        pixel = container.get_image().get_pixel(0,0);
        assert_eq!(pixel.data[1], expected_upper_byte,
                   "Error encoding more than 8 bits. Upper byte expected {} but encoded {}",
                   expected_upper_byte, pixel.data[1]);
        assert_eq!(pixel.data[2], expected_lower_byte,
                   "Error encoding more than 8 bits. Lower byte expected {} but encoded {}",
                   expected_lower_byte, pixel.data[2]);
    }

    #[test]
    fn test_encode_up_to_24_bits() {
        let mut test_bits: u32 = 0;
        let expected_upper_byte: u8 = 0b_00000110;
        let expected_middle_byte: u8 = 0b_00110100;
        let expected_lower_byte: u8 = 0b_00010110;
        test_bits = test_bits + ((expected_upper_byte as u32) << 16) +
            ((expected_middle_byte as u32) << 8) +
            (expected_lower_byte as u32);
        let test_bits_length: u8 = 19;
        let (test_env, test_image_path) = create_test_image(TestColors::BLACK);
        let mut container = ContainerImage::new(test_image_path.to_str()
            .expect("Something wrong happened converting test image path to str"));
        container.encode_bits(test_bits, test_bits_length, 0, 0);
        let mut pixel = container.get_image().get_pixel(0,0);
        pixel = container.get_image().get_pixel(0,0);
        assert_eq!(pixel.data[0], expected_upper_byte,
                   "Error encoding more than 16 bits. Upper byte expected {} but encoded {}",
                   expected_upper_byte, pixel.data[0]);
        assert_eq!(pixel.data[1], expected_middle_byte,
                   "Error encoding more than 16 bits. Middle byte expected {} but encoded {}",
                   expected_middle_byte, pixel.data[1]);
        assert_eq!(pixel.data[2], expected_lower_byte,
                   "Error encoding more than 16 bits. Lower byte expected {} but encoded {}",
                   expected_lower_byte, pixel.data[2]);
    }

    #[test]
    fn test_encode_less_than_8_bits_masked() {
        let test_bits: u32 = 0b_10110;
        let expected_recovered_bits: u8 = 0b_111_10110;
        let test_bits_length: u8 = 5;
        let (test_env, test_image_path) = create_test_image(TestColors::WHITE);
        let mut container = ContainerImage::new(test_image_path.to_str()
            .expect("Something wrong happened converting test image path to str"));
        container.encode_bits(test_bits, test_bits_length, 0, 0);
        let pixel = container.get_image().get_pixel(0,0);
        assert_eq!(pixel.data[2], expected_recovered_bits,
                   "Error encoding less than 8 bits masked. Expected {} but encoded {}",
                   expected_recovered_bits, pixel.data[2]);
    }

    #[test]
    fn test_encode_up_to_16_bits_masked() {
        let mut test_bits: u32 = 0;
        let expected_upper_byte: u8 = 0b_00110100;
        let expected_lower_byte: u8 = 0b_00010110;
        test_bits = test_bits + (expected_upper_byte as u32);
        test_bits = test_bits << 8;
        test_bits = test_bits + (expected_lower_byte as u32);
        let test_bits_length: u8 = 14;
        let expected_recovered_upper_byte: u8 = 0b_11_110100;
        let (test_env, test_image_path) = create_test_image(TestColors::WHITE);
        let mut container = ContainerImage::new(test_image_path.to_str()
            .expect("Something wrong happened converting test image path to str"));
        container.encode_bits(test_bits, test_bits_length, 0, 0);
        let mut pixel = container.get_image().get_pixel(0,0);
        pixel = container.get_image().get_pixel(0,0);
        assert_eq!(pixel.data[1], expected_recovered_upper_byte,
                   "Error encoding more than 8 bits. Upper byte expected {} but encoded {}",
                   expected_recovered_upper_byte, pixel.data[1]);
        assert_eq!(pixel.data[2], expected_lower_byte,
                   "Error encoding more than 8 bits. Lower byte expected {} but encoded {}",
                   expected_lower_byte, pixel.data[2]);
    }

    #[test]
    fn test_encode_up_to_24_bits_masked() {
        let mut test_bits: u32 = 0;
        let expected_upper_byte: u8 = 0b_00000110;
        let expected_middle_byte: u8 = 0b_00110100;
        let expected_lower_byte: u8 = 0b_00010110;
        test_bits = test_bits + ((expected_upper_byte as u32) << 16) +
            ((expected_middle_byte as u32) << 8) +
            (expected_lower_byte as u32);
        let test_bits_length: u8 = 19;
        let expected_recovered_upper_byte: u8 = 0b_11111_110;
        let (test_env, test_image_path) = create_test_image(TestColors::WHITE);
        let mut container = ContainerImage::new(test_image_path.to_str()
            .expect("Something wrong happened converting test image path to str"));
        let mut pixel = container.get_image().get_pixel(0,0);
        container.encode_bits(test_bits, test_bits_length, 0, 0);
        pixel = container.get_image().get_pixel(0,0);
        assert_eq!(pixel.data[0], expected_recovered_upper_byte,
                   "Error encoding more than 16 bits. Upper byte expected {} but encoded {}",
                   expected_recovered_upper_byte, pixel.data[0]);
        assert_eq!(pixel.data[1], expected_middle_byte,
                   "Error encoding more than 16 bits. Middle byte expected {} but encoded {}",
                   expected_middle_byte, pixel.data[1]);
        assert_eq!(pixel.data[2], expected_lower_byte,
                   "Error encoding more than 16 bits. Lower byte expected {} but encoded {}",
                   expected_lower_byte, pixel.data[2]);
    }

    #[test]
    fn test_decode_less_than_8_bits() {
        let test_bits: u32 = 0b_10110;
        let test_bits_length: u8 = 5;
        let (test_env, test_image_path) = create_test_image_with_custom_color(test_bits);
        let mut container = ContainerImage::new(test_image_path.to_str()
            .expect("Something wrong happened converting test image path to str"));
        let recovered_bits = container.decode_bits( 0, 0, test_bits_length);
        assert_eq!(test_bits, recovered_bits,
                   "Error decoding less than 8 bits. Expected {} but encoded {}",
                   test_bits, recovered_bits);
    }

    #[test]
    fn test_decode_up_to_16_bits() {
        let mut test_bits: u32 = 0;
        let expected_upper_byte: u8 = 0b_00110100;
        let expected_lower_byte: u8 = 0b_00010110;
        test_bits = test_bits + (expected_upper_byte as u32);
        test_bits = test_bits << 8;
        test_bits = test_bits + (expected_lower_byte as u32);
        let test_bits_length: u8 = 14;
        let (test_env, test_image_path) = create_test_image_with_custom_color(test_bits);
        let mut container = ContainerImage::new(test_image_path.to_str()
            .expect("Something wrong happened converting test image path to str"));
        let recovered_bits = container.decode_bits( 0, 0, test_bits_length);
        let recovered_bytes = u24_to_bytes(recovered_bits);
        assert_eq!(expected_upper_byte, recovered_bytes[0],
                   "Error decoding more than 8 bits. Upper byte expected {} but encoded {}",
                   expected_upper_byte, recovered_bytes[0]);
        assert_eq!(expected_lower_byte, recovered_bytes[1],
                   "Error decoding more than 8 bits. Lower byte expected {} but encoded {}",
                   expected_lower_byte, recovered_bytes[1]);
    }

    #[test]
    fn test_decode_up_to_24_bits() {
        let mut test_bits: u32 = 0;
        let expected_upper_byte: u8 = 0b_00000110;
        let expected_middle_byte: u8 = 0b_00110100;
        let expected_lower_byte: u8 = 0b_00010110;
        test_bits = test_bits + ((expected_upper_byte as u32) << 16) +
            ((expected_middle_byte as u32) << 8) +
            (expected_lower_byte as u32);
        let test_bits_length: u8 = 19;
        let (test_env, test_image_path) = create_test_image_with_custom_color(test_bits);
        let mut container = ContainerImage::new(test_image_path.to_str()
            .expect("Something wrong happened converting test image path to str"));
        let recovered_bits = container.decode_bits( 0, 0, test_bits_length,);
        let recovered_bytes = u24_to_bytes(recovered_bits);
        assert_eq!(expected_upper_byte, recovered_bytes[0],
                   "Error decoding more than 16 bits. Upper byte expected {} but decoded {}",
                   expected_upper_byte, recovered_bytes[0]);
        assert_eq!(expected_middle_byte, recovered_bytes[1],
                   "Error decoding more than 16 bits. Middle byte expected {} but decoded {}",
                   expected_middle_byte, recovered_bytes[1]);
        assert_eq!(expected_lower_byte, recovered_bytes[2],
                   "Error decoding more than 16 bits. Lower byte expected {} but decoded {}",
                   expected_lower_byte, recovered_bytes[2]);

    }
}
