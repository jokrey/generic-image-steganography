use std::{fmt, io};
use std::ops::Index;
use std::ops::IndexMut;
use std::path::Path;

use image::{DynamicImage, GenericImageView, ImageError, ImageResult, Rgba};
use image::error::{ParameterError, ParameterErrorKind};
use rand::{Rng, thread_rng};

use jokrey_utilities::general::{get_bit_at, sample_unique, set_bit_at};


pub fn get_message_length_in_bits(message: &[u8]) -> usize {
    message.len()*8
}

pub fn randomly_select_indices_within(message_bytes: &[u8], original: &dyn EncodingContainer) -> Vec<usize> {
    randomly_select_indices(get_message_length_in_bits(message_bytes), original.len())
}
pub fn randomly_select_indices(message_length_in_bits: usize, max_index: usize) -> Vec<usize> {
    let mut selected_indices = sample_unique(0..max_index, message_length_in_bits);
    selected_indices.sort();
    selected_indices
}

pub fn encode_into_vec(message_bytes: &[u8], original: &dyn EncodingContainer, selected_indices: &[usize]) -> DifCodeResult<Vec<u8>> {
    let mut encoded= vec![0; original.len()];
    encode(message_bytes, original, selected_indices, &mut encoded)?;
    Ok(encoded)
}
pub fn encode_into_image(message_bytes: &[u8], original: &DifCodeImage, selected_indices: &[usize]) -> DifCodeResult<DifCodeImage> {
    let mut encoded_image = DifCodeImage::with_capacity(original.width(), original.height());
    encode(&message_bytes, original, selected_indices, &mut encoded_image)?;
    Ok(encoded_image)
}
pub fn encode_into_image_into_path(message_bytes: &[u8], original: &DifCodeImage, selected_indices: &[usize], path: &str) -> DifCodeResult<()> {
    let encoded_image= encode_into_image(message_bytes, original, selected_indices)?;
    encoded_image.save(path)?;
    Ok(())
}

///Note: selected_indices must be sorted in ascending order
pub fn encode(message_bytes: &[u8], original: &dyn EncodingContainer, selected_indices: &[usize], encoded: &mut dyn EncodingContainer) -> DifCodeResult<()> {
    if get_message_length_in_bits(message_bytes) != selected_indices.len() {
        return Err(DifCodeError::from("message length in bits != selected indices length (not enough or too many selected indices)"));
    }

    let mut rng = thread_rng();

    let mut index_in_original = 0;
    let mut selected_indices_iterator = selected_indices.iter();
    for message_byte in message_bytes {
        for bit_i in 0..8 {
            let bit_to_encode = get_bit_at(*message_byte, bit_i);

            let selected_index_to_encode_at = selected_indices_iterator.next().unwrap();
            while index_in_original < *selected_index_to_encode_at {
                encoded[index_in_original] = original[index_in_original];
                index_in_original += 1;
            }

            let goal_difference: u8 = if bit_to_encode { 1 } else { 2 };
            encode_difference_into(*selected_index_to_encode_at, original, encoded, goal_difference, rng.gen_bool(0.5));
            index_in_original += 1;
        }
    }
    while index_in_original < original.len() {
        encoded[index_in_original] = original[index_in_original];
        index_in_original += 1;
    }
    Ok(())
}

fn encode_difference_into(index: usize, original: &dyn EncodingContainer, encoded: &mut dyn EncodingContainer, goal_difference: u8, difference_sign_positive_desired: bool) {
    let original_value = original[index];
    if difference_sign_positive_desired {
        if original_value > 255 - goal_difference { //does not fit, must subtract - despite wishes
            encoded[index] = original_value - goal_difference;
        } else {
            encoded[index] = original_value + goal_difference;
        }
    } else {
        if original_value < goal_difference { //does not fit, must add - despite wishes
            encoded[index] = original_value + goal_difference;
        } else {
            encoded[index] = original_value - goal_difference;
        }
    }
}






pub fn decode_into_vec(original: &dyn EncodingContainer, encoded: &dyn EncodingContainer) -> DifCodeResult<Vec<u8>> {
    if original.len() != encoded.len() {
        panic!("original data length != encoded data length")
    }

    let msg_length_in_bits = get_encoded_message_length_in_bits(original, encoded);
    if ! (msg_length_in_bits % 8 == 0) {
        return Err(DifCodeError::from("invalid input data (message length incorrect)"));
    }

    let mut decoded_message = vec![0u8; msg_length_in_bits/8];

    decode(original, encoded, msg_length_in_bits, decoded_message.as_mut_slice())?;

    Ok(decoded_message)
}

pub fn decode(original: &dyn EncodingContainer, encoded: &dyn EncodingContainer, msg_length_in_bits: usize, decoded_message_buffer: &mut [u8]) -> DifCodeResult<()> {
    let mut code_text_index:usize = 0;
    let mut message_index = 0;
    let mut message_byte = 0u8;
    let mut message_bit_index = 0;
    for _ in 0..msg_length_in_bits {
        let mut decoded_bit = None;
        while decoded_bit == None && code_text_index < original.len() {
            decoded_bit = decode_from_difference(code_text_index, original, encoded);
            code_text_index += 1;
        }

        if decoded_bit == Some(true) {
            message_byte = set_bit_at(message_byte, message_bit_index);
        }

        message_bit_index += 1;
        if message_bit_index == 8 {
            decoded_message_buffer[message_index] = message_byte;

            message_index += 1;
            message_byte = 0u8;
            message_bit_index = 0;
        }
    }
    Ok(())
}

fn decode_from_difference(index: usize, original: &dyn EncodingContainer, encoded: &dyn EncodingContainer) -> Option<bool> {
    let difference = original[index] as isize - encoded[index] as isize;

    match difference.abs() {
        1 => Some(true),
        2 => Some(false),
        _ => None
    }
}

pub fn get_encoded_message_length_in_bits(original: &dyn EncodingContainer, encoded: &dyn EncodingContainer) -> usize {
    let mut counter: usize = 0;
    for index in 0..original.len() {
        if original[index] != encoded[index] {
            counter += 1;
        }
    }
    return counter;
}

pub trait EncodingContainer:Index<usize, Output = u8>+IndexMut<usize> {
    fn len(&self) -> usize;
}
impl EncodingContainer for Vec<u8> {
    fn len(&self) -> usize {
        self.len()
    }
}
impl EncodingContainer for [u8] {
    fn len(&self) -> usize {
        self.len()
    }
}

pub struct DifCodeImage(DynamicImage);
impl DifCodeImage {
    pub fn from(orig: DynamicImage) -> DifCodeImage { DifCodeImage(orig) }
    pub fn width(&self) -> u32  { self.0.width()  }
    pub fn height(&self) -> u32 { self.0.height() }
    pub fn get_pixel(&self, x: u32, y: u32) -> Rgba<u8> { self.0.get_pixel(x, y) }
    pub fn index_to_xyz(&self, index: usize) -> (u32, u32, usize) {
        let mut index = index as u32;
        let z = index / (self.width() * self.height());
        index -= z * self.width() * self.height();
        let y = index / self.width();
        let x = index % self.width();
        return (x, y, z as usize)
    }

    pub fn with_capacity(width: u32, height: u32) -> DifCodeImage {
        DifCodeImage(DynamicImage::new_rgb8(width, height))
    }
    pub fn save(&self, path: &str) -> ImageResult<()> {
        self.save_to(Path::new(path))
    }
    pub fn save_to(&self, path: &Path) -> ImageResult<()> {
        let ext = path.extension()
            .and_then(|s| s.to_str())
            .map_or("".to_string(), |s| s.to_ascii_lowercase());
        if &*ext == "jpg" || &*ext == "jpeg" {
            Err(ImageError::Parameter(ParameterError::from_kind(ParameterErrorKind::Generic("Encoding as JPEG currently not supported (require lossless)".to_string()))))
            // //require quality == 100, i.e. lossless compression
            // let fout = &mut BufWriter::new(File::create(path)?);
            // self.0.write_to(fout, image::ImageOutputFormat::Jpeg(255))
        } else {
            self.0.save(path)
        }
    }
    pub fn open(path: &str) -> Result<DifCodeImage, ImageError> {
        Ok(DifCodeImage::from(image::open(path)?))
    }
    pub fn from_memory(buffer: &[u8]) -> Result<DifCodeImage, ImageError> {
        Ok(DifCodeImage::from(image::load_from_memory(buffer)?))
    }
}

impl fmt::Debug for DifCodeImage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DifCodeImage({}, {})", self.width(), self.height())
    }
}
impl fmt::Display for DifCodeImage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Image({}, {})", self.width(), self.height())
    }
}
impl PartialEq<DifCodeImage> for DifCodeImage {
    fn eq(&self, other: &DifCodeImage) -> bool {
        if self.width() != other.width() || self.width() != other.width() {
            return false;
        }
        for x in 0..other.width() {
            for y in 0..other.height() {
                if self.get_pixel(x, y) != other.get_pixel(x, y) {
                    return false;
                }
            }
        }
        return true;
    }
}
impl IndexMut<usize> for DifCodeImage {
    fn index_mut(&mut self, index: usize) -> &mut u8 {
        let (x, y, z) = self.index_to_xyz(index);
        &mut self.0.as_mut_rgb8().unwrap().get_pixel_mut(x, y).0[z as usize]
    }
}
impl Index<usize> for DifCodeImage {
    type Output = u8;
    fn index(&self, index: usize) -> &u8 {
        let (x, y, z) = self.index_to_xyz(index);
        &self.0.as_rgb8().unwrap().get_pixel(x, y).0[z as usize]
    }
}
impl EncodingContainer for DifCodeImage {
    fn len(&self) -> usize {
        (self.0.width() * self.0.height() * 3) as usize
    }
}



pub type DifCodeResult<T> = Result<T, DifCodeError>;
#[derive(Debug)]
pub enum DifCodeError {
    Internal(&'static str),
    IO(io::Error),
    IMG(ImageError)
}
impl From<&'static str> for DifCodeError {
    fn from(err: &'static str) -> DifCodeError {
        DifCodeError::Internal(err)
    }
}
impl From<io::Error> for DifCodeError {
    fn from(err: io::Error) -> DifCodeError {
        DifCodeError::IO(err)
    }
}
impl From<ImageError> for DifCodeError {
    fn from(err: ImageError) -> DifCodeError {
        DifCodeError::IMG(err)
    }
}


#[test]
fn test_bytes() {
    let message_bytes = vec![0u8, 1u8, 2u8];
    let original: Vec<u8> = (0..1024).map(|x| x as u8).collect();
    println!("message: {:?}", message_bytes);
    println!("original: {:?}", original);

    let mut selected_indices = randomly_select_indices_within(&message_bytes, &original);

    println!("selected_indices: {:?}", selected_indices);

    let encoded= encode_into_vec(&message_bytes, &original, &mut selected_indices).unwrap();

    println!("encoded: {:?}", encoded);

    let decoded = decode_into_vec(&original, &encoded).unwrap();

    println!("decoded: {:?}", decoded);

    assert_eq!(message_bytes, decoded);
}


#[test]
fn test_image_without_save() {
    let message_bytes: Vec<u8> = (0..64).map(|_| { rand::random::<u8>() }).collect();
    let original_image_path = "test/TestImage.jpg";

    println!("message bytes: {:?}", message_bytes);
    println!("original image path: {}", original_image_path);

    let original_image = DifCodeImage::open(original_image_path).unwrap();

    let encoded_image = encode_into_image(&message_bytes, &original_image,
                                          &randomly_select_indices(get_message_length_in_bits(&message_bytes), original_image.len())
    ).expect("encoding failed");

    let decoded_message = decode_into_vec(&original_image, &encoded_image).unwrap();

    assert_eq!(message_bytes, decoded_message);
}


#[test]
fn test_image_with_save() {
    let message_bytes: Vec<u8> = (0..64).map(|_| { rand::random::<u8>() }).collect();
    let original_image_path = "test/TestImage.jpg";
    let encoded_image_path = "test/TestImage.png";

    println!("message bytes: {:?}", message_bytes);
    println!("original image path: {}", original_image_path);

    let original_image = DifCodeImage::open(original_image_path).unwrap();

    let encoded_image = encode_into_image(&message_bytes, &original_image,
                                &randomly_select_indices(get_message_length_in_bits(&message_bytes), original_image.len())
    ).expect("encoding/saving failed");
    encoded_image.save(encoded_image_path);

    let decoded_message_from_ram_data = decode_into_vec(&original_image, &encoded_image).unwrap();
    assert_eq!(message_bytes, decoded_message_from_ram_data);

    //"proof" of commutativity
    let decoded_message_from_ram_data_commutativity = decode_into_vec(&encoded_image, &original_image).unwrap();
    assert_eq!(message_bytes, decoded_message_from_ram_data_commutativity);


    //time ----


    let original_image_reloaded = DifCodeImage::open(original_image_path).unwrap();
    assert_eq!(original_image, original_image_reloaded);

    let encoded_image_reloaded = DifCodeImage::open(encoded_image_path).unwrap();
    assert_eq!(encoded_image, encoded_image_reloaded);

    let decoded_message_from_disk = decode_into_vec(&original_image_reloaded, &encoded_image_reloaded).unwrap();
    assert_eq!(message_bytes, decoded_message_from_disk);
}

