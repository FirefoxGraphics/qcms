/* vim: set ts=8 sw=8 noexpandtab: */
//  qcms
//  Copyright (C) 2009 Mozilla Foundation
//  Copyright (C) 1998-2007 Marti Maria
//
// Permission is hereby granted, free of charge, to any person obtaining
// a copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO
// THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
// LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
// WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

use std::{
    sync::atomic::{AtomicBool, Ordering},
    sync::Arc,
};

use crate::transform::{precache_output, set_rgb_colorants};
use crate::{matrix::Matrix, s15Fixed16Number, s15Fixed16Number_to_float, Intent, Intent::*};

pub static SUPPORTS_ICCV4: AtomicBool = AtomicBool::new(cfg!(feature = "iccv4-enabled"));

pub type icColorSpaceSignature = u32;
pub const icMaxEnumData: icColorSpaceSignature = 4294967295;
pub const icSig15colorData: icColorSpaceSignature = 1178815570;
pub const icSig14colorData: icColorSpaceSignature = 1162038354;
pub const icSig13colorData: icColorSpaceSignature = 1145261138;
pub const icSig12colorData: icColorSpaceSignature = 1128483922;
pub const icSig11colorData: icColorSpaceSignature = 1111706706;
pub const icSig10colorData: icColorSpaceSignature = 1094929490;
pub const icSig9colorData: icColorSpaceSignature = 960711762;
pub const icSig8colorData: icColorSpaceSignature = 943934546;
pub const icSig7colorData: icColorSpaceSignature = 927157330;
pub const icSig6colorData: icColorSpaceSignature = 910380114;
pub const icSig5colorData: icColorSpaceSignature = 893602898;
pub const icSig4colorData: icColorSpaceSignature = 876825682;
pub const icSig3colorData: icColorSpaceSignature = 860048466;
pub const icSig2colorData: icColorSpaceSignature = 843271250;
pub const icSigCmyData: icColorSpaceSignature = 1129142560;
pub const icSigCmykData: icColorSpaceSignature = 1129142603;
pub const icSigHlsData: icColorSpaceSignature = 1212961568;
pub const icSigHsvData: icColorSpaceSignature = 1213421088;
pub const icSigGrayData: icColorSpaceSignature = 1196573017;
pub const icSigRgbData: icColorSpaceSignature = 1380401696;
pub const icSigYxyData: icColorSpaceSignature = 1501067552;
pub const icSigYCbCrData: icColorSpaceSignature = 1497588338;
pub const icSigLuvData: icColorSpaceSignature = 1282766368;
pub const icSigLabData: icColorSpaceSignature = 1281450528;
pub const icSigXYZData: icColorSpaceSignature = 1482250784;

pub const RGB_SIGNATURE: u32 = 0x52474220;
pub const GRAY_SIGNATURE: u32 = 0x47524159;
pub const XYZ_SIGNATURE: u32 = 0x58595A20;
pub const LAB_SIGNATURE: u32 = 0x4C616220;

#[repr(C)]
#[derive(Default)]
pub struct Profile {
    pub(crate) class_type: u32,
    pub(crate) color_space: u32,
    pub(crate) pcs: u32,
    pub(crate) rendering_intent: Intent,
    pub(crate) redColorant: XYZNumber,
    pub(crate) blueColorant: XYZNumber,
    pub(crate) greenColorant: XYZNumber,
    pub(crate) redTRC: Option<Box<curveType>>,
    pub(crate) blueTRC: Option<Box<curveType>>,
    pub(crate) greenTRC: Option<Box<curveType>>,
    pub(crate) grayTRC: Option<Box<curveType>>,
    pub(crate) A2B0: Option<Box<lutType>>,
    pub(crate) B2A0: Option<Box<lutType>>,
    pub(crate) mAB: Option<Box<lutmABType>>,
    pub(crate) mBA: Option<Box<lutmABType>>,
    pub(crate) chromaticAdaption: Matrix,
    pub(crate) output_table_r: Option<Arc<precache_output>>,
    pub(crate) output_table_g: Option<Arc<precache_output>>,
    pub(crate) output_table_b: Option<Arc<precache_output>>,
}

#[derive(Default)]
pub(crate) struct lutmABType {
    pub num_in_channels: u8,
    pub num_out_channels: u8,
    // 16 is the upperbound, actual is 0..num_in_channels.
    pub num_grid_points: [u8; 16],
    pub e00: s15Fixed16Number,
    pub e01: s15Fixed16Number,
    pub e02: s15Fixed16Number,
    pub e03: s15Fixed16Number,
    pub e10: s15Fixed16Number,
    pub e11: s15Fixed16Number,
    pub e12: s15Fixed16Number,
    pub e13: s15Fixed16Number,
    pub e20: s15Fixed16Number,
    pub e21: s15Fixed16Number,
    pub e22: s15Fixed16Number,
    pub e23: s15Fixed16Number,
    // reversed elements (for mBA)
    pub reversed: bool,
    pub clut_table: Option<Vec<f32>>,
    pub a_curves: [Option<Box<curveType>>; 10],
    pub b_curves: [Option<Box<curveType>>; 10],
    pub m_curves: [Option<Box<curveType>>; 10],
}

pub(crate) enum curveType {
    Curve(Vec<uInt16Number>),
    Parametric(Vec<f32>),
}
type uInt16Number = u16;

/* should lut8Type and lut16Type be different types? */
pub(crate) struct lutType {
    // used by lut8Type/lut16Type (mft2) only
    pub num_input_channels: u8,
    pub num_output_channels: u8,
    pub num_clut_grid_points: u8,
    pub e00: s15Fixed16Number,
    pub e01: s15Fixed16Number,
    pub e02: s15Fixed16Number,
    pub e10: s15Fixed16Number,
    pub e11: s15Fixed16Number,
    pub e12: s15Fixed16Number,
    pub e20: s15Fixed16Number,
    pub e21: s15Fixed16Number,
    pub e22: s15Fixed16Number,
    pub num_input_table_entries: u16,
    pub num_output_table_entries: u16,
    pub input_table: Vec<f32>,
    pub clut_table: Vec<f32>,
    pub output_table: Vec<f32>,
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct XYZNumber {
    pub X: s15Fixed16Number,
    pub Y: s15Fixed16Number,
    pub Z: s15Fixed16Number,
}

/* the names for the following two types are sort of ugly */
#[repr(C)]
#[derive(Copy, Clone)]
pub struct qcms_CIE_xyY {
    pub x: f64,
    pub y: f64,
    pub Y: f64,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct qcms_CIE_xyYTRIPLE {
    pub red: qcms_CIE_xyY,
    pub green: qcms_CIE_xyY,
    pub blue: qcms_CIE_xyY,
}

struct tag {
    pub signature: u32,
    pub offset: u32,
    pub size: u32,
}

/* It might be worth having a unified limit on content controlled
 * allocation per profile. This would remove the need for many
 * of the arbitrary limits that we used */
pub type be32 = u32;
pub type be16 = u16;

type tag_index = [tag];

/* a wrapper around the memory that we are going to parse
 * into a qcms_profile */
struct mem_source<'a> {
    pub buf: &'a [u8],
    pub valid: bool,
    pub invalid_reason: Option<&'static str>,
}
pub type uInt8Number = u8;
#[inline]
fn uInt8Number_to_float(a: uInt8Number) -> f32 {
    a as i32 as f32 / 255.0
}

#[inline]
fn uInt16Number_to_float(a: uInt16Number) -> f32 {
    a as i32 as f32 / 65535.0
}

fn cpu_to_be32(v: u32) -> be32 {
    v.to_be()
}
fn cpu_to_be16(v: u16) -> be16 {
    v.to_be()
}
fn be32_to_cpu(v: be32) -> u32 {
    u32::from_be(v)
}
fn be16_to_cpu(v: be16) -> u16 {
    u16::from_be(v)
}
fn invalid_source(mut mem: &mut mem_source, reason: &'static str) {
    mem.valid = false;
    mem.invalid_reason = Some(reason);
}
fn read_u32(mem: &mut mem_source, offset: usize) -> u32 {
    /* Subtract from mem->size instead of the more intuitive adding to offset.
     * This avoids overflowing offset. The subtraction is safe because
     * mem->size is guaranteed to be > 4 */
    if offset > mem.buf.len() - 4 {
        invalid_source(mem, "Invalid offset");
        0
    } else {
        let k = unsafe { std::ptr::read_unaligned(mem.buf.as_ptr().add(offset) as *const be32) };
        be32_to_cpu(k)
    }
}
fn read_u16(mem: &mut mem_source, offset: usize) -> u16 {
    if offset > mem.buf.len() - 2 {
        invalid_source(mem, "Invalid offset");
        0u16
    } else {
        let k = unsafe { std::ptr::read_unaligned(mem.buf.as_ptr().add(offset) as *const be16) };
        be16_to_cpu(k)
    }
}
fn read_u8(mem: &mut mem_source, offset: usize) -> u8 {
    if offset > mem.buf.len() - 1 {
        invalid_source(mem, "Invalid offset");
        0u8
    } else {
        unsafe { *(mem.buf.as_ptr().add(offset) as *mut u8) }
    }
}
fn read_s15Fixed16Number(mem: &mut mem_source, offset: usize) -> s15Fixed16Number {
    read_u32(mem, offset) as s15Fixed16Number
}
fn read_uInt8Number(mem: &mut mem_source, offset: usize) -> uInt8Number {
    read_u8(mem, offset)
}
fn read_uInt16Number(mem: &mut mem_source, offset: usize) -> uInt16Number {
    read_u16(mem, offset)
}
pub fn write_u32(mem: &mut [u8], offset: usize, value: u32) {
    if offset <= mem.len() - std::mem::size_of_val(&value) {
        panic!("OOB");
    }
    let mem = mem.as_mut_ptr();
    unsafe {
        std::ptr::write_unaligned(mem.add(offset) as *mut u32, cpu_to_be32(value));
    }
}
pub fn write_u16(mem: &mut [u8], offset: usize, value: u16) {
    if offset <= mem.len() - std::mem::size_of_val(&value) {
        panic!("OOB");
    }
    let mem = mem.as_mut_ptr();
    unsafe {
        std::ptr::write_unaligned(mem.add(offset) as *mut u16, cpu_to_be16(value));
    }
}

/* An arbitrary 4MB limit on profile size */
pub(crate) const MAX_PROFILE_SIZE: usize = 1024 * 1024 * 4;
const MAX_TAG_COUNT: u32 = 1024;

fn check_CMM_type_signature(_src: &mut mem_source) {
    //uint32_t CMM_type_signature = read_u32(src, 4);
    //TODO: do the check?
}
fn check_profile_version(src: &mut mem_source) {
    /*
    uint8_t major_revision = read_u8(src, 8 + 0);
    uint8_t minor_revision = read_u8(src, 8 + 1);
    */
    let reserved1: u8 = read_u8(src, (8 + 2) as usize);
    let reserved2: u8 = read_u8(src, (8 + 3) as usize);
    /* Checking the version doesn't buy us anything
    if (major_revision != 0x4) {
        if (major_revision > 0x2)
            invalid_source(src, "Unsupported major revision");
        if (minor_revision > 0x40)
            invalid_source(src, "Unsupported minor revision");
    }
    */
    if reserved1 as i32 != 0 || reserved2 as i32 != 0 {
        invalid_source(src, "Invalid reserved bytes");
    };
}

const INPUT_DEVICE_PROFILE: u32 = 0x73636e72; // 'scnr'
pub const DISPLAY_DEVICE_PROFILE: u32 = 0x6d6e7472; // 'mntr'
const OUTPUT_DEVICE_PROFILE: u32 = 0x70727472; // 'prtr'
const DEVICE_LINK_PROFILE: u32 = 0x6c696e6b; // 'link'
const COLOR_SPACE_PROFILE: u32 = 0x73706163; // 'spac'
const ABSTRACT_PROFILE: u32 = 0x61627374; // 'abst'
const NAMED_COLOR_PROFILE: u32 = 0x6e6d636c; // 'nmcl'

fn read_class_signature(mut profile: &mut Profile, mem: &mut mem_source) {
    profile.class_type = read_u32(mem, 12);
    match profile.class_type {
        DISPLAY_DEVICE_PROFILE
        | INPUT_DEVICE_PROFILE
        | OUTPUT_DEVICE_PROFILE
        | COLOR_SPACE_PROFILE => {}
        _ => {
            invalid_source(mem, "Invalid  Profile/Device Class signature");
        }
    };
}
fn read_color_space(mut profile: &mut Profile, mem: &mut mem_source) {
    profile.color_space = read_u32(mem, 16);
    match profile.color_space {
        RGB_SIGNATURE | GRAY_SIGNATURE => {}
        _ => {
            invalid_source(mem, "Unsupported colorspace");
        }
    };
}
fn read_pcs(mut profile: &mut Profile, mem: &mut mem_source) {
    profile.pcs = read_u32(mem, 20);
    match profile.pcs {
        XYZ_SIGNATURE | LAB_SIGNATURE => {}
        _ => {
            invalid_source(mem, "Unsupported pcs");
        }
    };
}
fn read_tag_table(_profile: &mut Profile, mem: &mut mem_source) -> Vec<tag> {
    let count = read_u32(mem, 128);
    if count > MAX_TAG_COUNT {
        invalid_source(mem, "max number of tags exceeded");
        return Vec::new();
    }
    let mut index = Vec::with_capacity(count as usize);
    for i in 0..count {
        index.push(tag {
            signature: read_u32(mem, (128 + 4 + 4 * i * 3) as usize),
            offset: read_u32(mem, (128 + 4 + 4 * i * 3 + 4) as usize),
            size: read_u32(mem, (128 + 4 + 4 * i * 3 + 8) as usize),
        });
    }

    index
}
/* if we've already got an ICC_H header we can ignore the following */
/* icc34 defines */
/* ****************************************************************
 Copyright (c) 1994-1996 SunSoft, Inc.

                    Rights Reserved

Permission is hereby granted, free of charge, to any person
obtaining a copy of this software and associated documentation
files (the "Software"), to deal in the Software without restrict-
ion, including without limitation the rights to use, copy, modify,
merge, publish distribute, sublicense, and/or sell copies of the
Software, and to permit persons to whom the Software is furnished
to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be
included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES
OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NON-
INFRINGEMENT.  IN NO EVENT SHALL SUNSOFT, INC. OR ITS PARENT
COMPANY BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
OTHER DEALINGS IN THE SOFTWARE.

Except as contained in this notice, the name of SunSoft, Inc.
shall not be used in advertising or otherwise to promote the
sale, use or other dealings in this Software without written
authorization from SunSoft Inc.
******************************************************************/
/*
 * QCMS, in general, is not threadsafe. However, it should be safe to create
 * profile and transformation objects on different threads, so long as you
 * don't use the same objects on different threads at the same time.
 */
/*
 * Color Space Signatures
 * Note that only icSigXYZData and icSigLabData are valid
 * Profile Connection Spaces (PCSs)
 */
/* 'XYZ ' */
/* 'Lab ' */
/* 'Luv ' */
/* 'YCbr' */
/* 'Yxy ' */
/* 'RGB ' */
/* 'GRAY' */
/* 'HSV ' */
/* 'HLS ' */
/* 'CMYK' */
/* 'CMY ' */
/* '2CLR' */
/* '3CLR' */
/* '4CLR' */
/* '5CLR' */
/* '6CLR' */
/* '7CLR' */
/* '8CLR' */
/* '9CLR' */
/* 'ACLR' */
/* 'BCLR' */
/* 'CCLR' */
/* 'DCLR' */
/* 'ECLR' */
/* 'FCLR' */
/* these values match the Rendering Intent values from the ICC spec */
/* Chris Murphy (CM consultant) suggests this as a default in the event that we
 * cannot reproduce relative + Black Point Compensation.  BPC brings an
 * unacceptable performance overhead, so we go with perceptual. */
//XXX: I don't really like the _DATA_ prefix

// Checks a profile for obvious inconsistencies and returns
// true if the profile looks bogus and should probably be
// ignored.
#[no_mangle]
pub extern "C" fn qcms_profile_is_bogus(profile: &mut Profile) -> bool {
    let mut sum: [f32; 3] = [0.; 3];
    let mut target: [f32; 3] = [0.; 3];
    let mut tolerance: [f32; 3] = [0.; 3];
    let rX: f32;
    let rY: f32;
    let rZ: f32;
    let gX: f32;
    let gY: f32;
    let gZ: f32;
    let bX: f32;
    let bY: f32;
    let bZ: f32;
    let negative: bool;
    let mut i: u32;
    // We currently only check the bogosity of RGB profiles
    if profile.color_space != RGB_SIGNATURE {
        return false;
    }
    if profile.A2B0.is_some()
        || profile.B2A0.is_some()
        || profile.mAB.is_some()
        || profile.mBA.is_some()
    {
        return false;
    }
    rX = s15Fixed16Number_to_float(profile.redColorant.X);
    rY = s15Fixed16Number_to_float(profile.redColorant.Y);
    rZ = s15Fixed16Number_to_float(profile.redColorant.Z);
    gX = s15Fixed16Number_to_float(profile.greenColorant.X);
    gY = s15Fixed16Number_to_float(profile.greenColorant.Y);
    gZ = s15Fixed16Number_to_float(profile.greenColorant.Z);
    bX = s15Fixed16Number_to_float(profile.blueColorant.X);
    bY = s15Fixed16Number_to_float(profile.blueColorant.Y);
    bZ = s15Fixed16Number_to_float(profile.blueColorant.Z);
    // Sum the values; they should add up to something close to white
    sum[0] = rX + gX + bX;
    sum[1] = rY + gY + bY;
    sum[2] = rZ + gZ + bZ;
    // Build our target vector (see mozilla bug 460629)
    target[0] = 0.96420;
    target[1] = 1.00000;
    target[2] = 0.82491;
    // Our tolerance vector - Recommended by Chris Murphy based on
    // conversion from the LAB space criterion of no more than 3 in any one
    // channel. This is similar to, but slightly more tolerant than Adobe's
    // criterion.
    tolerance[0] = 0.02;
    tolerance[1] = 0.02;
    tolerance[2] = 0.04;
    // Compare with our tolerance
    i = 0;
    while i < 3 {
        if !(sum[i as usize] - tolerance[i as usize] <= target[i as usize]
            && sum[i as usize] + tolerance[i as usize] >= target[i as usize])
        {
            return true;
        }
        i += 1
    }
    if !cfg!(target_os = "macos") {
        negative = (rX < 0.)
            || (rY < 0.)
            || (rZ < 0.)
            || (gX < 0.)
            || (gY < 0.)
            || (gZ < 0.)
            || (bX < 0.)
            || (bY < 0.)
            || (bZ < 0.);
    } else {
        // Chromatic adaption to D50 can result in negative XYZ, but the white
        // point D50 tolerance test has passed. Accept negative values herein.
        // See https://bugzilla.mozilla.org/show_bug.cgi?id=498245#c18 onwards
        // for discussion about whether profile XYZ can or cannot be negative,
        // per the spec. Also the https://bugzil.la/450923 user report.

        // FIXME: allow this relaxation on all ports?
        negative = false; // bogus
    }
    if negative {
        return true;
    }
    // All Good
    false
}

pub const TAG_bXYZ: u32 = 0x6258595a;
pub const TAG_gXYZ: u32 = 0x6758595a;
pub const TAG_rXYZ: u32 = 0x7258595a;
pub const TAG_rTRC: u32 = 0x72545243;
pub const TAG_bTRC: u32 = 0x62545243;
pub const TAG_gTRC: u32 = 0x67545243;
pub const TAG_kTRC: u32 = 0x6b545243;
pub const TAG_A2B0: u32 = 0x41324230;
pub const TAG_B2A0: u32 = 0x42324130;
pub const TAG_CHAD: u32 = 0x63686164;

fn find_tag(index: &tag_index, tag_id: u32) -> Option<&tag> {
    for t in index {
        if t.signature == tag_id {
            return Some(t);
        }
    }
    None
}

pub const XYZ_TYPE: u32 = 0x58595a20; // 'XYZ '
pub const CURVE_TYPE: u32 = 0x63757276; // 'curv'
pub const PARAMETRIC_CURVE_TYPE: u32 = 0x70617261; // 'para'
pub const LUT16_TYPE: u32 = 0x6d667432; // 'mft2'
pub const LUT8_TYPE: u32 = 0x6d667431; // 'mft1'
pub const LUT_MAB_TYPE: u32 = 0x6d414220; // 'mAB '
pub const LUT_MBA_TYPE: u32 = 0x6d424120; // 'mBA '
pub const CHROMATIC_TYPE: u32 = 0x73663332; // 'sf32'

fn read_tag_s15Fixed16ArrayType(src: &mut mem_source, index: &tag_index, tag_id: u32) -> Matrix {
    let tag = find_tag(index, tag_id);
    let mut matrix: Matrix = Matrix {
        m: [[0.; 3]; 3],
        invalid: false,
    };
    if let Some(tag) = tag {
        let mut i: u8;
        let offset: u32 = tag.offset;
        let type_0: u32 = read_u32(src, offset as usize);
        // Check mandatory type signature for s16Fixed16ArrayType
        if type_0 != CHROMATIC_TYPE {
            invalid_source(src, "unexpected type, expected \'sf32\'");
        }
        i = 0u8;
        while (i as i32) < 9 {
            matrix.m[(i as i32 / 3) as usize][(i as i32 % 3) as usize] = s15Fixed16Number_to_float(
                read_s15Fixed16Number(src, (offset + 8 + (i as i32 * 4) as u32) as usize),
            );
            i += 1
        }
        matrix.invalid = false
    } else {
        matrix.invalid = true;
        invalid_source(src, "missing sf32tag");
    }
    matrix
}
fn read_tag_XYZType(src: &mut mem_source, index: &tag_index, tag_id: u32) -> XYZNumber {
    let mut num: XYZNumber = {
        let init = XYZNumber { X: 0, Y: 0, Z: 0 };
        init
    };
    let tag = find_tag(&index, tag_id);
    if let Some(tag) = tag {
        let offset: u32 = tag.offset;
        let type_0: u32 = read_u32(src, offset as usize);
        if type_0 != XYZ_TYPE {
            invalid_source(src, "unexpected type, expected XYZ");
        }
        num.X = read_s15Fixed16Number(src, (offset + 8) as usize);
        num.Y = read_s15Fixed16Number(src, (offset + 12) as usize);
        num.Z = read_s15Fixed16Number(src, (offset + 16) as usize)
    } else {
        invalid_source(src, "missing xyztag");
    }
    num
}
// Read the tag at a given offset rather then the tag_index.
// This method is used when reading mAB tags where nested curveType are
// present that are not part of the tag_index.
fn read_curveType(src: &mut mem_source, offset: u32, len: &mut u32) -> Option<Box<curveType>> {
    const COUNT_TO_LENGTH: [u32; 5] = [1, 3, 4, 5, 7]; //PARAMETRIC_CURVE_TYPE
    let type_0: u32 = read_u32(src, offset as usize);
    let count: u32;
    if type_0 != CURVE_TYPE && type_0 != PARAMETRIC_CURVE_TYPE {
        invalid_source(src, "unexpected type, expected CURV or PARA");
        return None;
    }
    if type_0 == CURVE_TYPE {
        count = read_u32(src, (offset + 8) as usize);
        //arbitrary
        if count > 40000 {
            invalid_source(src, "curve size too large");
            return None;
        }
        let mut table = Vec::with_capacity(count as usize);
        for i in 0..count {
            table.push(read_u16(src, (offset + 12 + i * 2) as usize));
        }
        *len = 12 + count * 2;
        Some(Box::new(curveType::Curve(table)))
    } else {
        count = read_u16(src, (offset + 8) as usize) as u32;
        if count > 4 {
            invalid_source(src, "parametric function type not supported.");
            return None;
        }
        let mut params = Vec::with_capacity(count as usize);
        for i in 0..COUNT_TO_LENGTH[count as usize] {
            params.push(s15Fixed16Number_to_float(read_s15Fixed16Number(
                src,
                (offset + 12 + i * 4) as usize,
            )));
        }
        *len = 12 + COUNT_TO_LENGTH[count as usize] * 4;
        if count == 1 || count == 2 {
            /* we have a type 1 or type 2 function that has a division by 'a' */
            let a: f32 = params[1];
            if a == 0.0 {
                invalid_source(src, "parametricCurve definition causes division by zero");
            }
        }
        Some(Box::new(curveType::Parametric(params)))
    }
}
fn read_tag_curveType(
    src: &mut mem_source,
    index: &tag_index,
    tag_id: u32,
) -> Option<Box<curveType>> {
    let tag = find_tag(index, tag_id);
    if let Some(tag) = tag {
        let mut len: u32 = 0;
        return read_curveType(src, tag.offset, &mut len);
    } else {
        invalid_source(src, "missing curvetag");
    }
    None
}
// arbitrary
fn read_nested_curveType(
    src: &mut mem_source,
    curveArray: &mut [Option<Box<curveType>>; 10],
    num_channels: u8,
    curve_offset: u32,
) {
    let mut channel_offset: u32 = 0;
    let mut i: i32;
    i = 0;
    while i < num_channels as i32 {
        let mut tag_len: u32 = 0;
        curveArray[i as usize] = read_curveType(src, curve_offset + channel_offset, &mut tag_len);
        if curveArray[i as usize].is_none() {
            invalid_source(src, "invalid nested curveType curve");
            break;
        } else {
            channel_offset += tag_len;
            // 4 byte aligned
            if tag_len % 4 != 0 {
                channel_offset += 4 - tag_len % 4
            }
            i += 1
        }
    }
}

/* See section 10.10 for specs */
fn read_tag_lutmABType(src: &mut mem_source, tag: &tag) -> Option<Box<lutmABType>> {
    let offset: u32 = tag.offset;
    let mut a_curve_offset: u32;
    let mut b_curve_offset: u32;
    let mut m_curve_offset: u32;
    let mut matrix_offset: u32;
    let mut clut_offset: u32;
    let mut clut_size: u32 = 1;
    let clut_precision: u8;
    let type_0: u32 = read_u32(src, offset as usize);
    let num_in_channels: u8;
    let num_out_channels: u8;
    let mut lut: Box<lutmABType>;
    let mut i: u32;
    if type_0 != LUT_MAB_TYPE && type_0 != LUT_MBA_TYPE {
        return None;
    }
    num_in_channels = read_u8(src, (offset + 8) as usize);
    num_out_channels = read_u8(src, (offset + 9) as usize);
    if num_in_channels as i32 > 10 || num_out_channels as i32 > 10 {
        return None;
    }
    // We require 3in/out channels since we only support RGB->XYZ (or RGB->LAB)
    // XXX: If we remove this restriction make sure that the number of channels
    //      is less or equal to the maximum number of mAB curves in qcmsint.h
    //      also check for clut_size overflow. Also make sure it's != 0
    if num_in_channels as i32 != 3 || num_out_channels as i32 != 3 {
        return None;
    }
    // some of this data is optional and is denoted by a zero offset
    // we also use this to track their existance
    a_curve_offset = read_u32(src, (offset + 28) as usize);
    clut_offset = read_u32(src, (offset + 24) as usize);
    m_curve_offset = read_u32(src, (offset + 20) as usize);
    matrix_offset = read_u32(src, (offset + 16) as usize);
    b_curve_offset = read_u32(src, (offset + 12) as usize);
    // Convert offsets relative to the tag to relative to the profile
    // preserve zero for optional fields
    if a_curve_offset != 0 {
        a_curve_offset += offset
    }
    if clut_offset != 0 {
        clut_offset += offset
    }
    if m_curve_offset != 0 {
        m_curve_offset += offset
    }
    if matrix_offset != 0 {
        matrix_offset += offset
    }
    if b_curve_offset != 0 {
        b_curve_offset += offset
    }
    if clut_offset != 0 {
        debug_assert!(num_in_channels as i32 == 3);
        // clut_size can not overflow since lg(256^num_in_channels) = 24 bits.
        i = 0;
        while i < num_in_channels as u32 {
            clut_size *= read_u8(src, (clut_offset + i) as usize) as u32;
            if clut_size == 0 {
                invalid_source(src, "bad clut_size");
            }
            i += 1
        }
    } else {
        clut_size = 0
    }
    // 24bits * 3 won't overflow either
    clut_size *= num_out_channels as u32;
    if clut_size > 500000 {
        return None;
    }

    lut = Box::new(lutmABType::default());

    if clut_offset != 0 {
        i = 0;
        while i < num_in_channels as u32 {
            lut.num_grid_points[i as usize] = read_u8(src, (clut_offset + i) as usize);
            if lut.num_grid_points[i as usize] as i32 == 0 {
                invalid_source(src, "bad grid_points");
            }
            i += 1
        }
    }
    // Reverse the processing of transformation elements for mBA type.
    lut.reversed = type_0 == LUT_MBA_TYPE;
    lut.num_in_channels = num_in_channels;
    lut.num_out_channels = num_out_channels;
    if matrix_offset != 0 {
        // read the matrix if we have it
        lut.e00 = read_s15Fixed16Number(src, (matrix_offset + (4 * 0) as u32) as usize); // the caller checks that this doesn't happen
        lut.e01 = read_s15Fixed16Number(src, (matrix_offset + (4 * 1) as u32) as usize);
        lut.e02 = read_s15Fixed16Number(src, (matrix_offset + (4 * 2) as u32) as usize);
        lut.e10 = read_s15Fixed16Number(src, (matrix_offset + (4 * 3) as u32) as usize);
        lut.e11 = read_s15Fixed16Number(src, (matrix_offset + (4 * 4) as u32) as usize);
        lut.e12 = read_s15Fixed16Number(src, (matrix_offset + (4 * 5) as u32) as usize);
        lut.e20 = read_s15Fixed16Number(src, (matrix_offset + (4 * 6) as u32) as usize);
        lut.e21 = read_s15Fixed16Number(src, (matrix_offset + (4 * 7) as u32) as usize);
        lut.e22 = read_s15Fixed16Number(src, (matrix_offset + (4 * 8) as u32) as usize);
        lut.e03 = read_s15Fixed16Number(src, (matrix_offset + (4 * 9) as u32) as usize);
        lut.e13 = read_s15Fixed16Number(src, (matrix_offset + (4 * 10) as u32) as usize);
        lut.e23 = read_s15Fixed16Number(src, (matrix_offset + (4 * 11) as u32) as usize)
    }
    if a_curve_offset != 0 {
        read_nested_curveType(src, &mut lut.a_curves, num_in_channels, a_curve_offset);
    }
    if m_curve_offset != 0 {
        read_nested_curveType(src, &mut lut.m_curves, num_out_channels, m_curve_offset);
    }
    if b_curve_offset != 0 {
        read_nested_curveType(src, &mut lut.b_curves, num_out_channels, b_curve_offset);
    } else {
        invalid_source(src, "B curves required");
    }
    if clut_offset != 0 {
        clut_precision = read_u8(src, (clut_offset + 16) as usize);
        let mut clut_table = Vec::with_capacity(clut_size as usize);
        if clut_precision as i32 == 1 {
            for i in 0..clut_size {
                clut_table.push(uInt8Number_to_float(read_uInt8Number(
                    src,
                    (clut_offset + 20 + i * 1) as usize,
                )));
            }
            lut.clut_table = Some(clut_table);
        } else if clut_precision as i32 == 2 {
            for i in 0..clut_size {
                clut_table.push(uInt16Number_to_float(read_uInt16Number(
                    src,
                    (clut_offset + 20 + i * 2) as usize,
                )));
            }
            lut.clut_table = Some(clut_table);
        } else {
            invalid_source(src, "Invalid clut precision");
        }
    }
    if !src.valid {
        return None;
    }
    Some(lut)
}
fn read_tag_lutType(src: &mut mem_source, tag: &tag) -> Option<Box<lutType>> {
    let offset: u32 = tag.offset;
    let type_0: u32 = read_u32(src, offset as usize);
    let num_input_table_entries: u16;
    let num_output_table_entries: u16;
    let in_chan: u8;
    let grid_points: u8;
    let out_chan: u8;
    let input_offset: u32;
    let clut_offset: u32;
    let output_offset: u32;
    let clut_size: u32;
    let entry_size: usize;
    if type_0 == LUT8_TYPE {
        num_input_table_entries = 256u16;
        num_output_table_entries = 256u16;
        entry_size = 1;
        input_offset = 48
    } else if type_0 == LUT16_TYPE {
        num_input_table_entries = read_u16(src, (offset + 48) as usize);
        num_output_table_entries = read_u16(src, (offset + 50) as usize);

        // these limits come from the spec
        if num_input_table_entries < 2
            || num_input_table_entries > 4096
            || num_output_table_entries < 2
            || num_output_table_entries > 4096
        {
            invalid_source(src, "Bad channel count");
            return None;
        }
        entry_size = 2;
        input_offset = 52
    } else {
        debug_assert!(false);
        invalid_source(src, "Unexpected lut type");
        return None;
    }
    in_chan = read_u8(src, (offset + 8) as usize);
    out_chan = read_u8(src, (offset + 9) as usize);
    grid_points = read_u8(src, (offset + 10) as usize);
    clut_size = (grid_points as f64).powf(in_chan as f64) as u32;
    if clut_size > 500000 {
        invalid_source(src, "CLUT too large");
        return None;
    }
    if clut_size <= 0 {
        invalid_source(src, "CLUT must not be empty.");
        return None;
    }
    if in_chan as i32 != 3 || out_chan as i32 != 3 {
        invalid_source(src, "CLUT only supports RGB");
        return None;
    }

    let e00 = read_s15Fixed16Number(src, (offset + 12) as usize);
    let e01 = read_s15Fixed16Number(src, (offset + 16) as usize);
    let e02 = read_s15Fixed16Number(src, (offset + 20) as usize);
    let e10 = read_s15Fixed16Number(src, (offset + 24) as usize);
    let e11 = read_s15Fixed16Number(src, (offset + 28) as usize);
    let e12 = read_s15Fixed16Number(src, (offset + 32) as usize);
    let e20 = read_s15Fixed16Number(src, (offset + 36) as usize);
    let e21 = read_s15Fixed16Number(src, (offset + 40) as usize);
    let e22 = read_s15Fixed16Number(src, (offset + 44) as usize);

    let mut input_table = Vec::with_capacity((num_input_table_entries * in_chan as u16) as usize);
    for i in 0..(num_input_table_entries * in_chan as u16) {
        if type_0 == LUT8_TYPE {
            input_table.push(uInt8Number_to_float(read_uInt8Number(
                src,
                (offset + input_offset) as usize + i as usize * entry_size,
            )))
        } else {
            input_table.push(uInt16Number_to_float(read_uInt16Number(
                src,
                (offset + input_offset) as usize + i as usize * entry_size,
            )))
        }
    }
    clut_offset = ((offset + input_offset) as usize
        + (num_input_table_entries as i32 * in_chan as i32) as usize * entry_size)
        as u32;

    let mut clut_table = Vec::with_capacity((clut_size * out_chan as u32) as usize);
    for i in (0..clut_size * out_chan as u32).step_by(3) {
        if type_0 == LUT8_TYPE {
            clut_table.push(uInt8Number_to_float(read_uInt8Number(
                src,
                clut_offset as usize + i as usize * entry_size + 0,
            )));
            clut_table.push(uInt8Number_to_float(read_uInt8Number(
                src,
                clut_offset as usize + i as usize * entry_size + 1,
            )));
            clut_table.push(uInt8Number_to_float(read_uInt8Number(
                src,
                clut_offset as usize + i as usize * entry_size + 2,
            )))
        } else {
            clut_table.push(uInt16Number_to_float(read_uInt16Number(
                src,
                clut_offset as usize + i as usize * entry_size + 0,
            )));
            clut_table.push(uInt16Number_to_float(read_uInt16Number(
                src,
                clut_offset as usize + i as usize * entry_size + 2,
            )));
            clut_table.push(uInt16Number_to_float(read_uInt16Number(
                src,
                clut_offset as usize + i as usize * entry_size + 4,
            )))
        }
    }
    output_offset =
        (clut_offset as usize + (clut_size * out_chan as u32) as usize * entry_size) as u32;

    let mut output_table =
        Vec::with_capacity((num_output_table_entries * out_chan as u16) as usize);
    for i in 0..num_output_table_entries as i32 * out_chan as i32 {
        if type_0 == LUT8_TYPE {
            output_table.push(uInt8Number_to_float(read_uInt8Number(
                src,
                output_offset as usize + i as usize * entry_size,
            )))
        } else {
            output_table.push(uInt16Number_to_float(read_uInt16Number(
                src,
                output_offset as usize + i as usize * entry_size,
            )))
        }
    }
    Some(Box::new(lutType {
        num_input_table_entries,
        num_output_table_entries,
        num_input_channels: in_chan,
        num_output_channels: out_chan,
        num_clut_grid_points: grid_points,
        e00,
        e01,
        e02,
        e10,
        e11,
        e12,
        e20,
        e21,
        e22,
        input_table,
        clut_table,
        output_table,
    }))
}
fn read_rendering_intent(mut profile: &mut Profile, src: &mut mem_source) {
    let intent = read_u32(src, 64);
    profile.rendering_intent = match intent {
        x if x == QCMS_INTENT_PERCEPTUAL as u32 => QCMS_INTENT_PERCEPTUAL,
        x if x == QCMS_INTENT_RELATIVE_COLORIMETRIC as u32 => QCMS_INTENT_RELATIVE_COLORIMETRIC,
        x if x == QCMS_INTENT_SATURATION as u32 => QCMS_INTENT_SATURATION,
        x if x == QCMS_INTENT_ABSOLUTE_COLORIMETRIC as u32 => QCMS_INTENT_ABSOLUTE_COLORIMETRIC,
        _ => {
            invalid_source(src, "unknown rendering intent");
            Intent::default()
        }
    };
}
fn profile_create() -> Box<Profile> {
    Box::new(Profile::default())
}
/* build sRGB gamma table */
/* based on cmsBuildParametricGamma() */
fn build_sRGB_gamma_table(num_entries: i32) -> Vec<u16> {
    /* taken from lcms: Build_sRGBGamma() */
    let gamma: f64 = 2.4f64;
    let a: f64 = 1.0f64 / 1.055f64;
    let b: f64 = 0.055f64 / 1.055f64;
    let c: f64 = 1.0f64 / 12.92f64;
    let d: f64 = 0.04045f64;
    let mut table = Vec::with_capacity(num_entries as usize);

    for i in 0..num_entries {
        let x: f64 = i as f64 / (num_entries - 1) as f64;
        let y: f64;
        let mut output: f64;
        // IEC 61966-2.1 (sRGB)
        // Y = (aX + b)^Gamma | X >= d
        // Y = cX             | X < d
        if x >= d {
            let e: f64 = a * x + b;
            if e > 0f64 {
                y = e.powf(gamma)
            } else {
                y = 0f64
            }
        } else {
            y = c * x
        }
        // Saturate -- this could likely move to a separate function
        output = y * 65535.0f64 + 0.5f64;
        if output > 65535.0f64 {
            output = 65535f64
        }
        if output < 0f64 {
            output = 0f64
        }
        table.push(output.floor() as u16);
    }
    table
}
fn curve_from_table(table: &[u16]) -> Box<curveType> {
    Box::new(curveType::Curve(table.to_vec()))
}
pub fn float_to_u8Fixed8Number(a: f32) -> u16 {
    if a > 255.0 + 255.0 / 256f32 {
        0xffffu16
    } else if a < 0.0 {
        0u16
    } else {
        (a * 256.0 + 0.5).floor() as u16
    }
}

fn curve_from_gamma(gamma: f32) -> Box<curveType> {
    Box::new(curveType::Curve(vec![float_to_u8Fixed8Number(gamma)]))
}

/* from lcms: cmsWhitePointFromTemp */
/* tempK must be >= 4000. and <= 25000.
 * Invalid values of tempK will return
 * (x,y,Y) = (-1.0, -1.0, -1.0)
 * similar to argyll: icx_DTEMP2XYZ() */
fn white_point_from_temp(temp_K: i32) -> qcms_CIE_xyY {
    let mut white_point: qcms_CIE_xyY = qcms_CIE_xyY {
        x: 0.,
        y: 0.,
        Y: 0.,
    };
    let x: f64;
    let y: f64;
    let T: f64;
    let T2: f64;
    let T3: f64;
    // double M1, M2;
    // No optimization provided.
    T = temp_K as f64; // Square
    T2 = T * T; // Cube
    T3 = T2 * T;
    // For correlated color temperature (T) between 4000K and 7000K:
    if T >= 4000.0f64 && T <= 7000.0f64 {
        x = -4.6070f64 * (1E9f64 / T3)
            + 2.9678f64 * (1E6f64 / T2)
            + 0.09911f64 * (1E3f64 / T)
            + 0.244063f64
    } else if T > 7000.0f64 && T <= 25000.0f64 {
        x = -2.0064f64 * (1E9f64 / T3)
            + 1.9018f64 * (1E6f64 / T2)
            + 0.24748f64 * (1E3f64 / T)
            + 0.237040f64
    } else {
        // or for correlated color temperature (T) between 7000K and 25000K:
        // Invalid tempK
        white_point.x = -1.0f64;
        white_point.y = -1.0f64;
        white_point.Y = -1.0f64;
        debug_assert!(false, "invalid temp");
        return white_point;
    }
    // Obtain y(x)
    y = -3.000f64 * (x * x) + 2.870f64 * x - 0.275f64;
    // wave factors (not used, but here for futures extensions)
    // M1 = (-1.3515 - 1.7703*x + 5.9114 *y)/(0.0241 + 0.2562*x - 0.7341*y);
    // M2 = (0.0300 - 31.4424*x + 30.0717*y)/(0.0241 + 0.2562*x - 0.7341*y);
    // Fill white_point struct
    white_point.x = x;
    white_point.y = y;
    white_point.Y = 1.0f64;
    white_point
}
#[no_mangle]
pub extern "C" fn qcms_white_point_sRGB() -> qcms_CIE_xyY {
    white_point_from_temp(6504)
}

impl Profile {
    //XXX: it would be nice if we had a way of ensuring
    // everything in a profile was initialized regardless of how it was created
    //XXX: should this also be taking a black_point?
    /* similar to CGColorSpaceCreateCalibratedRGB */
    pub fn new_rgb_with_table(
        white_point: qcms_CIE_xyY,
        primaries: qcms_CIE_xyYTRIPLE,
        table: &[u16],
    ) -> Option<Box<Profile>> {
        let mut profile = profile_create();
        //XXX: should store the whitepoint
        if !set_rgb_colorants(&mut profile, white_point, primaries) {
            return None;
        }
        profile.redTRC = Some(curve_from_table(table));
        profile.blueTRC = Some(curve_from_table(table));
        profile.greenTRC = Some(curve_from_table(table));
        profile.class_type = DISPLAY_DEVICE_PROFILE;
        profile.rendering_intent = QCMS_INTENT_PERCEPTUAL;
        profile.color_space = RGB_SIGNATURE;
        profile.pcs = XYZ_TYPE;
        Some(profile)
    }
    pub fn new_sRGB() -> Box<Profile> {
        let Rec709Primaries = qcms_CIE_xyYTRIPLE {
            red: {
                qcms_CIE_xyY {
                    x: 0.6400f64,
                    y: 0.3300f64,
                    Y: 1.0f64,
                }
            },
            green: {
                qcms_CIE_xyY {
                    x: 0.3000f64,
                    y: 0.6000f64,
                    Y: 1.0f64,
                }
            },
            blue: {
                qcms_CIE_xyY {
                    x: 0.1500f64,
                    y: 0.0600f64,
                    Y: 1.0f64,
                }
            },
        };
        let D65 = qcms_white_point_sRGB();
        let table = build_sRGB_gamma_table(1024);

        Profile::new_rgb_with_table(D65, Rec709Primaries, &table).unwrap()
    }

    pub fn new_gray_with_gamma(gamma: f32) -> Box<Profile> {
        let mut profile = profile_create();

        profile.grayTRC = Some(curve_from_gamma(gamma));
        profile.class_type = DISPLAY_DEVICE_PROFILE;
        profile.rendering_intent = QCMS_INTENT_PERCEPTUAL;
        profile.color_space = GRAY_SIGNATURE;
        profile.pcs = XYZ_TYPE;
        profile
    }

    pub fn new_rgb_with_gamma_set(
        white_point: qcms_CIE_xyY,
        primaries: qcms_CIE_xyYTRIPLE,
        redGamma: f32,
        greenGamma: f32,
        blueGamma: f32,
    ) -> Option<Box<Profile>> {
        let mut profile = profile_create();

        //XXX: should store the whitepoint
        if !set_rgb_colorants(&mut profile, white_point, primaries) {
            return None;
        }
        profile.redTRC = Some(curve_from_gamma(redGamma));
        profile.blueTRC = Some(curve_from_gamma(blueGamma));
        profile.greenTRC = Some(curve_from_gamma(greenGamma));
        profile.class_type = DISPLAY_DEVICE_PROFILE;
        profile.rendering_intent = QCMS_INTENT_PERCEPTUAL;
        profile.color_space = RGB_SIGNATURE;
        profile.pcs = XYZ_TYPE;
        Some(profile)
    }

    pub fn new_from_slice(mem: &[u8]) -> Option<Box<Profile>> {
        let length: u32;
        let mut source: mem_source = mem_source {
            buf: mem,
            valid: false,
            invalid_reason: None,
        };
        let index;
        source.valid = true;
        let mut src: &mut mem_source = &mut source;
        if mem.len() < 4 {
            return None;
        }
        length = read_u32(src, 0);
        if length as usize <= mem.len() {
            // shrink the area that we can read if appropriate
            src.buf = &src.buf[0..length as usize];
        } else {
            return None;
        }
        /* ensure that the profile size is sane so it's easier to reason about */
        if src.buf.len() <= 64 || src.buf.len() >= MAX_PROFILE_SIZE {
            return None;
        }
        let mut profile = profile_create();

        check_CMM_type_signature(src);
        check_profile_version(src);
        read_class_signature(&mut profile, src);
        read_rendering_intent(&mut profile, src);
        read_color_space(&mut profile, src);
        read_pcs(&mut profile, src);
        //TODO read rest of profile stuff
        if !src.valid {
            return None;
        }

        index = read_tag_table(&mut profile, src);
        if !src.valid || index.is_empty() {
            return None;
        }

        if find_tag(&index, TAG_CHAD).is_some() {
            profile.chromaticAdaption = read_tag_s15Fixed16ArrayType(src, &index, TAG_CHAD)
        } else {
            profile.chromaticAdaption.invalid = true //Signal the data is not present
        }

        if profile.class_type == DISPLAY_DEVICE_PROFILE
            || profile.class_type == INPUT_DEVICE_PROFILE
            || profile.class_type == OUTPUT_DEVICE_PROFILE
            || profile.class_type == COLOR_SPACE_PROFILE
        {
            if profile.color_space == RGB_SIGNATURE {
                if let Some(A2B0) = find_tag(&index, TAG_A2B0) {
                    let lut_type = read_u32(src, A2B0.offset as usize);
                    if lut_type == LUT8_TYPE || lut_type == LUT16_TYPE {
                        profile.A2B0 = read_tag_lutType(src, A2B0)
                    } else if lut_type == LUT_MAB_TYPE {
                        profile.mAB = read_tag_lutmABType(src, A2B0)
                    }
                }
                if let Some(B2A0) = find_tag(&index, TAG_B2A0) {
                    let lut_type = read_u32(src, B2A0.offset as usize);
                    if lut_type == LUT8_TYPE || lut_type == LUT16_TYPE {
                        profile.B2A0 = read_tag_lutType(src, B2A0)
                    } else if lut_type == LUT_MBA_TYPE {
                        profile.mBA = read_tag_lutmABType(src, B2A0)
                    }
                }
                if find_tag(&index, TAG_rXYZ).is_some() || !SUPPORTS_ICCV4.load(Ordering::Relaxed) {
                    profile.redColorant = read_tag_XYZType(src, &index, TAG_rXYZ);
                    profile.greenColorant = read_tag_XYZType(src, &index, TAG_gXYZ);
                    profile.blueColorant = read_tag_XYZType(src, &index, TAG_bXYZ)
                }
                if !src.valid {
                    return None;
                }

                if find_tag(&index, TAG_rTRC).is_some() || !SUPPORTS_ICCV4.load(Ordering::Relaxed) {
                    profile.redTRC = read_tag_curveType(src, &index, TAG_rTRC);
                    profile.greenTRC = read_tag_curveType(src, &index, TAG_gTRC);
                    profile.blueTRC = read_tag_curveType(src, &index, TAG_bTRC);
                    if profile.redTRC.is_none()
                        || profile.blueTRC.is_none()
                        || profile.greenTRC.is_none()
                    {
                        return None;
                    }
                }
            } else if profile.color_space == GRAY_SIGNATURE {
                profile.grayTRC = read_tag_curveType(src, &index, TAG_kTRC);
                profile.grayTRC.as_ref()?;
            } else {
                debug_assert!(false, "read_color_space protects against entering here");
                return None;
            }
        } else {
            return None;
        }

        if !src.valid {
            return None;
        }
        Some(profile)
    }
    /// Precomputes the information needed for this profile to be
    /// used as the output profile when constructing a `Transform`.
    pub fn precache_output_transform(&mut self) {
        crate::transform::qcms_profile_precache_output_transform(self);
    }
}
