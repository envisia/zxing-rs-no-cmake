/*
* Copyright 2024 Axel Waggershauser
*/
// SPDX-License-Identifier: Apache-2.0

#![allow(unused_unsafe)]
#![allow(clippy::useless_transmute)]
#![allow(clippy::redundant_closure_call)]

mod tests;

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
mod bindings {
	include!("bindings.rs");
}

use bindings::*;

use flagset::{flags, FlagSet};
use paste::paste;
use std::ffi::{c_char, c_int, c_uint, c_void, CStr, CString, NulError};
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::mem::transmute;
use std::rc::Rc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
	#[error("{0}")]
	InvalidInput(String),

	#[error("NulError from CString::new")]
	NulError(#[from] NulError),
	//
	// #[error("data store disconnected")]
	// IOError(#[from] std::io::Error),
	// #[error("the data for key `{0}` is not available")]
	// Redaction(String),
	// #[error("invalid header (expected {expected:?}, found {found:?})")]
	// InvalidHeader {
	//     expected: String,
	//     found: String,
	// },
	// #[error("unknown data store error")]
	// Unknown,
}

// see https://github.com/dtolnay/thiserror/issues/62
impl From<std::convert::Infallible> for Error {
	fn from(_: std::convert::Infallible) -> Self {
		unreachable!()
	}
}

fn c2r_str(str: *mut c_char) -> String {
	let mut res = String::new();
	if !str.is_null() {
		unsafe { res = CStr::from_ptr(str).to_string_lossy().to_string() };
		unsafe { ZXing_free(str as *mut c_void) };
	}
	res
}

fn c2r_vec(buf: *mut u8, len: c_int) -> Vec<u8> {
	let mut res = Vec::<u8>::new();
	if !buf.is_null() && len > 0 {
		unsafe { res = std::slice::from_raw_parts(buf, len as usize).to_vec() };
		unsafe { ZXing_free(buf as *mut c_void) };
	}
	res
}

fn last_error() -> Error {
	match unsafe { ZXing_LastErrorMsg().as_mut() } {
		None => panic!("Internal error: ZXing_LastErrorMsg() returned NULL"),
		Some(error) => Error::InvalidInput(c2r_str(error)),
	}
}

macro_rules! last_error_or {
	($expr:expr) => {
		match unsafe { ZXing_LastErrorMsg().as_mut() } {
			None => Ok($expr),
			Some(error) => Err(Error::InvalidInput(c2r_str(error))),
		}
	};
}

macro_rules! make_zxing_enum {
    ($name:ident { $($field:ident),* }) => {
        #[repr(u32)]
        #[derive(Debug, Copy, Clone, PartialEq)]
        pub enum $name {
            $($field = paste! { [<ZXing_ $name _ $field>] },)*
        }
    }
}

macro_rules! make_zxing_flags {
    ($name:ident { $($field:ident),* }) => {
        flags! {
            #[repr(u32)]
            pub enum $name: c_uint {
                $($field = paste! { [<ZXing_ $name _ $field>] },)*
            }
        }
    }
}
#[rustfmt::skip] // workaround for broken #[rustfmt::skip::macros(make_zxing_enum)]
make_zxing_enum!(ImageFormat { Lum, RGB, RGBX });
#[rustfmt::skip]
make_zxing_enum!(ContentType { Text, Binary, Mixed, GS1, ISO15434, UnknownECI });
#[rustfmt::skip]
make_zxing_enum!(Binarizer { LocalAverage, GlobalHistogram, FixedThreshold, BoolCast });
#[rustfmt::skip]
make_zxing_enum!(TextMode { Plain, ECI, HRI, Hex, Escaped });
#[rustfmt::skip]
make_zxing_enum!(EanAddOnSymbol { Ignore, Read, Require });

#[rustfmt::skip]
make_zxing_flags!(BarcodeFormat {
	None, Aztec, Codabar, Code39, Code93, Code128, DataBar, DataBarExpanded, DataMatrix, EAN8, EAN13, ITF,
	MaxiCode, PDF417, QRCode, UPCA, UPCE, MicroQRCode, RMQRCode, DXFilmEdge, LinearCodes, MatrixCodes, Any
});

pub type BarcodeFormats = FlagSet<BarcodeFormat>;

impl Display for BarcodeFormat {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", unsafe { c2r_str(ZXing_BarcodeFormatToString(BarcodeFormats::from(*self).bits())) })
	}
}

impl Display for ContentType {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", unsafe { c2r_str(ZXing_ContentTypeToString(transmute(*self))) })
	}
}

#[derive(Debug, PartialEq)]
struct ImageViewOwner<'a>(*mut ZXing_ImageView, PhantomData<&'a u8>);

impl Drop for ImageViewOwner<'_> {
	fn drop(&mut self) {
		unsafe { ZXing_ImageView_delete(self.0) }
	}
}
#[derive(Debug, Clone, PartialEq)]
pub struct ImageView<'a>(Rc<ImageViewOwner<'a>>);

impl<'a> From<&'a ImageView<'a>> for ImageView<'a> {
	fn from(img: &'a ImageView) -> Self {
		img.clone()
	}
}

impl<'a> ImageView<'a> {
	fn try_into_int<T: TryInto<c_int>>(val: T) -> Result<c_int, Error> {
		val.try_into().map_err(|_| Error::InvalidInput("Could not convert Integer into c_int.".to_string()))
	}

	/// Constructs an ImageView from a raw pointer and the width/height (in pixels)
	/// and row_stride/pix_stride (in bytes).
	///
	/// # Safety
	///
	/// The memory gets accessed inside the c++ library at random places between
	/// `ptr` and `ptr + height * row_stride` or `ptr + width * pix_stride`.
	/// Note that both the stride values could be negative, e.g. if the image
	/// view is rotated.
	pub unsafe fn from_ptr<T: TryInto<c_int>, U: TryInto<c_int>>(
		ptr: *const u8,
		width: T,
		height: T,
		format: ImageFormat,
		row_stride: U,
		pix_stride: U,
	) -> Result<Self, Error> {
		let iv = ZXing_ImageView_new(
			ptr,
			Self::try_into_int(width)?,
			Self::try_into_int(height)?,
			format as ZXing_ImageFormat,
			Self::try_into_int(row_stride)?,
			Self::try_into_int(pix_stride)?,
		);
		if iv.is_null() {
			Err(last_error())
		} else {
			Ok(ImageView(Rc::new(ImageViewOwner(iv, PhantomData))))
		}
	}

	pub fn from_slice<T: TryInto<c_int>>(data: &'a [u8], width: T, height: T, format: ImageFormat) -> Result<Self, Error> {
		unsafe {
			let iv = ZXing_ImageView_new_checked(
				data.as_ptr(),
				data.len() as c_int,
				Self::try_into_int(width)?,
				Self::try_into_int(height)?,
				format as ZXing_ImageFormat,
				0,
				0,
			);
			if iv.is_null() {
				Err(last_error())
			} else {
				Ok(ImageView(Rc::new(ImageViewOwner(iv, PhantomData))))
			}
		}
	}

	pub fn cropped(self, left: i32, top: i32, width: i32, height: i32) -> Self {
		unsafe { ZXing_ImageView_crop((self.0).0, left, top, width, height) }
		self
	}

	pub fn rotated(self, degree: i32) -> Self {
		unsafe { ZXing_ImageView_rotate((self.0).0, degree) }
		self
	}
}

#[cfg(feature = "image")]
use image;

#[cfg(feature = "image")]
impl<'a> From<&'a image::GrayImage> for ImageView<'a> {
	fn from(img: &'a image::GrayImage) -> Self {
		ImageView::from_slice(img.as_ref(), img.width(), img.height(), ImageFormat::Lum).unwrap()
	}
}

#[cfg(feature = "image")]
impl<'a> TryFrom<&'a image::DynamicImage> for ImageView<'a> {
	type Error = Error;

	fn try_from(img: &'a image::DynamicImage) -> Result<Self, Error> {
		let format = match img {
			image::DynamicImage::ImageLuma8(_) => Some(ImageFormat::Lum),
			image::DynamicImage::ImageRgb8(_) => Some(ImageFormat::RGB),
			image::DynamicImage::ImageRgba8(_) => Some(ImageFormat::RGBX),
			_ => None,
		};
		match format {
			Some(format) => Ok(ImageView::from_slice(img.as_bytes(), img.width(), img.height(), format)?),
			None => Err(Error::InvalidInput("Invalid image format (must be either luma8|rgb8|rgba8)".to_string())),
		}
	}
}

pub struct ReaderOptions(*mut ZXing_ReaderOptions);

impl Drop for ReaderOptions {
	fn drop(&mut self) {
		unsafe { ZXing_ReaderOptions_delete(self.0) }
	}
}

impl Default for ReaderOptions {
	fn default() -> Self {
		Self::new()
	}
}

impl AsRef<ReaderOptions> for ReaderOptions {
	fn as_ref(&self) -> &ReaderOptions {
		self
	}
}

macro_rules! property {
	($name:ident, $type:ty) => {
		pub fn $name(self, v: impl Into<$type>) -> Self {
			paste! { unsafe { [<ZXing_ReaderOptions_set $name:camel>](self.0, transmute(v.into())) } };
			self
		}

		paste! {
			pub fn [<set_ $name>](&mut self, v : impl Into<$type>) -> &mut Self {
				unsafe { [<ZXing_ReaderOptions_set $name:camel>](self.0, transmute(v.into())) };
				self
			}

			pub fn [<get_ $name>](&self) -> $type {
				unsafe { transmute([<ZXing_ReaderOptions_get $name:camel>](self.0)) }
			}
		}
	};
}

impl ReaderOptions {
	pub fn new() -> Self {
		unsafe { ReaderOptions(ZXing_ReaderOptions_new()) }
	}

	property!(try_harder, bool);
	property!(try_rotate, bool);
	property!(try_invert, bool);
	property!(try_downscale, bool);
	property!(is_pure, bool);
	property!(return_errors, bool);
	property!(formats, BarcodeFormats);
	property!(text_mode, TextMode);
	property!(binarizer, Binarizer);
	property!(ean_add_on_symbol, EanAddOnSymbol);
	property!(max_number_of_symbols, i32);
	property!(min_line_count, i32);
}

pub struct Barcode(*mut ZXing_Barcode);

impl Drop for Barcode {
	fn drop(&mut self) {
		unsafe { ZXing_Barcode_delete(self.0) }
	}
}

pub type PointI = ZXing_PointI;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Position {
	pub top_left: PointI,
	pub top_right: PointI,
	pub bottom_right: PointI,
	pub bottom_left: PointI,
}

impl Display for PointI {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}x{}", self.x, self.y)
	}
}

impl Display for Position {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", unsafe {
			c2r_str(ZXing_PositionToString(*(self as *const Position as *const ZXing_Position)))
		})
	}
}

macro_rules! getter {
	($r_name:ident, $c_name:ident, $conv:expr, $type:ty) => {
		pub fn $r_name(&self) -> $type {
			paste! { unsafe { $conv([<ZXing_Barcode_ $c_name>](self.0)) } }
		}
	};
}

impl Barcode {
	getter!(is_valid, isValid, transmute, bool);
	getter!(format, format, (|f| BarcodeFormats::new(f).unwrap().into_iter().last().unwrap()), BarcodeFormat);
	getter!(content_type, contentType, transmute, ContentType);
	getter!(text, text, c2r_str, String);
	getter!(ec_level, ecLevel, c2r_str, String);
	getter!(error_message, errorMsg, c2r_str, String);
	getter!(symbology_identifier, symbologyIdentifier, c2r_str, String);
	getter!(position, position, transmute, Position);
	getter!(orientation, orientation, transmute, i32);
	getter!(has_eci, hasECI, transmute, bool);
	getter!(is_inverted, isInverted, transmute, bool);
	getter!(is_mirrored, isMirrored, transmute, bool);
	getter!(line_count, lineCount, transmute, i32);

	pub fn bytes(&self) -> Vec<u8> {
		let mut len: c_int = 0;
		unsafe { c2r_vec(ZXing_Barcode_bytes(self.0, &mut len), len) }
	}
	pub fn bytes_eci(&self) -> Vec<u8> {
		let mut len: c_int = 0;
		unsafe { c2r_vec(ZXing_Barcode_bytesECI(self.0, &mut len), len) }
	}
}

pub fn barcode_formats_from_string(str: impl AsRef<str>) -> Result<BarcodeFormats, Error> {
	let cstr = CString::new(str.as_ref())?;
	let res = unsafe { BarcodeFormats::new_unchecked(ZXing_BarcodeFormatsFromString(cstr.as_ptr())) };
	match res.bits() {
		u32::MAX => last_error_or!(BarcodeFormats::default()),
		0 => Ok(BarcodeFormats::full()),
		_ => Ok(res),
	}
}

pub fn read_barcodes<'a, IV, RO>(image: IV, opts: RO) -> Result<Vec<Barcode>, Error>
where
	IV: TryInto<ImageView<'a>>,
	IV::Error: Into<Error>,
	RO: AsRef<ReaderOptions>,
{
	let iv_: ImageView = image.try_into().map_err(Into::into)?;
	unsafe {
		let results = ZXing_ReadBarcodes((iv_.0).0, opts.as_ref().0);
		if !results.is_null() {
			let size = ZXing_Barcodes_size(results);
			let mut vec = Vec::<Barcode>::with_capacity(size as usize);
			for i in 0..size {
				vec.push(Barcode(ZXing_Barcodes_move(results, i)));
			}
			ZXing_Barcodes_delete(results);
			Ok(vec)
		} else {
			Err(last_error())
		}
	}
}
