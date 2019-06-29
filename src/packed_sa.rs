use bincode::config as bincode_config;
use bitpacking::{BitPacker, BitPacker4x as Packer};
use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use std::io::Result;

// Little endian of b"SA4x", i.e. Compressed Suffix Array using BitPacker4x.
const MAGIC_CSA4: u32 = 2016690515;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackedSuffixArray {
    magic: u32,
    length: u32,
    data: Vec<u8>,
}

impl PackedSuffixArray {
    pub fn from_sa(mut sa: &[u32]) -> Self {
        assert!(sa.len() as u64 <= std::u32::MAX as u64);

        let magic = MAGIC_CSA4;
        let length = sa.len() as u32;
        let bits = sa_bits(length);
        let u32_chunk_size = Packer::BLOCK_LEN;
        let u8_chunk_size = bits as usize * Packer::BLOCK_LEN / 8;
        let chunk_count = ceiling_div(sa.len(), u32_chunk_size);

        let packer = Packer::new();
        let mut data = Vec::with_capacity(u8_chunk_size * chunk_count);
        let mut buf = vec![0u8; u8_chunk_size];
        while sa.len() >= u32_chunk_size {
            let (chunk, tail) = sa.split_at(u32_chunk_size);
            packer.compress(chunk, &mut buf[..], bits);
            data.extend_from_slice(&buf[..]);
            sa = tail;
        }
        if sa.len() > 0 {
            let mut chunk = vec![0; u32_chunk_size];
            chunk[..sa.len()].copy_from_slice(sa);
            packer.compress(&chunk[..], &mut buf[..], bits);

            let mut tail = buf.len();
            while tail > 0 && buf[tail - 1] == 0 {
                tail -= 1;
            }
            data.extend_from_slice(&buf[..tail]);
        }

        PackedSuffixArray {
            magic,
            length,
            data,
        }
    }

    pub fn into_sa(self) -> Vec<u32> {
        assert_eq!(self.magic, MAGIC_CSA4);
        let bits = sa_bits(self.length);
        let u32_chunk_size = Packer::BLOCK_LEN;
        let u8_chunk_size = bits as usize * Packer::BLOCK_LEN / 8;

        let packer = Packer::new();
        let mut data = &self.data[..];
        let mut remain = self.length as usize;
        let mut sa = Vec::with_capacity(remain);
        let mut buf = vec![0u32; u32_chunk_size];
        while data.len() >= u8_chunk_size {
            let n;
            let (chunk, tail) = data.split_at(u8_chunk_size);
            packer.decompress(chunk, &mut buf[..], bits);
            if tail.len() == 0 {
                n = remain;
            } else {
                n = u32_chunk_size;
            }

            sa.extend_from_slice(&buf[..n]);
            data = tail;
            remain -= n;
        }
        if data.len() > 0 {
            let mut chunk = vec![0; u8_chunk_size];
            chunk[..data.len()].copy_from_slice(data);
            packer.decompress(&chunk[..], &mut buf[..], bits);
            sa.extend_from_slice(&buf[..remain]);
        }

        sa
    }

    pub fn dump<W: Write>(&self, file: W) -> Result<()> {
        let mut cfg = bincode_config();
        cfg.little_endian();
        match cfg.serialize_into(file, self) {
            Ok(_) => Ok(()),
            Err(e) => Err(error_conv(e)),
        }
    }

    pub fn dump_bytes(&self) -> Result<Vec<u8>> {
        let mut cfg = bincode_config();
        cfg.little_endian();
        match cfg.serialize(self) {
            Ok(bytes) => Ok(bytes),
            Err(e) => Err(error_conv(e)),
        }
    }

    pub fn load<R: Read>(file: R) -> Result<Self> {
        let mut cfg = bincode_config();
        cfg.little_endian();
        match cfg.deserialize_from(file) {
            Ok(packed) => Ok(packed),
            Err(e) => Err(error_conv(e)),
        }
    }

    pub fn load_bytes(bytes: &[u8]) -> Result<Self> {
        let mut cfg = bincode_config();
        cfg.little_endian();
        match cfg.deserialize(bytes) {
            Ok(packed) => Ok(packed),
            Err(e) => Err(error_conv(e)),
        }
    }
}

fn sa_bits(length: u32) -> u8 {
    (32 - length.saturating_sub(1).leading_zeros()) as u8
}

fn ceiling_div(x: usize, y: usize) -> usize {
    x / y + usize::from(x % y != 0)
}

fn error_conv(err: bincode::Error) -> std::io::Error {
    use bincode::ErrorKind as BincodeErrorKind;
    use std::io::{Error, ErrorKind as IoErrorKind};

    match *err {
        BincodeErrorKind::Io(e) => e,
        BincodeErrorKind::SizeLimit => {
            Error::new(IoErrorKind::Other, BincodeErrorKind::SizeLimit)
        }
        BincodeErrorKind::Custom(e) => Error::new(IoErrorKind::Other, e),
        e => Error::new(IoErrorKind::InvalidData, e),
    }
}
