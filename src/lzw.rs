//! Modified version of https://github.com/nwin/lzw

use crate::{bitstream::BitStream, ParseError};

const MAX_CODESIZE: u8 = 12;
const MAX_ENTRIES: usize = 1 << MAX_CODESIZE as usize;

/// Alias for a LZW code point. It is a 9-bit unsigned integer.
type Code = u16;

// Do not use Option<Code>, to save memory
const CODE_NONE: Code = u16::MAX;

/// Decoding dictionary.
///
/// It is not generic due to current limitations of Rust
/// Inspired by http://www.cplusplus.com/articles/iL18T05o/
#[derive(Debug)]
pub(crate) struct DecodingDict {
    min_size: u8,
    table: heapless::Vec<(Code, u8), 4096>, // FIXME: 4096 is not sufficient for some gifs
    buffer: heapless::Vec<u8, 1024>,
}

impl DecodingDict {
    /// Creates a new dict
    pub fn new(min_size: u8) -> DecodingDict {
        DecodingDict {
            min_size,
            table: heapless::Vec::new(),
            buffer: heapless::Vec::new(),
        }
    }

    /// Resets the dictionary
    pub fn reset(&mut self) {
        self.table.clear();
        for i in 0..(1u16 << self.min_size as usize) {
            self.table.push((CODE_NONE, i as u8)).unwrap();
        }
    }

    /// Inserts a value into the dict
    #[inline(always)]
    pub fn push(&mut self, key: Code, value: u8) {
        self.table.push((key, value)).unwrap(); // TODO: overflow check
    }

    /// Reconstructs the data for the corresponding code
    pub fn reconstruct(&mut self, code: Code) -> Result<&[u8], ParseError> {
        self.buffer.clear();
        let mut code = code;
        let mut cha;
        // Check the first access more thoroughly since a bad code
        // could occur if the data is malformed
        if code != CODE_NONE {
            match self.table.get(code as usize) {
                Some(&(code_, cha_)) => {
                    code = code_;
                    cha = cha_;
                }
                None => {
                    return Err(ParseError::InvalidByte); //
                }
            }
            self.buffer.push(cha).unwrap();
        }
        while code != CODE_NONE {
            if self.buffer.len() >= MAX_ENTRIES {
                return Err(ParseError::InvalidByte); // Invalid code sequence. Cycle in decoding table
            }
            //(code, cha) = self.table[k as usize];
            // Note: This could possibly be replaced with an unchecked array access if
            //  - value is asserted to be < self.next_code() in push
            //  - min_size is asserted to be < MAX_CODESIZE
            let entry = self.table[code as usize];
            code = entry.0;
            cha = entry.1;
            self.buffer.push(cha).unwrap();
        }
        self.buffer.reverse();
        Ok(&self.buffer)
    }

    /// Returns the buffer constructed by the last reconstruction
    #[inline(always)]
    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    /// Number of entries in the dictionary
    #[inline(always)]
    pub fn next_code(&self) -> u16 {
        self.table.len() as u16
    }
}

pub struct Decoder<I: Iterator<Item = u8>> {
    bs: BitStream<I>,
    prev: Code,
    table: DecodingDict,
    buf: [u8; 1],
    code_size: u8,
    min_code_size: u8,
    clear_code: Code,
    end_code: Code,
}

impl<I> Decoder<I>
where
    I: Iterator<Item = u8>,
{
    pub fn new(r: I, min_code_size: u8) -> Decoder<I> {
        let clear_code = 1 << min_code_size;
        let end_code = clear_code + 1;
        let table = DecodingDict::new(min_code_size);
        Decoder {
            bs: BitStream::new(r),
            prev: CODE_NONE,
            table,
            buf: [0],
            code_size: min_code_size + 1,
            min_code_size,
            clear_code,
            end_code,
        }
    }

    pub fn decode_next(&mut self) -> Result<Option<&[u8]>, ParseError> {
        let code = match self.bs.next_bits(self.code_size) {
            Some(code) => code,
            None => return Ok(None), // end of stream
        };

        if code == self.clear_code {
            self.table.reset();
            self.table.push(CODE_NONE, 0); // clear code
            self.table.push(CODE_NONE, 0); // end code
            self.code_size = self.min_code_size + 1;
            self.prev = CODE_NONE;
            Ok(Some(&[]))
        } else if code == self.end_code {
            Ok(Some(&[]))
        } else {
            let next_code = self.table.next_code();
            if code > next_code {
                return Err(ParseError::InvalidByte); // invalid code 9bit, should be LE next_code
            }
            let prev = self.prev;
            let result = if prev == CODE_NONE {
                self.buf = [code as u8];
                &self.buf[..]
            } else {
                if code == next_code {
                    let chr = self.table.reconstruct(prev)?[0];
                    self.table.push(prev, chr);
                    self.table.reconstruct(code)?
                } else if code < next_code {
                    let chr = self.table.reconstruct(code)?[0];
                    self.table.push(prev, chr);
                    self.table.buffer()
                } else {
                    unreachable!("checked above")
                }
            };
            if next_code == (1 << self.code_size as usize) - 1 && self.code_size < MAX_CODESIZE {
                self.code_size += 1;
            }
            self.prev = code;
            Ok(Some(result))
        }
    }
}
