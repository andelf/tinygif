use core::fmt::Debug;
use core::marker::PhantomData;

use embedded_graphics::{
    pixelcolor::{raw::RawU24, Rgb888},
    prelude::RawData,
};

use crate::parser::{le_u16, take, take1, take_slice};

mod bitstream;
pub mod lzw;
mod parser;

pub struct LenPrefixRawDataView<'a> {
    remains: &'a [u8],
    current_block: &'a [u8],
    cursor: u8,
}

impl<'a> LenPrefixRawDataView<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        let len = data[0] as usize;
        Self {
            remains: &data[1 + len..],
            current_block: &data[1..1 + len],
            cursor: 0,
        }
    }

    fn shift_cursor(&mut self) {
        if self.current_block.is_empty() {
            // nop
        } else if self.cursor < self.current_block.len() as u8 - 1 {
            self.cursor += 1;
        } else {
            self.cursor = 0;
            self.shift_next_block();
        }
    }

    // leave cursor untouched
    fn shift_next_block(&mut self) {
        if self.current_block.is_empty() {
            // no more blocks
            return;
        }
        let len = self.remains[0] as usize;
        if len == 0 {
            self.remains = &[];
            self.current_block = &[];
        } else {
            self.current_block = &self.remains[1..1 + len];
            self.remains = &self.remains[1 + len..];
        }
    }
}

impl Iterator for LenPrefixRawDataView<'_> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_block.is_empty() {
            return None;
        }

        let current = self.current_block[self.cursor as usize];
        self.shift_cursor();
        Some(current)
    }
}

#[non_exhaustive]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Version {
    V87a,
    V89a,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Header {
    version: Version,
    pub width: u16,
    pub height: u16,
    has_global_color_table: bool,
    color_resolution: u8, // 3 bits
    // _is_sorted: bool,
    pub bg_color_index: u8,
    // _pixel_aspect_ratio: u8
}

impl Header {
    pub fn parse(input: &[u8]) -> Result<(&[u8], (Header, Option<ColorTable<'_>>)), ParseError> {
        let (input, magic) = take::<3>(input)?;

        if &magic != b"GIF" {
            return Err(ParseError::InvalidFileSignature(magic));
        }

        let (input, ver) = take::<3>(input)?;
        let version = if &ver == b"87a" {
            Version::V87a
        } else if &ver == b"89a" {
            Version::V89a
        } else {
            return Err(ParseError::InvalidFileSignature(magic));
        };

        let (intput, screen_width) = le_u16(input)?;
        let (intput, screen_height) = le_u16(intput)?;

        println!("screen_width: {}", screen_width);
        println!("screen_height: {}", screen_height);

        let (input, flags) = take1(intput)?;
        let has_global_color_table = flags & 0b1000_0000 != 0;
        let global_color_table_size = if has_global_color_table {
            2_usize.pow(((flags & 0b0000_0111) + 1) as u32)
        } else {
            0
        };
        let color_resolution = (flags & 0b0111_0000) >> 4;
        let _is_sorted = flags & 0b0000_1000 != 0;

        let (input, bg_color_index) = take1(input)?;
        let (input, _pixel_aspect_ratio) = take1(input)?;

        let (input, color_table) = if global_color_table_size > 0 {
            // Each color table entry is 3 bytes long
            let (input, table) = take_slice(input, global_color_table_size * 3)?;
            (input, Some(ColorTable::new(table)))
        } else {
            (input, None)
        };

        Ok((
            input,
            (
                Header {
                    version,
                    width: screen_width,
                    height: screen_height,
                    has_global_color_table,
                    color_resolution,
                    bg_color_index,
                },
                color_table,
            ),
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ColorTable<'a> {
    data: &'a [u8],
}

impl<'a> ColorTable<'a> {
    pub(crate) const fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    /// Returns the number of entries.
    pub const fn len(&self) -> usize {
        self.data.len() / 3
    }

    /// Returns a color table entry.
    ///
    /// `None` is returned if `index` is out of bounds.
    pub fn get(&self, index: u8) -> Option<Rgb888> {
        // MSRV: Experiment with slice::as_chunks when it's stabilized

        let offset = index as usize * 3;
        let bytes = self.data.get(offset..offset + 3)?;

        Some(
            RawU24::from_u32((bytes[0] as u32) << 16 | (bytes[1] as u32) << 8 | (bytes[2] as u32))
                .into(),
        )
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct RawGif<'a> {
    /// Image header.
    header: Header,

    global_color_table: Option<ColorTable<'a>>,

    /// Image data.
    image_data: &'a [u8],
}

impl<'a> RawGif<'a> {
    pub fn from_slice(bytes: &'a [u8]) -> Result<Self, ParseError> {
        let (_remaining, (header, color_table)) = Header::parse(bytes)?;

        Ok(Self {
            header,
            global_color_table: color_table,
            image_data: _remaining,
        })
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct GraphicControl {
    pub is_transparent: bool,
    pub transparent_color_index: u8,
    // centisecond
    pub delay_centis: u16,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]

pub struct ImageBlock<'a> {
    pub left: u16,
    pub top: u16,
    pub width: u16,
    pub height: u16,
    pub is_interlaced: bool,
    pub lzw_min_code_size: u8,
    local_color_table: Option<ColorTable<'a>>,
    pub image_data: &'a [u8],
}

impl<'a> ImageBlock<'a> {
    // parse after 0x2c separator
    pub fn parse(input: &'a [u8]) -> Result<(&[u8], Self), ParseError> {
        let (input, left) = le_u16(input)?;
        let (input, top) = le_u16(input)?;
        let (input, width) = le_u16(input)?;
        let (input, height) = le_u16(input)?;
        let (input, flags) = take1(input)?;
        let is_interlaced = flags & 0b0100_0000 != 0;
        let has_local_color_table = flags & 0b1000_0000 != 0;
        let local_color_table_size = if has_local_color_table {
            2_usize.pow(((flags & 0b0000_0111) + 1) as u32)
        } else {
            0
        };

        let (input, local_color_table) = if local_color_table_size > 0 {
            // Each color table entry is 4 bytes long
            let (input, table) = take_slice(input, local_color_table_size * 3)?;
            (input, Some(ColorTable::new(table)))
        } else {
            (input, None)
        };

        let (input, lzw_min_code_size) = take1(input)?;
        //let (input, image_data) = take_slice(input, 0)?;

        let mut input0 = input;
        let mut n = 1;
        loop {
            let (input, block_size) = take1(input0)?;
            if block_size == 0 {
                input0 = input;
                break;
            }
            let (input, _) = take_slice(input, block_size as usize)?;
            n += block_size as usize + 1;
            input0 = input;
        }

        Ok((
            input0,
            Self {
                left,
                top,
                width,
                height,
                is_interlaced,
                lzw_min_code_size,
                local_color_table,
                image_data: &input[..n],
            },
        ))
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum ExtensionBlock<'a> {
    GraphicControl(GraphicControl),
    Comment(&'a [u8]),
    PlainText(&'a [u8]),
    NetscapeApplication { repetitions: u16 },
    Application, // ignore content
    Unknown(u8, &'a [u8]),
}

impl<'a> ExtensionBlock<'a> {
    pub fn parse(input: &'a [u8]) -> Result<(&'a [u8], Self), ParseError> {
        let (input, ext_label) = take1(input)?;
        match ext_label {
            0xff => {
                let (input, block_size_1) = take1(input)?;
                let (input, app_id) = take::<8>(input)?;
                let (input, app_auth_code) = take::<3>(input)?;
                if block_size_1 == 11 && &app_id == b"NETSCAPE" && &app_auth_code == b"2.0" {
                    let (input, block_size_2) = take1(input)?;
                    if block_size_2 == 3 {
                        let (input, always_one) = take1(input)?;
                        if always_one == 1 {
                            let (input, repetitions) = le_u16(input)?;
                            let (input, eob) = take1(input)?;
                            if eob == 0 {
                                return Ok((
                                    input,
                                    ExtensionBlock::NetscapeApplication { repetitions },
                                ));
                            }
                        }
                    }
                }
                let mut input0 = input;
                loop {
                    let (input, block_size_2) = take1(input0)?;
                    if block_size_2 == 0 {
                        input0 = input;
                        break;
                    }
                    let (input, _data) = take_slice(input, block_size_2 as usize)?;
                    input0 = input;
                }
                Ok((input0, ExtensionBlock::Application))
            }
            0xf9 => {
                // Graphic Control Extension
                let (input, _block_size) = take1(input)?;
                let (input, flags) = take1(input)?;
                let is_transparent = flags & 0b0000_0001 != 0;
                let (input, delay_centis) = le_u16(input)?;
                let (input, transparent_color_index) = take1(input)?;
                let (input, block_terminator) = take1(input)?;
                if block_terminator != 0 {
                    return Err(ParseError::InvalidByte);
                }

                Ok((
                    input,
                    ExtensionBlock::GraphicControl(GraphicControl {
                        is_transparent,
                        transparent_color_index,
                        delay_centis,
                    }),
                ))
            }
            _ => unimplemented!(),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Segment<'a> {
    Image(ImageBlock<'a>),
    Extension(ExtensionBlock<'a>),
    // 0x3b
    Trailer,
}

impl<'a> Segment<'a> {
    pub fn parse(input: &'a [u8]) -> Result<(&'a [u8], Self), ParseError> {
        let (input, ext_magic) = take1(input)?;

        if ext_magic == 0x21 {
            let (input, ext) = ExtensionBlock::parse(input)?;
            Ok((input, Segment::Extension(ext)))
        } else if ext_magic == 0x2c {
            // Image Block
            let (input, image_block) = ImageBlock::parse(input)?;
            Ok((input, Segment::Image(image_block)))
        } else if ext_magic == 0x3b {
            if input.is_empty() {
                Ok((input, Segment::Trailer))
            } else {
                Err(ParseError::JunkAfterTrailerByte)
            }
        } else {
            return Err(ParseError::InvalidByte);
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Gif<'a, C> {
    raw_gif: RawGif<'a>,
    color_type: PhantomData<C>,
}

/// Parse error.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum ParseError {
    /// The image uses an unsupported bit depth.
    UnsupportedBpp(u16),

    /// Unexpected end of file.
    UnexpectedEndOfFile,

    /// Invalid file signatures.
    ///
    /// BMP files must start with `BM`.
    InvalidFileSignature([u8; 3]),

    /// Unsupported compression method.
    UnsupportedCompressionMethod(u32),

    /// Unsupported header length.
    UnsupportedHeaderLength(u32),

    /// Unsupported channel masks.
    UnsupportedChannelMasks,

    /// Invalid image dimensions.
    InvalidImageDimensions,

    InvalidByte,

    JunkAfterTrailerByte,
}
