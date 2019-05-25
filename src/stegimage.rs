/// Module to hide data inside an image.
///
/// It should work with any non loseless image format, currently:
/// * PNG
/// * BMP
/// * PPM
use std::fmt;
use std::iter::Iterator;
use image::{DynamicImage, GenericImageView};

use crate::*;
use crate::bytetools::{mask, u24_to_bytes, bytes_to_u24};
use crate::fileio::Chunk;

const HEADER_PIXEL_LENGTH: u8 = 32;
const SIZE_LENGTH: u8 = 32;
const SUPPORTED_EXTENSIONS: [&str; 3] = ["png", "bmp", "ppm"];

/// Check if this file is supported as a valid host image.
///
/// Actually this function only check image as a valid extension. Valid extensions for
/// image file are in this module *SUPPORTED_EXTENSIONS* const list.
///
/// # Parameters:
/// * filename: Host image filename. It must include an extension.
///
/// # Returns:
/// * True if this images type is supported and false if not.
/// * Can raise an error if we can not get file extension.
fn supported_image(filename: &str)-> Result<bool> {
    if filename.contains("."){
        let extension: &str = match (filename.split(".").collect::<Vec<&str>>()).last() {
            Some(ext)=> ext,
            None=> bail!("Error getting image extension.")
        };
        let normalized_extension = extension.to_lowercase();
        if SUPPORTED_EXTENSIONS.contains(&normalized_extension.as_str()) {
            Ok(true)
        } else {
            Ok(false)
        }
    } else {
        bail!("Error: host file has no extension to check it is supported.")
    }
}

/// Helper type to store Pixels positions.
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
struct Position{
    x: u32,
    y: u32
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(x:{}, y:{})", self.x, self.y)
    }
}

/// Every ContainerImage that has been identified as host of a hidden image has a ReadingState
/// type to manage hidden file extraction.
struct ReadingState {
    hidden_file_size: u32,
    chunk_size: u8,
    reading_position: u32
}

impl ReadingState {
    #[must_use]
    pub fn new(hidden_file_size: u32, chunk_size: u8, reading_position: u32)-> Self{
        ReadingState{hidden_file_size, chunk_size, reading_position}
    }
}

/// Wrapper to deal with image that is going to contain hidden file.
pub struct ContainerImage <'a> {
    image: DynamicImage,
    width: u32,
    height: u32,
    reading_state: Option<ReadingState>,
    file_pathname: &'a str,
}

impl <'a> ContainerImage <'a>{
    #[must_use]
    pub fn new(file_pathname: &'a str)-> Result<Self> {
        if let Ok(true) = supported_image(file_pathname) {
            let image = image::open(file_pathname)
                .expect("Something wrong happened opening given image");
            let (width, height) = image.dimensions();
            Ok(ContainerImage{image, width, height, reading_state: None, file_pathname})
        } else {
            bail!("Image type not supported.")
        }

    }

    /// Prepare ContainerImage to host a hidden file.
    ///
    /// It is called when you know which file to hide. When you pass in its file size
    /// it is encoded in ContainerImage header. Besides, file to hide size is used to calculate
    /// how many bits should be hidden per pixel.
    ///
    /// This method should be called once, before hide_data() is called for the first time.
    ///
    /// # Parameters:
    /// * total_data_size: File to hide size in bytes
    ///
    /// # Returns:
    /// * Bits to be hidden per pixel.
    pub fn setup_hiding(&mut self, total_data_size: u32) -> u8 {
        self.encode_header(total_data_size);
        self.get_chunk_size(total_data_size)
    }

    /// Identify this ContainerImage as hidden file host and prepare extraction.
    ///
    /// When you call this function, hidden file size is extracted from hidden and chunk size
    /// is calculated so we can know how many bits from every pixel are actually hidden data.
    ///
    /// All that info is stored in a ReadingState type into ContainerImage. After
    /// setup_extraction() creates a ReadingState instance into ContainerImage you can call
    /// that ContainerImage as an Iterator to extract hidden data chunks.
    pub fn setup_hidden_data_extraction(&mut self){
        let hidden_file_size = self.decode_header();
        let chunk_size = self.get_chunk_size(hidden_file_size);
        let reading_state = ReadingState::new(hidden_file_size, chunk_size, 0);
        self.reading_state = Some(reading_state);
    }

    /// Get needed chunk size to hide desired file into this image.
    ///
    /// # Parameter:
    /// * total_data_size: Total amount of bytes for data to be hidden.
    ///
    /// # Returns:
    /// * Chunk size. Each chunk will be encoded in a pixel.
    fn get_chunk_size(&self, total_data_size: u32)-> u8{
        let usable_pixels_amount = (self.height * self.width) - HEADER_PIXEL_LENGTH as u32;
        let total_data_size_in_bits = total_data_size * 8;
        if total_data_size_in_bits > usable_pixels_amount * 24 {
            panic!("File to be hidden is too big for this host image. Current is {} bytes \
            but maximum for this image is {} bytes", total_data_size, usable_pixels_amount * 24)
        } else {
            let bits_per_pixel = (((total_data_size_in_bits) as f32) / usable_pixels_amount as f32).ceil() as u8;
            bits_per_pixel
        }
    }

    /// First HEADER_PIXEL_LENGTH pixels of container image hides a u32 with encoded
    /// data length. This functions encodes that u32 in those pixels.
    ///
    /// This way decoding function knows how many bytes decode from host image.
    ///
    /// # Parameters:
    /// * total_data_size: Total amount of bytes for data hidden.
    fn encode_header(&mut self, total_data_size: u32){
        let bits_per_pixel = SIZE_LENGTH / HEADER_PIXEL_LENGTH;
        for i in 0..HEADER_PIXEL_LENGTH {
            let mask_for_portion = !mask(SIZE_LENGTH - bits_per_pixel, false) >> (bits_per_pixel * i);
            let bits_portion = total_data_size & mask_for_portion;
            let bits_normalized = (bits_portion as u64) >> (bits_per_pixel * (HEADER_PIXEL_LENGTH - 1 - i));
            self.encode_bits(bits_normalized as u32, bits_per_pixel, i as u32, 0);
        }
    }

    /// Read first HEADER_PIXEL_LENGTH pixels of container image to decode length of hidden
    /// data.
    ///
    /// # Returns:
    /// * Length in bytes of hidden data file.
    fn decode_header(&self)-> u32{
        let mut size = 0u32;
        let bits_per_pixel = SIZE_LENGTH / HEADER_PIXEL_LENGTH;
        for i in 0..HEADER_PIXEL_LENGTH {
            let partial_bits = self.decode_bits(i as u32, 0, bits_per_pixel);
            let left_shift = (SIZE_LENGTH - 1) - (i * bits_per_pixel);
            size += partial_bits << left_shift;
        }
        size
    }

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
        // I don't know if we have an image with alpha channel so both cases should be implemented.
        if let Some(contained_image) = self.image.as_rgba8() {
            let pixel = contained_image.get_pixel(x, y);
            ContainerImage::extract_hidden_data(&[pixel[0], pixel[1], pixel[2]], bits_length)
        } else {
            let contained_image = self.image.as_rgb8()
                .expect("Something wrong happened when accessing to inner image to encode data");
            let pixel = contained_image.get_pixel(x, y);
            ContainerImage::extract_hidden_data(&[pixel[0], pixel[1], pixel[2]], bits_length)
        }
    }

    /// Called by self.decode_bits() to get portion of pixel data that contains hidden bits.
    fn extract_hidden_data(pixel: &[u8; 3], bits_length: u8)-> u32{
        let pixel_value = bytes_to_u24(pixel);
        let recovered_bits = pixel_value & mask(bits_length, false);
        recovered_bits
    }

    /// Hide a chunk inside host image.
    ///
    /// chunk.order is used to decide which pixel is going to hide chunk.data.
    pub fn hide_data(&mut self, chunk: &Chunk){
        let Position{x, y} = self.get_coordinates(chunk.order);
        self.encode_bits(chunk.data, chunk.length, x, y);
    }

    /// Get pixel coordinates where nth chunk should be encoded.
    ///
    /// # Parameters:
    /// * position: Position of data chunk inside file to be hidden.
    ///
    /// # Returns:
    /// * Position of image pixel where this chunk should be stored.
    fn get_coordinates(&self, position: u32)-> Position{
        let offset_position = HEADER_PIXEL_LENGTH as u32 + position;
        let x = offset_position % self.width;
        let y = offset_position / self.width;
        Position{x, y}
    }

    fn get_image(&mut self)-> &mut DynamicImage {
        &mut self.image
    }
}

/// Iterator to extract hidden file content a chunk at a time.
///
/// Iterator will try to fill data attribute of Chunk. If it can not fill it, because it is
/// extracting last few bits then those bits are left justified to data attribute and length
/// attribute is set to how many files it was able to read.
impl <'a> Iterator for ContainerImage <'a>{
    type Item = Chunk;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(state) = &self.reading_state {
            let bit_position = state.reading_position * state.chunk_size as u32;
            if bit_position < (state.hidden_file_size * 8) {
                let reading_coordinates = self.get_coordinates(state.reading_position);
                let extracted_bits = self.decode_bits(reading_coordinates.x, reading_coordinates.y, state.chunk_size);
                let returned_chunk = Chunk::new(extracted_bits, state.chunk_size, state.reading_position);
                let next_reading_position = state.reading_position + 1;
                let new_state = ReadingState::new(state.hidden_file_size,
                                                  state.chunk_size,
                                                  next_reading_position);
                self.reading_state = Some(new_state);
                Some(returned_chunk)
            } else { // No more hidden data left in container image.
                None
            }
        } else {
            panic!("You tried to use this ContainerImage as an Iterator before calling setup_hidden_data_extraction().");
        }
    }
}

/// Save to file every change done over image.
///
/// Image crate works in memory so changes should be written before disposing ContainerImage.
impl <'a> Drop for ContainerImage <'a> {
    fn drop(&mut self) {
        if let None = &self.reading_state {
            self.image.save(self.file_pathname)
                .expect("Image could not be saved");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use bitreader::BitReader;
    use image::{ImageBuffer, GenericImageView};
//    use std::mem::size_of_val;

    use crate::bytetools::get_bits;
    use test_common::TestEnvironment;

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
    fn test_supported_image() {
        // Check supported images.
        assert!(supported_image("path/dummy.png").unwrap_or(false));
        assert!(supported_image("path1/path2/dummy.ppm").unwrap_or(false));
        assert!(supported_image("dummy.bmp").unwrap_or(false));
        // Check unsupported images.
        assert!(!supported_image("dummy.jpg").unwrap_or(false));
        assert!(!supported_image("path/dummy.ico").unwrap_or(false));
    }

    #[test]
    fn test_support_image_with_no_extension() {
        if let Err(ref errors) = supported_image("path/dummy"){
            let mut error_message_found = false;
            for (index, error) in errors.iter().enumerate(){
                    let message: &str = error.description();
                    if message.contains("no extension") { error_message_found = true; }
            }
            if !error_message_found { assert!(false) };
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_get_chunk_size() {
        let (test_env, test_image_path) = create_test_image(TestColors::BLACK);
        let container = ContainerImage::new(test_image_path.to_str()
            .expect("Something wrong happened converting test image path to str")).unwrap();
        // Temporary test image has 512x512 = 262.144 pixels.
        // But we use first HEADER_PIXEL_LENGTH bits for header, so we can use
        // 262.144 - HEADER_PIXEL_LENGTH to hide data.
        let chunk_size = container.get_chunk_size(8156); // Size of resources/genesis.txt is 8156.
        let expected_chunk_size = ((8156_f64 * 8_f64) / ((512_f64*512_f64) - HEADER_PIXEL_LENGTH as f64)).ceil() as u8;
        assert_eq!(expected_chunk_size, chunk_size,
                   "Recovered chunk size was not what we were expecting. Expected {} but got {}",
                   expected_chunk_size, chunk_size);
    }

    #[test]
    #[should_panic]
    fn test_get_chunk_size_file_too_big() {
        let (test_env, test_image_path) = create_test_image(TestColors::BLACK);
        let container = ContainerImage::new(test_image_path.to_str()
            .expect("Something wrong happened converting test image path to str")).unwrap();
        // Temporary test image has 512x512 = 262.144 pixels.
        // But we use first HEADER_PIXEL_LENGTH bits for header, so we can use
        // 262.144 - HEADER_PIXEL_LENGTH to hide data = 262.112 pixels.
        // Every pixel can hide up to 24 bits os hidden data, so this
        // image can hide up to 6.290.688 bits = 786.336 bytes.
        let chunk_size = container.get_chunk_size(800000);
    }

    #[test]
    fn test_encode_header() {
        let encoded_size: u32 = 33;
        let (test_env, test_image_path) = create_test_image(TestColors::BLACK);
        let mut container = ContainerImage::new(test_image_path.to_str()
            .expect("Something wrong happened converting test image path to str")).unwrap();
        container.encode_header(encoded_size);
        let mut recovered_size: u64 = 0;
        let bits_per_pixel = SIZE_LENGTH / HEADER_PIXEL_LENGTH;
        for i in 0..HEADER_PIXEL_LENGTH {
            let pixel = container.get_image().get_pixel(i as u32,0);
            let pixel_hidden_bits = bytes_to_u24(&[pixel[0], pixel[1], pixel[2]]) & mask(bits_per_pixel, false);
            recovered_size += (pixel_hidden_bits as u64) << (bits_per_pixel * (HEADER_PIXEL_LENGTH - 1 - i));
        }
        assert_eq!(recovered_size as u32, encoded_size,
            "Error recovering encoded header: Expected {} but recovered {}",
            encoded_size, recovered_size as u32);
    }

    #[test]
    fn test_decode_header() {
        let encoded_size: u32 = 33;
        let (test_env, test_image_path) = create_test_image(TestColors::BLACK);
        let mut container = ContainerImage::new(test_image_path.to_str()
            .expect("Something wrong happened converting test image path to str")).unwrap();
        let bits_per_pixel = SIZE_LENGTH / HEADER_PIXEL_LENGTH;
        for i in 0..HEADER_PIXEL_LENGTH {
            // First encode header manually.
            let bits = get_bits(encoded_size, i * bits_per_pixel, bits_per_pixel) as u32;
            let pixel = container.get_image().as_mut_rgb8()
                .expect("Error accessing to test image")
                .get_pixel_mut(i as u32, 0);
            let original_value = bytes_to_u24(&[pixel[0], pixel[1], pixel[2]]);
            let modified_value = (original_value & mask(bits_per_pixel, true)) + bits;
            let modified_bytes = u24_to_bytes(modified_value);
            *pixel = image::Rgb([modified_bytes[0], modified_bytes[1], modified_bytes[2]]);
        }
        // Now decode with tested function.
        let decoded_size = container.decode_header();
        assert_eq!(decoded_size, encoded_size, "Error decoding header: Expected {} but recovered {}",
                   encoded_size, decoded_size);
    }

    #[test]
    fn test_encode_less_than_8_bits() {
        let test_bits: u32 = 0b_10110;
        let test_bits_length: u8 = 5;
        let (test_env, test_image_path) = create_test_image(TestColors::BLACK);
        let mut container = ContainerImage::new(test_image_path.to_str()
            .expect("Something wrong happened converting test image path to str")).unwrap();
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
            .expect("Something wrong happened converting test image path to str")).unwrap();
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
            .expect("Something wrong happened converting test image path to str")).unwrap();
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
            .expect("Something wrong happened converting test image path to str")).unwrap();
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
            .expect("Something wrong happened converting test image path to str")).unwrap();
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
            .expect("Something wrong happened converting test image path to str")).unwrap();
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
        let container = ContainerImage::new(test_image_path.to_str()
            .expect("Something wrong happened converting test image path to str")).unwrap();
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
        let container = ContainerImage::new(test_image_path.to_str()
            .expect("Something wrong happened converting test image path to str")).unwrap();
        let recovered_bits = container.decode_bits( 0, 0, test_bits_length);
        let recovered_bytes = u24_to_bytes(recovered_bits);
        assert_eq!(expected_upper_byte, recovered_bytes[1],
                   "Error decoding more than 8 bits. Upper byte expected {} but encoded {}",
                   expected_upper_byte, recovered_bytes[1]);
        assert_eq!(expected_lower_byte, recovered_bytes[2],
                   "Error decoding more than 8 bits. Lower byte expected {} but encoded {}",
                   expected_lower_byte, recovered_bytes[2]);
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
        let container = ContainerImage::new(test_image_path.to_str()
            .expect("Something wrong happened converting test image path to str")).unwrap();
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

    #[test]
    fn test_get_coordinates() {
        let test_image_width: u32 = 512;
        let position_first_row = 5;
        let position_second_row = 570;
        let position_third_row = 1100;
        let expected_first_row_coordinates = Position{x: (HEADER_PIXEL_LENGTH + 5) as u32, y: 0};
        let expected_second_row_coordinates = Position{x: (position_second_row as u32 - test_image_width + HEADER_PIXEL_LENGTH as u32), y: 1};
        let expected_third_row_coordinates = Position{x: (position_third_row as u32 - (test_image_width * 2) + HEADER_PIXEL_LENGTH as u32), y: 2};
        // Test environment build.
        let (test_env, test_image_path) = create_test_image(TestColors::BLACK);
        let container = ContainerImage::new(test_image_path.to_str()
            .expect("Something wrong happened converting test image path to str")).unwrap();
        // Tests.
        let recovered_position_first_row = container.get_coordinates(position_first_row);
        assert_eq!(expected_first_row_coordinates, recovered_position_first_row,
                   "Recovered position for first row was not what we were expecting. Expected {} but got {}",
                   &expected_first_row_coordinates, recovered_position_first_row);
        let recovered_position_second_row = container.get_coordinates(position_second_row);
        assert_eq!(expected_second_row_coordinates, recovered_position_second_row,
                   "Recovered position for second row was not what we were expecting. Expected {} but got {}",
                   &expected_second_row_coordinates, recovered_position_second_row);
        let recovered_position_third_row = container.get_coordinates(position_third_row);
        assert_eq!(expected_third_row_coordinates, recovered_position_third_row,
                   "Recovered position for third row was not what we were expecting. Expected {} but got {}",
                   &expected_third_row_coordinates, recovered_position_third_row);
    }

    #[test]
    fn test_encode_data() {
        let hidden_data = 0b_111000111_u32;
        let hidden_data_length = 9;
        let position = 5_u8;
        let chunk = Chunk::new(hidden_data, hidden_data_length, position as u32);
        // Test environment build.
        let (test_env, test_image_path) = create_test_image(TestColors::BLACK);
        let mut container = ContainerImage::new(test_image_path.to_str()
            .expect("Something wrong happened converting test image path to str")).unwrap();
        // Test:
        container.hide_data(&chunk);
        let pixel = container.get_image().get_pixel((HEADER_PIXEL_LENGTH + position) as u32, 0);
        assert_eq!(0b_1_u8, pixel.data[1],
                   "Recovered data for upper byte was not what we were expecting. Expected {:#b} but got {:#b}",
                   0b_1_u8, pixel.data[1]);
        assert_eq!(0b_11000111_u8, pixel.data[2],
                   "Recovered data for lower byte was not what we were expecting. Expected {:#b} but got {:#b}",
                   0b_11000111_u8, pixel.data[2]);
    }

    #[test]
    fn test_header_and_hidden_data_dont_overlap() {
        let header = 0b_1_u32;
        let hidden_data = 0b_0000_0000_0000_0000_1010_0101_1100_0111_u32;
        let hidden_data_length = 24;
        let position = 0_u8;
        let chunk = Chunk::new(hidden_data, hidden_data_length, position as u32);
        // Test environment build.
        let (test_env, test_image_path) = create_test_image(TestColors::BLACK);
        let mut container = ContainerImage::new(test_image_path.to_str()
            .expect("Something wrong happened converting test image path to str")).unwrap();
        // Test:
        container.encode_header(header);
        container.hide_data(&chunk);
        let recovered_header = container.decode_header();
        assert_eq!(header, recovered_header,
                   "Recovered data for header was not what we were expecting. Expected {:#b} but got {:#b}",
                   header, recovered_header);
        let pixel = container.get_image().get_pixel((HEADER_PIXEL_LENGTH + position) as u32, 0);
        assert_eq!(0b_0000_0000_u8, pixel.data[0],
                   "Recovered data for upper byte was not what we were expecting. Expected {:#b} but got {:#b}",
                   0b_0000_0000_u8, pixel.data[0]);
        assert_eq!(0b_1010_0101__u8, pixel.data[1],
                   "Recovered data for middle byte was not what we were expecting. Expected {:#b} but got {:#b}",
                   0b_1010_0101_u8, pixel.data[1]);
        assert_eq!(0b_1100_0111_u8, pixel.data[2],
                   "Recovered data for lower byte was not what we were expecting. Expected {:#b} but got {:#b}",
                   0b_1100_0111_u8, pixel.data[2]);
    }
    
    #[test]
    fn test_container_image_iterator() {
        let hidden_data: [u32; 3] = [0b_1010_0101_1100_0111_u32,
            0b_1111_0000_0000_1111_u32,
            0b_1111_1010_0000_0000_u32];
        // I actually hide only 3 bytes per u32 so hidden file size is 3*3 instead of 3*4 bytes.
        let hidden_data_size = (3*3) as usize;
        // Build test environment.
        let (test_env, test_image_path) = create_test_image(TestColors::BLACK);
        let mut container = ContainerImage::new(test_image_path.to_str()
            .expect("Something wrong happened converting test image path to str")).unwrap();
        // Populate test environment with hidden data.
        let chunk_size = container.setup_hiding(hidden_data_size as u32);
        let mut position = 0_u32;
        for data in hidden_data.iter() {
            let data_bytes = u24_to_bytes(*data);
            let mut bit_reader = BitReader::new(&data_bytes);
            for _ in 0..(24/chunk_size) {
                let data_chunk = bit_reader.read_u32(chunk_size)
                    .expect("Error reading data chunk.");
                let chunk = Chunk::new(data_chunk, chunk_size, position as u32);
                container.hide_data(&chunk);
                position += 1;
            }
        }
        // Test.
        let mut recovered_data: [u32; 3] = [0; 3];
        container.setup_hidden_data_extraction();
        for (i, chunk) in container.enumerate() {
            let u24_index = i / 24;
            recovered_data[u24_index] = (recovered_data[u24_index] << chunk_size) + chunk.data;
        }
        assert_eq!(hidden_data, recovered_data,
                   "ContainerImage iterator did not recover expected data. Expected {:#?} but recovered {:#?}",
                   hidden_data, recovered_data)
    }

    #[test]
    fn test_drop() {
        let dummy_size = 6363_u32;
        // Build test environment.
        let (test_env, test_image_path) = create_test_image(TestColors::BLACK);
        {
            let mut container = ContainerImage::new(test_image_path.to_str()
                .expect("Something wrong happened converting test image path to str")).unwrap();
            let _ = container.setup_hiding(dummy_size);
        } // Here container should be written to disk, with dummy_size encoded at its header, before dropping container.
        // Now try to recover encoded size.
        let mut container = ContainerImage::new(test_image_path.to_str()
            .expect("Something wrong happened converting test image path to str")).unwrap();
        container.setup_hidden_data_extraction();
        if let Some(state) = &container.reading_state {
            let extracted_size = state.hidden_file_size;
            assert_eq!(dummy_size, extracted_size,
                       "Recovered size is not what we were expecting. Expected {} but recovered {}.",
                       dummy_size, extracted_size);
        } else {
            assert!(false, "No reading state recovered");
        }
    }
}
