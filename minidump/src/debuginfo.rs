use goblin;
use goblin::{elf, mach, pe};
use std::io::Cursor;

use errors::Result;

#[derive(Debug)]
pub struct ElfFile<'a> {
    inner: elf::Elf<'a>,
    data: &'a [u8],
}

impl<'a> ElfFile<'a> {
    pub fn parse(buffer: &'a [u8]) -> Result<Self> {
        Ok(ElfFile {
            inner: elf::Elf::parse(buffer)?,
            data: buffer,
        })
    }
}

#[derive(Debug)]
pub struct MachOFile<'a> {
    inner: mach::MachO<'a>,
}

impl<'a> MachOFile<'a> {
    pub fn parse(buffer: &'a [u8]) -> Result<Self> {
        Ok(MachOFile {
            inner: mach::MachO::parse(buffer, 0)?,
        })
    }
}

#[derive(Debug)]
pub struct PEFile<'a> {
    inner: pe::PE<'a>,
}

impl<'a> PEFile<'a> {
    pub fn parse(buffer: &'a [u8]) -> Result<Self> {
        Ok(PEFile {
            inner: pe::PE::parse(buffer)?,
        })
    }
}

#[derive(Debug)]
pub enum ObjectFile<'a> {
    Elf(ElfFile<'a>),
    MachO(MachOFile<'a>),
    PE(PEFile<'a>),
}

impl<'a> ObjectFile<'a> {
    pub fn parse(buffer: &'a [u8]) -> Result<Self> {
        let mut cursor = Cursor::new(buffer);
        Ok(match goblin::peek(&mut cursor)? {
            goblin::Hint::Elf(_) => ObjectFile::Elf(ElfFile::parse(buffer)?),
            goblin::Hint::Mach(_) => ObjectFile::MachO(MachOFile::parse(buffer)?),
            goblin::Hint::PE => ObjectFile::PE(PEFile::parse(buffer)?),
            _ => return Err("".into()),
        })
    }
}

// GENERIC DWARF STUFF ------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct DwarfSection<'a> {
    /// Data of this section
    pub data: &'a [u8],

    /// Section file offset
    pub offset: u64,
}

impl<'a> DwarfSection<'a> {
    pub fn new(data: &'a [u8], offset: u64) -> Self {
        DwarfSection { data, offset }
    }
}

pub trait Dwarf<'a> {
    /// Get the contents of the section named `section_name`, if such
    /// a section exists.
    fn get_section(&self, section_name: &str) -> Option<DwarfSection<'a>>;

    /// Get the endianity of this debug container
    fn little_endian(&self) -> bool;
}

// DWARF IMPLEMENTATION FOR ELF ---------------------------------------

impl<'a> Dwarf<'a> for ElfFile<'a> {
    fn get_section(&self, section_name: &str) -> Option<DwarfSection<'a>> {
        let &Self {
            ref inner,
            ref data,
        } = self;

        // We iterate over all section headers and try to find the section that corresponds to the
        // given name. Since get_section expects DWARF section names as used in ELF, we can directly
        // compare them.
        for header in &inner.section_headers {
            if let Some(Ok(name)) = inner.shdr_strtab.get(header.sh_name) {
                if name == section_name {
                    let sec_data = &data[header.sh_offset as usize..][..header.sh_size as usize];
                    return Some(DwarfSection::new(sec_data, header.sh_offset));
                }
            }
        }

        None
    }

    fn little_endian(&self) -> bool {
        self.inner.little_endian
    }
}

// DWARF IMPLEMENTATION FOR MACHO -------------------------------------

trait MachOArchitecture {
    fn cpu_type(&self) -> u32;
    fn cpu_subtype(&self) -> u32;
    fn from_macho(cpu_type: u32, cpu_subtype: u32) -> Self;
}

// TODO(ja): Implement MachOArchitecture for Architecture

/// Translate the "." prefix to the "__" prefix used by OSX/Mach-O, eg
/// ".debug_info" to "__debug_info".
fn macho_map_section_name(section_name: &str) -> String {
    let mut string = String::with_capacity(section_name.len() + 1);
    string.insert_str(0, "__");
    string.insert_str(2, &section_name[1..]);
    string
}

impl<'a> Dwarf<'a> for MachOFile<'a> {
    fn get_section(&self, section_name: &str) -> Option<DwarfSection<'a>> {
        // In MacOS, certain debug information is moved to the `__eh_frame` section in the main
        // binary instead of the `__debug_frame` section of the dSYM file. In that case, the section
        // is located in the `__TEXT` segment of the object file, instead of the `__DWARF` section
        // as usual.
        let segment_name = if section_name == ".eh_frame" {
            "__TEXT"
        } else {
            "__DWARF"
        };

        // Try to locate the segment within this object file
        let segment = self.inner.segments.into_iter().find(|segment| {
            segment
                .name()
                .map(|name| name == segment_name)
                .unwrap_or(false)
        });

        if let Some(segment) = segment {
            // MachO uses the `__` prefix instead of a `.` for all debug sections.
            let section_name = macho_map_section_name(section_name);

            for section in segment {
                if let Ok((section, data)) = section {
                    if let Ok(name) = section.name() {
                        if name == section_name {
                            return Some(DwarfSection::new(data, section.offset as u64));
                        }
                    }
                }
            }
        }

        None
    }

    fn little_endian(&self) -> bool {
        self.inner.little_endian
    }
}

// PDB STUFF --------------------------------------------------------

// TODO(ja): Implement a PdbTrait
