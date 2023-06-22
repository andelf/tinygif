use crate::ParseError;

#[inline]
pub fn take1(input: &[u8]) -> Result<(&[u8], u8), ParseError> {
    if let (Some(value), Some(rest)) = (input.get(0), input.get(1..)) {
        Ok((rest, *value))
    } else {
        Err(ParseError::UnexpectedEndOfFile)
    }
}

#[inline]
pub fn take<const N: usize>(input: &[u8]) -> Result<(&[u8], [u8; N]), ParseError> {
    if let (Some(value), Some(rest)) = (input.get(0..N), input.get(N..)) {
        Ok((rest, value.try_into().unwrap()))
    } else {
        Err(ParseError::UnexpectedEndOfFile)
    }
}

#[inline]
pub fn take_slice(input: &[u8], length: usize) -> Result<(&[u8], &[u8]), ParseError> {
    if let (Some(value), Some(rest)) = (input.get(0..length), input.get(length..)) {
        Ok((rest, value))
    } else {
        Err(ParseError::UnexpectedEndOfFile)
    }
}

#[inline]
pub fn le_u16(input: &[u8]) -> Result<(&[u8], u16), ParseError> {
    let (input, value) = take::<2>(input)?;
    Ok((input, u16::from_le_bytes(value)))
}

pub fn eat_len_prefixed_subblocks(input: &[u8]) -> Result<&[u8], ParseError> {
    let mut input0 = input;
    loop {
        let (input, len) = take1(input0)?;
        if len == 0 {
            return Ok(input);
        } else if input.len() < len as usize {
            return Err(ParseError::UnexpectedEndOfFile);
        } else {
            input0 = &input[len as usize..];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eat_len_prefixed_subblocks() {
        let input = b"\x02\x01\x02\x00";
        let input = eat_len_prefixed_subblocks(input).unwrap();
        assert_eq!(input, b"");

        let input = b"\x00\x01\x02\x00";
        let input = eat_len_prefixed_subblocks(input).unwrap();
        assert_eq!(input, b"\x01\x02\x00");
    }
}
