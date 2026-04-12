use alloc::vec::Vec;
use nom::{IResult, Parser};

pub struct ElfFile<'data> {
    pub entrypoint: usize,
    // section header table file offset
    shoff: usize,
    pub phdrs: Vec<ElfPhdr<'data>>,
}

pub struct ElfPhdr<'data> {
    pub type_: PhdrType,
    pub flags: u32,
    pub vaddr: usize,
    pub paddr: usize,
    pub memsz: usize,
    pub align: usize,
    pub data: &'data [u8],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhdrType {
    Load,
    Other(u32),
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

fn parse_phdr<'data>(
    start_input: &'data [u8],
    input: &'data [u8],
) -> IResult<&'data [u8], ElfPhdr<'data>> {
    let (input, type_) = nom::number::le_u32().parse(input)?;
    let type_ = match type_ {
        1 => PhdrType::Load,
        _ => PhdrType::Other(type_),
    };
    let (input, flags) = nom::number::le_u32().parse(input)?;
    let (input, offset) = nom::number::le_u64().parse(input)?;
    let offset = offset as usize;
    let (input, vaddr) = nom::number::le_u64().parse(input)?;
    let vaddr = vaddr as usize;
    let (input, paddr) = nom::number::le_u64().parse(input)?;
    let paddr = paddr as usize;
    let (input, filesz) = nom::number::le_u64().parse(input)?;
    let filesz = filesz as usize;
    let (input, memsz) = nom::number::le_u64().parse(input)?;
    let memsz = memsz as usize;
    let (input, align) = nom::number::le_u64().parse(input)?;
    let align = align as usize;

    Ok((
        input,
        ElfPhdr {
            type_,
            flags,
            vaddr,
            paddr,
            memsz,
            align,
            data: &start_input[offset..(offset + filesz)],
        },
    ))
}

pub fn parse_elf_file(input: &[u8]) -> Result<ElfFile<'_>, nom::Err<nom::error::Error<&[u8]>>> {
    let start_input = input;
    let (input, ident) = parse_ident(input)?;
    let (input, _type) = nom::bytes::complete::take(2usize)(input)?;
    let (input, machine) = nom::number::le_u16().parse(input)?;
    #[cfg(target_arch = "x86_64")]
    const EXPECTED_MACHINE: u16 = 0x3E;
    #[cfg(target_arch = "aarch64")]
    const EXPECTED_MACHINE: u16 = 0xB7;
    assert_eq!(
        machine, EXPECTED_MACHINE,
        "Unsupported machine type in ELF file",
    );

    let (input, _version) = nom::number::le_u32().parse(input)?;

    let (input, entrypoint) = nom::number::le_u64().parse(input)?;
    let entrypoint = entrypoint as usize;
    // program header offset
    let (input, phoff) = nom::number::le_u64().parse(input)?;
    // section header offset
    let (input, shoff) = nom::number::le_u64().parse(input)?;
    let shoff = shoff as usize;
    let (input, _flags) = nom::number::le_u32().parse(input)?;
    let (input, _ehsize) = nom::number::le_u16().parse(input)?;
    let (input, phentsize) = nom::number::le_u16().parse(input)?;
    let (input, phnum) = nom::number::le_u16().parse(input)?;
    let (input, _shentsize) = nom::number::le_u16().parse(input)?;
    let (input, _shnum) = nom::number::le_u16().parse(input)?;
    let (input, _shstrndx) = nom::number::le_u16().parse(input)?;
    let phoff = phoff as usize - (start_input.len() - input.len());
    let (input, _) = nom::bytes::complete::take(phoff)(input)?;

    let parse_phdr = |input| parse_phdr(start_input, input);
    let (input, phdrs) = nom::multi::count(parse_phdr, phnum as usize).parse(input)?;

    Ok(ElfFile {
        entrypoint,
        shoff,
        phdrs,
    })
}
