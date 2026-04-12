use alloc::vec::Vec;
use nom::{IResult, Parser};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfType {
    Executable,
    SharedObject,
    Other(u16),
}

pub struct ElfFile<'data> {
    pub type_: ElfType,
    pub entrypoint: usize,
    shoff: usize,
    pub segments: Vec<ProgramSegment<'data>>,
}

pub enum ProgramSegment<'data> {
    Load(ProgramLoadSegment<'data>),
    Unknown(ProgramUnknownSegment<'data>),
}

pub struct ProgramLoadSegment<'data> {
    pub flags: ProgramSegmentFlags,
    pub vaddr: usize,
    pub paddr: usize,
    pub memsz: usize,
    pub align: usize,
    pub data: &'data [u8], // Note: The length of this slice is the `filesz`!
}

pub struct ProgramUnknownSegment<'data> {
    pub type_val: u32,
    pub flags: ProgramSegmentFlags,
    pub vaddr: usize,
    pub paddr: usize,
    pub memsz: usize,
    pub align: usize,
    pub raw_data: &'data [u8],
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ProgramSegmentFlags(pub u32);

impl ProgramSegmentFlags {
    pub const EXECUTABLE: Self = Self(0x1);
    pub const WRITABLE: Self = Self(0x2);
    pub const READABLE: Self = Self(0x4);

    #[inline]
    pub fn from_bits_truncate(bits: u32) -> Self {
        Self(bits)
    }

    #[inline]
    pub fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl<'data> ProgramSegment<'data> {
    pub fn vaddr(&self) -> usize {
        match self {
            Self::Load(seg) => seg.vaddr,
            Self::Unknown(seg) => seg.vaddr,
        }
    }

    pub fn memsz(&self) -> usize {
        match self {
            Self::Load(seg) => seg.memsz,
            Self::Unknown(seg) => seg.memsz,
        }
    }

    pub fn flags(&self) -> ProgramSegmentFlags {
        match self {
            Self::Load(seg) => seg.flags,
            Self::Unknown(seg) => seg.flags,
        }
    }
}

fn parse_ident(input: &[u8]) -> IResult<&[u8], ()> {
    let (input, _) = nom::bytes::complete::tag(b"\x7FELF" as &[u8])(input)?;
    let (input, _class) = nom::bytes::complete::take(1usize)(input)?;
    let (input, _data) = nom::bytes::complete::take(1usize)(input)?;
    let (input, _version) = nom::bytes::complete::take(1usize)(input)?;
    let (input, _os_abi) = nom::bytes::complete::take(1usize)(input)?;
    let (input, _abi_version) = nom::bytes::complete::take(1usize)(input)?;
    let (input, _) = nom::bytes::complete::take(7usize)(input)?;
    Ok((input, ()))
}

fn parse_segment<'data>(
    start_input: &'data [u8],
    input: &'data [u8],
) -> IResult<&'data [u8], ProgramSegment<'data>> {
    let (input, type_val) = nom::number::le_u32().parse(input)?;
    let (input, flags) = nom::number::le_u32().parse(input)?;
    let (input, offset) = nom::number::le_u64().parse(input)?;
    let (input, vaddr) = nom::number::le_u64().parse(input)?;
    let (input, paddr) = nom::number::le_u64().parse(input)?;
    let (input, filesz) = nom::number::le_u64().parse(input)?;
    let (input, memsz) = nom::number::le_u64().parse(input)?;
    let (input, align) = nom::number::le_u64().parse(input)?;

    let offset = offset as usize;
    let filesz = filesz as usize;

    // Extract the actual segment data from the file using the offset
    let segment_data = &start_input[offset..(offset + filesz)];
    let parsed_flags = ProgramSegmentFlags::from_bits_truncate(flags);

    let segment = match type_val {
        1 => ProgramSegment::Load(ProgramLoadSegment {
            flags: parsed_flags,
            vaddr: vaddr as usize,
            paddr: paddr as usize,
            memsz: memsz as usize,
            align: align as usize,
            data: segment_data,
        }),
        _ => ProgramSegment::Unknown(ProgramUnknownSegment {
            type_val,
            flags: parsed_flags,
            vaddr: vaddr as usize,
            paddr: paddr as usize,
            memsz: memsz as usize,
            align: align as usize,
            raw_data: segment_data,
        }),
    };

    Ok((input, segment))
}

pub fn parse_elf_file(input: &[u8]) -> Result<ElfFile<'_>, nom::Err<nom::error::Error<&[u8]>>> {
    let start_input = input;
    let (input, _ident) = parse_ident(input)?;

    let (input, type_val) = nom::number::le_u16().parse(input)?;
    let type_ = match type_val {
        2 => ElfType::Executable,
        3 => ElfType::SharedObject,
        _ => ElfType::Other(type_val),
    };

    let (input, machine) = nom::number::le_u16().parse(input)?;
    #[cfg(target_arch = "x86_64")]
    const EXPECTED_MACHINE: u16 = 0x3E;
    #[cfg(target_arch = "aarch64")]
    const EXPECTED_MACHINE: u16 = 0xB7;
    assert_eq!(
        machine, EXPECTED_MACHINE,
        "Unsupported machine type in ELF file"
    );

    let (input, _version) = nom::number::le_u32().parse(input)?;
    let (input, entrypoint) = nom::number::le_u64().parse(input)?;
    let entrypoint = entrypoint as usize;

    let (input, phoff) = nom::number::le_u64().parse(input)?;
    let (input, shoff) = nom::number::le_u64().parse(input)?;
    let shoff = shoff as usize;

    let (input, _flags) = nom::number::le_u32().parse(input)?;
    let (input, _ehsize) = nom::number::le_u16().parse(input)?;
    let (input, _phentsize) = nom::number::le_u16().parse(input)?;
    let (input, phnum) = nom::number::le_u16().parse(input)?;
    let (input, _shentsize) = nom::number::le_u16().parse(input)?;
    let (input, _shnum) = nom::number::le_u16().parse(input)?;
    let (input, _shstrndx) = nom::number::le_u16().parse(input)?;

    let phoff = phoff as usize - (start_input.len() - input.len());
    let (input, _) = nom::bytes::complete::take(phoff)(input)?;

    let parse_segment_fn = |input| parse_segment(start_input, input);
    let (_input, segments) = nom::multi::count(parse_segment_fn, phnum as usize).parse(input)?;

    Ok(ElfFile {
        type_,
        entrypoint,
        shoff,
        segments,
    })
}
