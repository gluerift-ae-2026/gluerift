use anyhow::{Context, Result, bail};
use prost::Message;
use std::io::Read;

const MAX_FRAME: usize = 4096;

pub fn encode_frame<M: Message>(message: &M) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    message.encode_length_delimited(&mut out)?;
    if out.len() > MAX_FRAME {
        bail!("encoded protobuf frame exceeds {MAX_FRAME} bytes")
    }
    Ok(out)
}

pub fn decode_frame<M: Message + Default>(bytes: &[u8]) -> Result<M> {
    if bytes.is_empty() {
        bail!("empty protobuf input")
    }
    if bytes.len() > MAX_FRAME {
        bail!("protobuf input exceeds {MAX_FRAME} bytes")
    }
    let mut cursor = bytes;
    let message =
        M::decode_length_delimited(&mut cursor).context("decode length-delimited protobuf")?;
    if !cursor.is_empty() {
        bail!("trailing bytes or extra protobuf frame")
    }
    Ok(message)
}

pub fn read_stdin() -> Result<Vec<u8>> {
    let mut input = Vec::new();
    std::io::stdin()
        .take((MAX_FRAME + 1) as u64)
        .read_to_end(&mut input)?;
    if input.len() > MAX_FRAME {
        bail!("stdin exceeds {MAX_FRAME} bytes")
    }
    Ok(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pb::E01Carrier;

    #[test]
    fn rejects_concatenated_frames() {
        let one = encode_frame(&E01Carrier { decision: 1 }).unwrap();
        let mut two = one.clone();
        two.extend_from_slice(&one);
        assert!(decode_frame::<E01Carrier>(&two).is_err());
    }
}
