use std::{fmt, io};
use std::ops::{Index, IndexMut};
use std::path::Path;

use image::{DynamicImage, GenericImageView, ImageError, ImageResult, Rgba};
use image::error::{ParameterError, ParameterErrorKind};

pub fn get_length_in_bits(message: &[u8]) -> usize {
    message.len()*8
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
    pub fn raw(&self) -> &DynamicImage { &self.0 }
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
