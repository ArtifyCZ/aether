use alloc::vec::Vec;
use bitflags::bitflags;
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
    Dynamic(ProgramDynamicSegment),
    Unknown(ProgramUnknownSegment<'data>),
}

pub struct ProgramLoadSegment<'data> {
    pub flags: ProgramSegmentFlags,
    pub vaddr: usize,
    pub paddr: usize,
    pub memsz: usize,
    pub align: usize,
    pub data: &'data [u8], // filesz is implicitly data.len()
}

pub struct ProgramDynamicSegment {
    pub vaddr: usize,
    pub relocations: Vec<RelativeRelocation>,
}

pub struct ProgramUnknownSegment<'data> {
    pub type_val: u32,
    pub flags: ProgramSegmentFlags,
    pub vaddr: usize,
    pub paddr: usize,
    pub memsz: usize,
    pub align: usize,
    pub raw_data: &'data [u8], // filesz is implicitly raw_data.len()
}

#[derive(Debug, Clone, Copy)]
pub struct RelativeRelocation {
    pub offset: usize,
    pub addend: i64,
}

bitflags! {
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct ProgramSegmentFlags: u32 {
        const EXECUTABLE = 0x1;
        const WRITABLE = 0x2;
        const READABLE = 0x4;
    }
}

struct RawPhdr<'data> {
    type_val: u32,
    flags: u32,
    vaddr: usize,
    paddr: usize,
    memsz: usize,
    align: usize,
    data: &'data [u8],
}

fn parse_raw_phdr<'data>(
    start_input: &'data [u8],
    input: &'data [u8],
) -> IResult<&'data [u8], RawPhdr<'data>> {
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

    // Instantly slice the data out of the file
    let data = &start_input[offset..(offset + filesz)];

    Ok((
        input,
        RawPhdr {
            type_val,
            flags,
            vaddr: vaddr as usize,
            paddr: paddr as usize,
            memsz: memsz as usize,
            align: align as usize,
            data,
        },
    ))
}

impl<'data> ProgramSegment<'data> {
    pub fn vaddr(&self) -> usize {
        match self {
            Self::Load(seg) => seg.vaddr,
            Self::Dynamic(seg) => seg.vaddr,
            Self::Unknown(seg) => seg.vaddr,
        }
    }

    pub fn memsz(&self) -> usize {
        match self {
            Self::Load(seg) => seg.memsz,
            Self::Unknown(seg) => seg.memsz,
            Self::Dynamic(_) => 0,
        }
    }

    pub fn flags(&self) -> ProgramSegmentFlags {
        match self {
            Self::Load(seg) => seg.flags,
            Self::Unknown(seg) => seg.flags,
            Self::Dynamic(_) => ProgramSegmentFlags::READABLE | ProgramSegmentFlags::WRITABLE,
        }
    }

    /// Converts a RawPhdr into a strongly-typed semantic segment
    fn from_raw(raw: &RawPhdr<'data>, all_raws: &[RawPhdr<'data>]) -> Self {
        let parsed_flags = ProgramSegmentFlags::from_bits_truncate(raw.flags);

        match raw.type_val {
            1 => ProgramSegment::Load(ProgramLoadSegment {
                flags: parsed_flags,
                vaddr: raw.vaddr,
                paddr: raw.paddr,
                memsz: raw.memsz,
                align: raw.align,
                data: raw.data, // Zero-copy pass-through
            }),
            2 => {
                let mut relocations = Vec::new();
                let mut rela_vaddr = 0;
                let mut rela_sz = 0;
                let mut rela_ent = 0;

                // 1. Read the tags from this segment's own pre-sliced data
                for chunk in raw.data.chunks_exact(16) {
                    let d_tag = i64::from_le_bytes(chunk[0..8].try_into().unwrap());
                    let d_val = u64::from_le_bytes(chunk[8..16].try_into().unwrap());
                    match d_tag {
                        0 => break,
                        7 => rela_vaddr = d_val as usize,
                        8 => rela_sz = d_val as usize,
                        9 => rela_ent = d_val as usize,
                        _ => {}
                    }
                }

                // Resolve the relocations cleanly using the other segments' slices
                if rela_vaddr != 0 && rela_sz != 0 && rela_ent >= 24 {
                    let mut rela_data_slice = None;

                    for other in all_raws {
                        if other.type_val == 1
                            && rela_vaddr >= other.vaddr
                            && rela_vaddr < other.vaddr + other.memsz
                        {
                            let offset_in_slice = rela_vaddr - other.vaddr;

                            // Strict safety check: Don't panic on malformed ELFs!
                            if offset_in_slice + rela_sz <= other.data.len() {
                                rela_data_slice =
                                    Some(&other.data[offset_in_slice..offset_in_slice + rela_sz]);
                            }
                            break;
                        }
                    }

                    if let Some(rela_data) = rela_data_slice {
                        for chunk in rela_data.chunks_exact(rela_ent) {
                            let r_offset =
                                u64::from_le_bytes(chunk[0..8].try_into().unwrap()) as usize;
                            let r_info = u64::from_le_bytes(chunk[8..16].try_into().unwrap());
                            let r_addend = i64::from_le_bytes(chunk[16..24].try_into().unwrap());

                            let r_type = (r_info & 0xffffffff) as u32;
                            if r_type == 8 || r_type == 1027 {
                                relocations.push(RelativeRelocation {
                                    offset: r_offset,
                                    addend: r_addend,
                                });
                            }
                        }
                    }
                }

                ProgramSegment::Dynamic(ProgramDynamicSegment {
                    vaddr: raw.vaddr,
                    relocations,
                })
            }
            _ => ProgramSegment::Unknown(ProgramUnknownSegment {
                type_val: raw.type_val,
                flags: parsed_flags,
                vaddr: raw.vaddr,
                paddr: raw.paddr,
                memsz: raw.memsz,
                align: raw.align,
                raw_data: raw.data,
            }),
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

    // Pass 1: Parse strictly structural data
    let parse_raw_fn = |input| parse_raw_phdr(start_input, input);
    let (_input, raw_phdrs) = nom::multi::count(parse_raw_fn, phnum as usize).parse(input)?;

    // Pass 2: Elevate to safe, semantic memory abstractions
    let segments = raw_phdrs
        .iter()
        .map(|raw| ProgramSegment::from_raw(raw, &raw_phdrs))
        .collect();

    Ok(ElfFile {
        type_,
        entrypoint,
        shoff,
        segments,
    })
}
