use anyhow::Result;

use crate::{rlp::Rlp, Error};

pub fn to_bytes(node: &Rlp) -> Result<Vec<u8>> {
    let Rlp::String(value) = node else {
        return Err(Error::RlpError.into());
    };
    Ok(value.clone())
}

pub fn to_string(node: &Rlp) -> Result<String> {
    Ok(format!("0x{}", hex::encode(to_bytes(node)?)))
}

pub fn to_big_int(node: &Rlp) -> Result<usize> {
    let bytes = to_bytes(node)?;
    if bytes.is_empty() {
        return Ok(0);
    }
    let mut value = 0usize;
    for datum in bytes {
        value <<= 8;
        value |= datum as usize;
    }
    Ok(value)
}
