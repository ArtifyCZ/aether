use alloc::borrow::Cow;
use alloc::ffi::CString;
use alloc::vec::Vec;
use core::ffi::CStr;
use core::fmt::Debug;
use nom::IResult;
use nom::Parser;
use nom::bytes::complete::take;
use nom::multi::many_till;

fn align_up(v: usize, a: usize) -> usize {
    let mask = !(a - 1);
    (v + (a - 1)) & mask
}

pub struct TarHeader<'data> {
    pub name: Cow<'data, CStr>,
    pub mode: &'data [u8],
    pub uid: &'data [u8],
    pub gid: &'data [u8],
    pub size: usize,
    pub mtime: &'data [u8],
    pub checksum: &'data [u8],
    pub type_flag: u8,
    pub file_data: &'data [u8],
}

impl<'data> Debug for TarHeader<'data> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // file_data is intentionally omitted
        f.debug_struct("TarHeader")
            .field("name", &self.name)
            .field("mode", &self.mode)
            .field("uid", &self.uid)
            .field("gid", &self.gid)
            .field("size", &self.size)
            .field("mtime", &self.mtime)
            .field("checksum", &self.checksum)
            .field("type_flag", &self.type_flag)
            .finish()
    }
}

fn parse_name(input: &[u8]) -> IResult<&[u8], Cow<'_, CStr>> {
    let (input, name) = take(100usize)(input)?;
    let name = match CStr::from_bytes_until_nul(name) {
        Ok(name) => Cow::Borrowed(name),
        Err(_) => Cow::Owned(CString::new(name).unwrap()),
    };
    Ok((input, name))
}

fn parse_size(input: &[u8]) -> IResult<&[u8], usize> {
    let (input, bytes) = take(12usize)(input)?;

    // Trim null bytes and spaces to avoid '0' subtraction overflow
    let mut size = 0usize;
    for &digit in bytes {
        if digit == 0 || digit == b' ' {
            continue;
        }
        if digit < b'0' || digit > b'7' {
            break;
        } // Standard TAR octal
        size = (size * 8) + (digit - b'0') as usize;
    }
    Ok((input, size))
}

fn parse_header(input: &[u8]) -> IResult<&[u8], TarHeader<'_>> {
    let start_input = input; // Keep track of the start of the 512-byte block

    let (input, name) = parse_name(input)?;
    let (input, mode) = take(8usize)(input)?;
    let (input, uid) = take(8usize)(input)?;
    let (input, gid) = take(8usize)(input)?;
    let (input, size) = parse_size(input)?;
    let (input, mtime) = take(12usize)(input)?;
    let (input, checksum) = take(8usize)(input)?;
    let (input, type_flag) = take(1usize)(input)?;
    let type_flag = type_flag[0];

    // 1. The header is exactly 512 bytes. Skip the rest of the metadata block.
    // (100 + 8 + 8 + 8 + 12 + 12 + 8 + 1 = 157 bytes parsed so far)
    let (input, _) = take(512usize - (start_input.len() - input.len()))(input)?;

    // 2. Take the actual file content
    let (input, file_data) = take(size)(input)?;

    // 3. File data is padded to the nearest 512-byte boundary.
    // If a file is 10 bytes, it occupies 512 bytes in the archive.
    let padding = if size % 512 == 0 {
        0
    } else {
        512 - (size % 512)
    };
    let (input, _) = take(padding)(input)?;

    Ok((
        input,
        TarHeader {
            name,
            mode,
            uid,
            gid,
            size,
            mtime,
            checksum,
            type_flag,
            file_data,
        },
    ))
}

fn parse_eof(input: &[u8]) -> IResult<&[u8], ()> {
    let (input, _) = nom::bytes::complete::tag(b"\0".as_ref())(input)?;
    Ok((input, ()))
}

pub fn parse_tarball_archive(
    data: &[u8],
) -> Result<Vec<TarHeader<'_>>, nom::Err<nom::error::Error<&[u8]>>> {
    match many_till(parse_header, parse_eof).parse(data) {
        Ok((_, (headers, ()))) => Ok(headers),
        Err(err) => Err(err),
    }
}
