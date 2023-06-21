//! Read different number of bits from a byte stream

pub struct BitStream<I: Iterator<Item = u8>> {
    r: I,
    byte: u8,
    // current bit start pos. LSB fist
    bit_cursor: u8,
}

impl<I> BitStream<I>
where
    I: Iterator<Item = u8>,
{
    pub fn new(r: I) -> Self {
        Self {
            r,
            byte: 0,
            bit_cursor: 8, // point to the LSB of the next byte
        }
    }

    pub fn next_bits(&mut self, nbit: u8) -> Option<u16> {
        if nbit >= 16 {
            panic!("nbit must be < 16");
        }
        if self.bit_cursor == 8 {
            self.byte = self.r.next()?;
            self.bit_cursor = 0;
        }
        let mut res = (self.byte >> self.bit_cursor) as u16;
        let mut bits_fullfilled = 8 - self.bit_cursor;

        if bits_fullfilled >= nbit {
            self.bit_cursor += nbit;
            return Some(res & ((1u16 << nbit) - 1));
        }

        while bits_fullfilled < nbit {
            self.byte = self.r.next()?;
            res |= (self.byte as u16) << bits_fullfilled;
            bits_fullfilled += 8;
        }

        self.bit_cursor = nbit - (bits_fullfilled - 8);
        assert!(self.bit_cursor <= 8);
        Some(res & ((1u16 << nbit) - 1))
    }
}
