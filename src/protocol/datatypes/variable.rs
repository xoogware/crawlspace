/*
 * Copyright (c) 2024 Andrew Brower.
 * This file is part of Crawlspace.
 *
 * Crawlspace is free software: you can redistribute it and/or
 * modify it under the terms of the GNU Affero General Public
 * License as published by the Free Software Foundation, either
 * version 3 of the License, or (at your option) any later version.
 *
 * Crawlspace is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
 * Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public
 * License along with Crawlspace. If not, see
 * <https://www.gnu.org/licenses/>.
 */

use std::fmt::Display;
use std::io::Write;

use byteorder::ReadBytesExt;
use color_eyre::eyre::Result;
use serde::Deserialize;

use crate::protocol::{Decode, Encode};

#[derive(thiserror::Error, Debug)]
pub enum VariableDecodeError {
    #[error("VarNum exceeds 32 bits")]
    TooLong,
    #[error("VarNum incomplete")]
    Incomplete,
}

pub trait VariableNumber<'a>: Sized + Encode + Decode<'a> {
    const SEGMENT_BITS: u8 = 0b01111111;
    const CONTINUE_BITS: u8 = 0b10000000;

    const MAX_BYTES: usize;

    fn len(self) -> usize;
}

macro_rules! make_var_num {
    ($name: ident, $type: ty, $max_bytes: expr) => {
        #[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub $type);

        impl VariableNumber<'_> for $name {
            const MAX_BYTES: usize = $max_bytes;

            fn len(self) -> usize {
                match self.0 {
                    0 => 1,
                    n => (31 - n.leading_zeros() as usize) / 7 + 1,
                }
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl Decode<'_> for $name {
            fn decode(r: &mut &[u8]) -> Result<Self> {
                let mut v: $type = 0;

                for i in 0..Self::MAX_BYTES {
                    let byte = r.read_u8().map_err(|_| VariableDecodeError::Incomplete)?;
                    v |= <$type>::from(byte & Self::SEGMENT_BITS) << (i * 7);
                    if byte & Self::CONTINUE_BITS == 0 {
                        return Ok(Self(v));
                    }
                }

                if r.len() > 0 {
                    Err(VariableDecodeError::TooLong)?;
                }

                Err(VariableDecodeError::Incomplete)?
            }
        }
    };
}

make_var_num!(VarInt, i32, 5);
make_var_num!(VarLong, i64, 10);

impl Encode for VarInt {
    // implementation taken from https://github.com/as-com/varint-simd/blob/0f468783da8e181929b01b9c6e9f741c1fe09825/src/encode/mod.rs#L71
    // only the first branch is done here because we never need to change varint size
    fn encode(&self, mut w: impl Write) -> Result<()> {
        let x = self.0 as u64;
        let stage1 = (x & 0x000000000000007f)
            | ((x & 0x0000000000003f80) << 1)
            | ((x & 0x00000000001fc000) << 2)
            | ((x & 0x000000000fe00000) << 3)
            | ((x & 0x00000000f0000000) << 4);

        let leading = stage1.leading_zeros();

        let unused_bytes = (leading - 1) / 8;
        let bytes_needed = 8 - unused_bytes;

        let msbs = 0x8080808080808080;
        let msbmask = 0xFFFFFFFFFFFFFFFF >> ((8 - bytes_needed + 1) * 8 - 1);

        let merged = stage1 | (msbs & msbmask);
        let bytes = merged.to_le_bytes();

        Ok(w.write_all(unsafe { bytes.get_unchecked(..bytes_needed as usize) })?)
    }
}

impl VarLong {
    // how cute...
    #[inline(always)]
    #[cfg(target_feature = "bmi2")]
    fn num_to_vector_stage1(self) -> [u8; 16] {
        use std::arch::x86_64::*;
        let mut res = [0u64; 2];

        let x = self.0 as u64;

        res[0] = unsafe { _pdep_u64(x, 0x7f7f7f7f7f7f7f7f) };
        res[1] = unsafe { _pdep_u64(x >> 56, 0x000000000000017f) };

        unsafe { core::mem::transmute(res) }
    }

    #[inline(always)]
    #[cfg(all(target_feature = "avx2", not(all(target_feature = "bmi2"))))]
    fn num_to_vector_stage1(self) -> [u8; 16] {
        use std::arch::x86_64::*;
        let mut res = [0u64; 2];
        let x = self;

        let b = unsafe { _mm_set1_epi64x(self as i64) };
        let c = unsafe {
            _mm_or_si128(
                _mm_or_si128(
                    _mm_sllv_epi64(
                        _mm_and_si128(b, _mm_set_epi64x(0x00000007f0000000, 0x000003f800000000)),
                        _mm_set_epi64x(4, 5),
                    ),
                    _mm_sllv_epi64(
                        _mm_and_si128(b, _mm_set_epi64x(0x0001fc0000000000, 0x00fe000000000000)),
                        _mm_set_epi64x(6, 7),
                    ),
                ),
                _mm_or_si128(
                    _mm_sllv_epi64(
                        _mm_and_si128(b, _mm_set_epi64x(0x000000000000007f, 0x0000000000003f80)),
                        _mm_set_epi64x(0, 1),
                    ),
                    _mm_sllv_epi64(
                        _mm_and_si128(b, _mm_set_epi64x(0x00000000001fc000, 0x000000000fe00000)),
                        _mm_set_epi64x(2, 3),
                    ),
                ),
            )
        };
        let d = unsafe { _mm_or_si128(c, _mm_bsrli_si128(c, 8)) };

        res[0] = unsafe { _mm_extract_epi64(d, 0) as u64 };
        res[1] = ((x & 0x7f00000000000000) >> 56) | ((x & 0x8000000000000000) >> 55);

        unsafe { core::mem::transmute(res) }
    }

    // TODO: need to confirm this works. for now it's just a naive translation of avx2,
    // but could definitely be improved -- blocking NEON implementation of Encode
    //
    // #[inline(always)]
    // #[cfg(target_feature = "neon")]
    // fn num_to_vector_stage1(self) -> [u8; 16] {
    //     use std::arch::aarch64::*;
    //
    //     let mut res = [0u64; 2];
    //     let x = self;
    //
    //     let b = unsafe { vdupq_n_s64(self.0 as i64) };
    //     let c = unsafe {
    //         vorrq_s64(
    //             vorrq_s64(
    //                 vshlq_s64(
    //                     vandq_s64(
    //                         b,
    //                         vcombine_s64(
    //                             vcreate_s64(0x000003f800000000),
    //                             vcreate_s64(0x00000007f0000000),
    //                         ),
    //                     ),
    //                     vcombine_s64(vcreate_s64(5), vcreate_s64(4)),
    //                 ),
    //                 vshlq_s64(
    //                     vandq_s64(
    //                         b,
    //                         vcombine_s64(
    //                             vcreate_s64(0x00fe000000000000),
    //                             vcreate_s64(0x0001fc0000000000),
    //                         ),
    //                     ),
    //                     vcombine_s64(vcreate_s64(7), vcreate_s64(6)),
    //                 ),
    //             ),
    //             vorrq_s64(
    //                 vshlq_s64(
    //                     vandq_s64(
    //                         b,
    //                         vcombine_s64(
    //                             vcreate_s64(0x0000000000003f80),
    //                             vcreate_s64(0x000000000000007f),
    //                         ),
    //                     ),
    //                     vcombine_s64(vcreate_s64(1), vcreate_s64(0)),
    //                 ),
    //                 vshlq_s64(
    //                     vandq_s64(
    //                         b,
    //                         vcombine_s64(
    //                             vcreate_s64(0x000000000fe00000),
    //                             vcreate_s64(0x00000000001fc000),
    //                         ),
    //                     ),
    //                     vcombine_s64(vcreate_s64(3), vcreate_s64(2)),
    //                 ),
    //             ),
    //         )
    //     };
    //     let d = unsafe { vorrq_s64(c, vshrq_n_s64::<8>(c)) };
    //
    //     res[0] = unsafe { vgetq_lane_s64(d, 0) as u64 };
    //     res[1] =
    //         ((x.0 as u64 & 0x7f00000000000000) >> 56) | ((x.0 as u64 & 0x8000000000000000) >> 55);
    //
    //     unsafe { core::mem::transmute(res) }
    // }
}

impl Encode for VarLong {
    // ...and here's the second branch ^_^
    #[cfg(any(target_feature = "bmi2", target_feature = "avx2"))]
    fn encode(&self, mut w: impl Write) -> Result<()> {
        use std::arch::x86_64::*;
        unsafe {
            // Break the number into 7-bit parts and spread them out into a vector
            let stage1: __m128i = std::mem::transmute(self.num_to_vector_stage1());

            // Create a mask for where there exist values
            // This signed comparison works because all MSBs should be cleared at this point
            // Also handle the special case when num == 0
            let minimum = _mm_set_epi8(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xffu8 as i8);
            let exists = _mm_or_si128(_mm_cmpgt_epi8(stage1, _mm_setzero_si128()), minimum);
            let bits = _mm_movemask_epi8(exists);

            // Count the number of bytes used
            let bytes = 32 - bits.leading_zeros() as u8; // lzcnt on supported CPUs

            // Fill that many bytes into a vector
            let ascend = _mm_setr_epi8(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
            let mask = _mm_cmplt_epi8(ascend, _mm_set1_epi8(bytes as i8));

            // Shift it down 1 byte so the last MSB is the only one set, and make sure only the MSB is set
            let shift = _mm_bsrli_si128(mask, 1);
            let msbmask = _mm_and_si128(shift, _mm_set1_epi8(128u8 as i8));

            // Merge the MSB bits into the vector
            let merged = _mm_or_si128(stage1, msbmask);

            Ok(w.write_all(
                std::mem::transmute::<__m128i, [u8; 16]>(merged).get_unchecked(..bytes as usize),
            )?)
        }
    }

    // TODO: implement this using neon? not likely we'll use arm-based servers but maybe nice for
    // local testing?
    #[cfg(not(any(target_feature = "bmi2", target_feature = "avx2")))]
    fn encode(&self, mut w: impl Write) -> Result<()> {
        use byteorder::WriteBytesExt;

        let mut val = self.0 as u64;
        loop {
            if val & 0b1111111111111111111111111111111111111111111111111111111110000000 == 0 {
                w.write_u8(val as u8)?;
                return Ok(());
            }
            w.write_u8(val as u8 & 0b01111111 | 0b10000000)?;
            val >>= 7;
        }
    }
}
