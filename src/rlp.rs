use anyhow::Result;
use bytes::buf::Buf;
use bytes::buf::Reader;
use std::io::BufRead;
use std::io::Read;

use crate::Error;

#[derive(Debug)]
pub enum Rlp {
    String(Vec<u8>),
    List(Vec<Rlp>),
}

pub fn parse_element(reader: &mut Reader<&[u8]>) -> Result<Rlp> {
    let mut tag = [0u8];
    reader.read_exact(&mut tag)?;
    let tag = tag[0];
    match tag {
        0..=0x7E => Ok(Rlp::String(vec![tag])),
        0x7F..=0xB6 => read_string(tag as usize - 0x80, reader),
        0xB7..=0xBE => {
            let length = read_length(tag as usize - 0xB7, reader)?;
            read_string(length, reader)
        }
        0xBF..=0xF6 => {
            let list_length = tag as usize - 0xC0;
            read_list(list_length, reader)
        }
        tag => {
            let length = read_length(tag as usize - 0xF7, reader)?;
            read_list(length, reader)
        }
    }
}

fn read_string(length: usize, reader: &mut Reader<&[u8]>) -> Result<Rlp> {
    if length == 0 {
        return Ok(Rlp::String(vec![]));
    }
    if length >= 1_000_000 {
        return Err(Error::RlpError.into());
    }
    let mut bytes = vec![0u8; length];
    reader.read_exact(&mut bytes)?;

    Ok(Rlp::String(bytes))
}

fn read_list(length: usize, reader: &mut Reader<&[u8]>) -> Result<Rlp> {
    if length >= 1_000_000 {
        return Err(Error::RlpError.into());
    }

    let mut data = vec![0u8; length];
    reader.read_exact(&mut data)?;
    let mut reader = data.reader();

    let mut items = Vec::new();
    while reader.has_data_left()? {
        items.push(parse_element(&mut reader)?);
    }
    Ok(Rlp::List(items))
}

fn read_length(length: usize, reader: &mut Reader<&[u8]>) -> Result<usize> {
    if length >= 1_000_000 {
        return Err(Error::RlpError.into());
    }

    let mut bytes = vec![0u8; length];
    reader.read_exact(&mut bytes)?;

    let mut value = 0usize;
    for datum in bytes {
        value <<= 8;
        value |= datum as usize;
    }

    Ok(value)
}
