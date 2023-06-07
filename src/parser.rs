use crate::ParseError;

pub fn take1(input: &[u8]) -> Result<(&[u8], u8), ParseError> {
    if let (Some(value), Some(rest)) = (input.get(0), input.get(1..)) {
        Ok((rest, *value))
    } else {
        Err(ParseError::UnexpectedEndOfFile)
    }
}

pub fn take<const N: usize>(input: &[u8]) -> Result<(&[u8], [u8; N]), ParseError> {
    if let (Some(value), Some(rest)) = (input.get(0..N), input.get(N..)) {
        Ok((rest, value.try_into().unwrap()))
    } else {
        Err(ParseError::UnexpectedEndOfFile)
    }
}

pub fn take_slice(input: &[u8], length: usize) -> Result<(&[u8], &[u8]), ParseError> {
    if let (Some(value), Some(rest)) = (input.get(0..length), input.get(length..)) {
        Ok((rest, value))
    } else {
        Err(ParseError::UnexpectedEndOfFile)
    }
}

pub fn le_u16(input: &[u8]) -> Result<(&[u8], u16), ParseError> {
    let (input, value) = take::<2>(input)?;
    Ok((input, u16::from_le_bytes(value)))
}
