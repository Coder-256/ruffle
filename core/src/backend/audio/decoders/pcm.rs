use super::{Decoder, Frame, SeekableDecoder};
use std::{
    convert::TryInto as _,
    io::{Cursor, Read},
};

/// Decoder for PCM audio data in a Flash file.
/// Flash exports this when you use the "Raw" compression setting.
/// 8-bit unsigned or 16-bit signed PCM.
pub struct PcmDecoder<R: Read> {
    inner: R,
    sample_rate: u32,
    is_stereo: bool,
    is_16_bit: bool,
}

impl<R: Read> PcmDecoder<R> {
    pub fn new(inner: R, is_stereo: bool, sample_rate: u32, is_16_bit: bool) -> Self {
        PcmDecoder {
            inner,
            is_stereo,
            sample_rate,
            is_16_bit,
        }
    }
}

impl<R: Read> Iterator for PcmDecoder<R> {
    type Item = Frame;

    fn next(&mut self) -> Option<Self::Item> {
        let samples: Box<[i16]> = if self.is_stereo {
            if self.is_16_bit {
                let mut bytes = [0u8; 4];
                self.inner.read_exact(&mut bytes).ok()?;
                let left = i16::from_le_bytes(bytes[0..2].try_into().unwrap());
                let right = i16::from_le_bytes(bytes[2..4].try_into().unwrap());
                Box::new([left, right])
            } else {
                let mut bytes = [0u8; 2];
                self.inner.read_exact(&mut bytes).ok()?;
                let left = (i16::from(bytes[0]) - 127) * 128;
                let right = (i16::from(bytes[1]) - 127) * 128;
                Box::new([left, right])
            }
        } else if self.is_16_bit {
            let mut bytes = [0u8; 2];
            self.inner.read_exact(&mut bytes).ok()?;
            let sample = i16::from_le_bytes(bytes);
            Box::new([sample])
        } else {
            let mut bytes = [0u8];
            self.inner.read_exact(&mut bytes).ok()?;
            let sample = (i16::from(bytes[0]) - 127) * 128;
            Box::new([sample])
        };

        Some(Frame {
            num_channels: if self.is_stereo { 2 } else { 1 },
            sample_rate: self.sample_rate,
            samples,
        })
    }
}

impl<R: AsRef<[u8]>> SeekableDecoder for PcmDecoder<Cursor<R>> {
    fn reset(&mut self) {
        self.inner.set_position(0);
    }

    fn seek_to_sample_frame(&mut self, frame: u32) {
        let num_channels = if self.is_stereo { 2 } else { 1 };
        let pos = u64::from(frame) * num_channels * 2;
        self.inner.set_position(pos);
    }
}
