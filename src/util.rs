use std::{fmt, io};
use std::ops::{Index, IndexMut};
use std::path::Path;

use image::{DynamicImage, ImageError, ImageResult, RgbImage, Rgb};
use image::error::{ParameterError, ParameterErrorKind};
use ndarray::{Array3};

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

pub struct DifCodeImage(RgbImage);
impl DifCodeImage {
    pub fn raw(&self) -> &RgbImage { &self.0 }
    pub fn from(orig: RgbImage) -> DifCodeImage { DifCodeImage(orig) }
    pub fn width(&self) -> u32  { self.0.width()  }
    pub fn height(&self) -> u32 { self.0.height() }
    pub fn get_pixel(&self, x: u32, y: u32) -> Rgb<u8> { *self.0.get_pixel(x, y) }
    pub fn get_rgorb(&self, x: u32, y: u32, z: u32) -> u8 { self.0.get_pixel(x, y).0[z as usize] }
    pub fn index_to_xyz(&self, index: usize) -> (u32, u32, u32) {
        DifCodeImage::index_to_xyz_with_wh(index, self.width(), self.height())
    }
    pub fn index_to_xyz_with_wh(index: usize, width: u32, height: u32) -> (u32, u32, u32) {
        let mut index = index as u32;
        let z = index / (width * height);
        index -= z * width * height;
        let y = index / width;
        let x = index % width;
        return (x, y, z)
    }

    pub fn with_capacity(width: u32, height: u32) -> DifCodeImage {
        DifCodeImage(DynamicImage::new_rgb8(width, height).into_rgb())
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
        let image = image::open(path)?;
        Ok(DifCodeImage::from(image.into_rgb()))
    }
    pub fn from_memory(buffer: &[u8]) -> Result<DifCodeImage, ImageError> {
        Ok(DifCodeImage::from(image::load_from_memory(buffer)?.into_rgb()))
    }

    pub fn generate_integral_image_for_rgb(&self) -> IntegralRgbImage {
        let mut integral_image = IntegralRgbImage::zeros(self.width(), self.height());
        for x in 0..self.width() {
            for y in 0..self.height() {
                for z in 0..3 {
                    let real_v = self.get_rgorb(x, y, z);
                    let i_ymm = integral_image.get_ymm_or_0(x, y, z);
                    let i_xmm = integral_image.get_xmm_or_0(x, y, z);
                    let i_xmm_ymm = integral_image.get_xmm_and_ymm_or_0(x, y, z);
                    integral_image[[x, y, z]] = real_v as u32 + i_xmm + i_ymm - i_xmm_ymm;
                }
            }
        }
        integral_image
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
        &mut self.0.get_pixel_mut(x, y).0[z as usize]
    }
}
impl Index<usize> for DifCodeImage {
    type Output = u8;
    fn index(&self, index: usize) -> &u8 {
        let (x, y, z) = self.index_to_xyz(index);
        &self.0.get_pixel(x, y).0[z as usize]
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
    InternalMismatchedContainerSizes,
    /// Contains the number of BITS(!) successfully fit, before capacity was reached
    InternalCapacityReached(usize),
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


trait SubWithMin where Self: Sized {
    fn sub_min(self, min: Self, other: Self) -> Self;
}
impl SubWithMin for u32 {
    fn sub_min(self, min: Self, other: Self) -> Self {
        if self >= other {
            self - other
        } else {
            min
        }
    }
}


pub struct IntegralRgbImage(Array3<u32>);
impl IntegralRgbImage {
    pub fn zeros(w: u32, h: u32) -> IntegralRgbImage {
        let x = Array3::<u32>::zeros([w as usize, h as usize, 3]);
        IntegralRgbImage(x)
    }
    pub fn average_in_radius(&self, x: u32, y: u32, z: u32, radius: u32) -> u8 {
        let x_min = x.sub_min(0, radius);
        let y_min = y.sub_min(0, radius);
        let x_max = (x + radius).min(self.width() - 1);
        let y_max = (y + radius).min(self.height() - 1);

        let total_pixels_considered = (x_max - x_min) * (y_max - y_min);

        (self.calculate_area_sum(x_min, y_min, x_max, y_max, z)
             / total_pixels_considered) as u8
    }

    pub fn get_at_p(&self, p: (u32, u32, u32)) -> u32 {
        self.get_at(p.0, p.1, p.2)
    }
    pub fn get_at(&self, x: u32, y: u32, z: u32) -> u32 {
        self[[x, y, z]]
    }
    pub fn width(&self) -> u32 {
        self.0.shape()[0] as u32
    }
    pub fn height(&self) -> u32 {
        self.0.shape()[1] as u32
    }

    pub fn calculate_area_sum_at_ps(&self, p1: (u32, u32, u32), p2: (u32, u32, u32)) -> u32 {
        if p1.2 != p2.2 {
            panic!("z is unequal in the points");
        }
        self.calculate_area_sum(p1.0, p1.1, p2.0, p2.1, p1.2)
    }
    pub fn calculate_area_sum(&self, p1_x: u32, p1_y: u32, p2_x: u32, p2_y: u32, z: u32) -> u32 {
        if p1_x > p2_x || p1_y > p2_y {
            panic!("p1 > p2");
        }

        let (a_x, a_y) = (p1_x, p1_y);
        let (d_x, d_y) = (p2_x, p2_y);
        let (b_x, b_y) = (d_x, a_y);
        let (c_x, c_y) = (a_x, d_y);

        self.get_at(d_x, d_y, z) +
            self.get_at(a_x, a_y, z) - self.get_at(b_x, b_y, z) - self.get_at(c_x, c_y, z)
    }



    pub fn get_ymm_or_0(&self, x: u32, y: u32, z: u32) -> u32 {
        if y > 0 {
            self[[x, y - 1, z]]
        } else {
            0
        }
    }
    pub fn get_xmm_or_0(&self, x: u32, y: u32, z: u32) -> u32 {
        if x > 0 {
            self[[x - 1, y, z]]
        } else {
            0
        }
    }
    pub fn get_xmm_and_ymm_or_0(&self, x: u32, y: u32, z: u32) -> u32 {
        if x > 0 && y > 0 {
            self[[x - 1, y - 1, z]]
        } else {
            0
        }
    }
}

impl Index<[usize; 3]> for IntegralRgbImage {
    type Output = u32;
    fn index(&self, s: [usize; 3]) -> &u32 {
        self.0.index(s)
    }
}
impl IndexMut<[usize; 3]> for IntegralRgbImage {
    fn index_mut(&mut self, s: [usize; 3]) -> &mut u32 {
        self.0.index_mut(s)
    }
}
impl Index<[u32; 3]> for IntegralRgbImage {
    type Output = u32;
    fn index(&self, s: [u32; 3]) -> &u32 {
        self.0.index([s[0] as usize, s[1] as usize, s[2] as usize])
    }
}
impl IndexMut<[u32; 3]> for IntegralRgbImage {
    fn index_mut(&mut self, s: [u32; 3]) -> &mut u32 {
        self.0.index_mut([s[0] as usize, s[1] as usize, s[2] as usize])
    }
}