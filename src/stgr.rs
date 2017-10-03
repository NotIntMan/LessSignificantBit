use std::iter::Iterator;
use std::iter::IntoIterator;
use image::GenericImage;
use image::Rgba;
use std::string::FromUtf8Error;

pub struct MessageCoder<'a, T>
where T: Iterator<Item = u8> + 'a {
	source: &'a mut T,
	element: Option<u8>,
	bit_index: u8,
}

impl<'a, T> MessageCoder<'a, T>
where T: Iterator<Item = u8> + 'a {
	pub fn new(source: &'a mut T) -> MessageCoder<'a, T> {
		MessageCoder {
			source: source,
			element: None,
			bit_index: 8,
		}
	}
}

#[inline]
fn has(num: u8, bit: u8) -> bool {
	return (num & (1 << bit)) > 0;
}

impl<'a, T> Iterator for MessageCoder<'a, T>
where T: Iterator<Item = u8> + 'a {
	type Item = bool;
	fn next(&mut self) -> Option<Self::Item> {
		if self.bit_index >= 8 {
			self.bit_index = 0;
			self.element = self.source.next();
		}
		self.element.map(|value| {
			let result = has(value, 7 - self.bit_index);
			self.bit_index += 1;
			result
		})
	}
}

struct MessageDecoder<'a,T>
where T: Iterator<Item = bool> + 'a {
	source: &'a mut T
}

impl<'a, T> MessageDecoder<'a, T>
where T: Iterator<Item = bool> + 'a {
	fn new(source: &'a mut T) -> MessageDecoder<'a, T> {
		MessageDecoder {
			source: source,
		}
	}
}

#[inline]
fn bit_to_num(v: bool) -> u8 {
	if v {1} else {0}
}

impl<'a, T> Iterator for MessageDecoder<'a, T>
where T: Iterator<Item = bool> + 'a {
	type Item = u8;
	fn next(&mut self) -> Option<Self::Item> {
		self.source.next()
			.and_then(|left| self.source.next().map(|bitr| bit_to_num(left) * 2 + bit_to_num(bitr)))
			.and_then(|left| self.source.next().map(|bitr| left * 2 + bit_to_num(bitr)))
			.and_then(|left| self.source.next().map(|bitr| left * 2 + bit_to_num(bitr)))
			.and_then(|left| self.source.next().map(|bitr| left * 2 + bit_to_num(bitr)))
			.and_then(|left| self.source.next().map(|bitr| left * 2 + bit_to_num(bitr)))
			.and_then(|left| self.source.next().map(|bitr| left * 2 + bit_to_num(bitr)))
			.and_then(|left| self.source.next().map(|bitr| left * 2 + bit_to_num(bitr)))
	}
}

struct PixelPositionIterator {
	dims: (u32, u32),
	current: (u32, u32),
}

impl PixelPositionIterator {
	fn new<T>(img: &T) -> Self
	where T: GenericImage {
		PixelPositionIterator {
			dims: img.dimensions(),
			current: (0, 0),
		}
	}
}

impl Iterator for PixelPositionIterator {
	type Item = (u32, u32);
	fn next(&mut self) -> Option<Self::Item> {
		if (self.dims.0 == 0) || (self.dims.1 == 0) {
			return None;
		}
		if self.current.0 >= self.dims.0 {
			self.current.0 = 0;
			self.current.1 += 1;
		}
		if self.current.1 >= self.dims.1 {
			return None;
		}
		let result = self.current;
		self.current.0 += 1;
		return Some(result);
	}
}

struct TripleIterator<T> {
	data: [T; 3],
	pos: usize,
}

impl<T> TripleIterator<T> {
	fn new(data: [T; 3]) -> TripleIterator<T> {
		TripleIterator {
			data: data,
			pos: 0,
		}
	}
}

impl<T> Iterator for TripleIterator<T>
where T: Copy {
	type Item = T;
	fn next(&mut self) -> Option<Self::Item> {
		if self.pos < 3 {
			let result = self.data[self.pos];
			self.pos += 1;
			Some(result)
		} else {
			None
		}
	}
}

#[allow(dead_code)]
fn format_pixel(pixel: &Rgba<u8>, x: u32, y: u32) -> String {
	format!("pixel ({}, {}, {}) or [{}, {}, {}] at ({}, {})",
		pixel.data[0],
		pixel.data[1],
		pixel.data[2],
		bit_to_num(has(pixel.data[0], 0)),
		bit_to_num(has(pixel.data[1], 0)),
		bit_to_num(has(pixel.data[2], 0)),
		x,
		y,
	)
}

pub fn read_message<T>(img: &T) -> Result<String, FromUtf8Error>
where T: GenericImage<Pixel = Rgba<u8>> {
	let mut bool_iter = PixelPositionIterator::new(img)
		.flat_map(|(x, y)| {
			let pixel = img.get_pixel(x, y);
			// println!("Readed {}", format_pixel(&pixel, x, y));
			return TripleIterator::new([
				(pixel.data[0] & 1) == 1,
				(pixel.data[1] & 1) == 1,
				(pixel.data[2] & 1) == 1,
			]);
		});
	let vec = MessageDecoder::new(&mut bool_iter)
		.take_while(|v| *v != 0)
		.collect::<Vec<_>>();
	// println!("Readed bytes: {:?}", vec);
	String::from_utf8(vec)
}

fn setbit(num: u8, bit: u8, val: bool) -> u8 {
	let pow = 1 << bit;
	if val {
		num | pow
	} else {
		if has(num, bit) {
			num - pow
		} else {
			num
		}
	}
}

pub fn write_message<T>(img: &mut T, msg: String)
where T: GenericImage<Pixel = Rgba<u8>> {
	let pixel_iter = PixelPositionIterator::new(img);
	let mut vec = Vec::from(msg.into_bytes());
	vec.push(0);
	// println!("Bytes to write: {:?}", vec);
	let mut byte_iter = vec.into_iter();
	let mut bit_iter = MessageCoder::new(&mut byte_iter);
	for (x, y) in pixel_iter {
		let mut pixel = img.get_pixel(x, y);
		let mut need_return = false;
		pixel.data[0] = setbit(pixel.data[0], 0, match bit_iter.next() {
			Some(val) => val,
			None => {
				need_return = true;
				false
			},
		});
		pixel.data[1] = setbit(pixel.data[1], 0, match bit_iter.next() {
			Some(val) => val,
			None => {
				need_return = true;
				false
			},
		});
		pixel.data[2] = setbit(pixel.data[2], 0, match bit_iter.next() {
			Some(val) => val,
			None => {
				need_return = true;
				false
			},
		});
		img.put_pixel(x, y, pixel);
		// println!("Writed {}", format_pixel(&pixel, x, y));
		if need_return {
			return;
		}
	}
}

#[test]
fn num_to_message() {
	let input = [72, 105, 33, 0];
	let mut iter = input.iter().map(|v| *v);
	assert_eq!(
		MessageCoder::new(&mut iter)
		.map(|a| if a {1} else {0})
		.collect::<Vec<_>>(),
		vec!(
			0, 1, 0, 0, 1, 0, 0, 0,
			0, 1, 1, 0, 1, 0, 0, 1,
			0, 0, 1, 0, 0, 0, 0, 1,
			0, 0, 0, 0, 0, 0, 0, 0,
		)
	);
}

#[test]
fn message_to_num() {
	let input = [
			0, 1, 0, 0, 1, 0, 0, 0,
			0, 1, 1, 0, 1, 0, 0, 1,
			0, 0, 1, 0, 0, 0, 0, 1,
			0, 0, 0, 0, 0, 0, 0, 0,
	];
	let mut it = input.into_iter().map(|a| *a == 1);
	assert_eq!(
		MessageDecoder::new(&mut it)
			.collect::<Vec<_>>()
		,
		vec!(72, 105, 33, 0)
	);
}
