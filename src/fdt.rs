//! Flattened Device Tree (FDT) definitions and routines.
//!
//! The DeviceTree specification is available at https://www.devicetree.org.

use crate::util::raw_cast;

// Note: FDT data is stored in big endian format

static MAGIC: u32 = 0xd00dfeed;
static COMPATIBLE_VERSION: u32 = 17;

pub enum FdtError {
    BadMagic(u32),
    InvalidFormat,
    IncompatibleVersion,
    BufferOverflow,
}

pub struct Fdt<'a> {
    header: &'a Header,
    mem_rsvmap: &'a [u8],
    dt_struct: &'a [u8],
    dt_strings: &'a [u8],
}

impl<'a> Fdt<'a> {
    pub unsafe fn from_ptr(pointer: *const core::ffi::c_void) -> Result<Fdt<'static>, FdtError> {
        let fdt_size = Fdt::read_length(pointer)?;
        let fdt_buffer = unsafe { core::slice::from_raw_parts(pointer as *const u8, fdt_size) };

        Fdt::from_buffer(fdt_buffer)
    }

    pub unsafe fn read_length(pointer: *const core::ffi::c_void) -> Result<usize, FdtError> {
        let header = & *(pointer as *const Header);

        if !header.is_valid() {
            Err(FdtError::BadMagic(header.magic))
        } else {
            Ok(u32::from_be(header.totalsize) as usize)
        }
    }

    pub unsafe fn from_buffer(buffer: &'a [u8]) -> Result<Fdt<'a>, FdtError> {
        if buffer.len() < core::mem::size_of::<Header>() {
            return Err(FdtError::BufferOverflow);
        }

        let header_buffer = &buffer[0..core::mem::size_of::<Header>()];
        if let Some(header) = raw_cast::<Header>(header_buffer) {
            if !header.is_valid() {
                return Err(FdtError::BadMagic(header.magic));
            }
            if !header.is_compatible() {
                return Err(FdtError::IncompatibleVersion);
            }

            let offset_reservation = u32::from_be(header.off_mem_rsvmap) as usize;
            let offset_structure = u32::from_be(header.off_dt_struct) as usize;
            let offset_strings = u32::from_be(header.off_dt_strings) as usize;

            let end_structure = offset_structure + u32::from_be(header.size_dt_struct) as usize;
            let end_strings = offset_strings + u32::from_be(header.size_dt_strings) as usize;

            // TODO: size of memory reservation map?

            if offset_structure > end_structure ||
                offset_strings > end_strings
            {
                return Err(FdtError::InvalidFormat);
            }

            if end_strings > buffer.len() {
                return Err(FdtError::BufferOverflow);
            }

            Ok(Fdt {
                header,
                mem_rsvmap: &buffer[offset_reservation..offset_reservation],
                dt_struct: &buffer[offset_structure..end_structure],
                dt_strings: &buffer[offset_strings..end_strings],
            })
        } else {
            Err(FdtError::InvalidFormat)
        }
    }

    pub fn nodes(&self) -> DeviceTreeIterator<'_, 'a> {
        DeviceTreeIterator {
            fdt: self,
            next_token_offset: 0,
        }
    }
}

/// FDT Header
#[repr(C)]
pub struct Header {
    magic: u32,
    totalsize: u32,
    off_dt_struct: u32,
    off_dt_strings: u32,
    off_mem_rsvmap: u32,
    version: u32,
    last_comp_version: u32,
    boot_cpuid_phys: u32,
    size_dt_strings: u32,
    size_dt_struct: u32,
}

impl Header {
    fn is_valid(&self) -> bool {
        u32::from_be(self.magic) == MAGIC
    }

    fn is_compatible(&self) -> bool {
        u32::from_be(self.version) >= COMPATIBLE_VERSION &&
            u32::from_be(self.last_comp_version) <= COMPATIBLE_VERSION
    }
}

pub type FdtTokenType = u32;
pub const FDT_BEGIN_NODE: FdtTokenType = 0x00000001;
pub const FDT_END_NODE: FdtTokenType = 0x00000002;
pub const FDT_PROP: FdtTokenType = 0x00000003;
pub const FDT_NOP: FdtTokenType = 0x00000004;
pub const FDT_END: FdtTokenType = 0x00000009;

pub struct DeviceTreeIterator<'a, 'b> {
    fdt: &'a Fdt<'b>,
    next_token_offset: usize,
}

impl<'a, 'b> core::iter::Iterator for DeviceTreeIterator<'a, 'b> {
    type Item = Option<&'b [u8]>;

    fn next(&mut self) -> Option<Self::Item> {
        let dt_struct = self.fdt.dt_struct;
        let offset = self.next_token_offset;

        if offset >= dt_struct.len() {
            return None;
        }

        let mut next_offset = offset + core::mem::size_of::<FdtTokenType>();
        let token_type = read_word(&dt_struct[offset..next_offset]);

        match token_type {
            FDT_BEGIN_NODE => {
                let name_start = next_offset;
                while dt_struct[next_offset] != 0 {
                    next_offset += 1;
                    if next_offset >= dt_struct.len() {
                        return None;
                    }
                }

                let name = &dt_struct[name_start..next_offset];

                next_offset += 1;
                self.next_token_offset = next_aligned(next_offset);

                Some(Some(name))
            },
            FDT_PROP => {
                let length = read_word(&dt_struct[next_offset..(next_offset + 4)]) as usize;
                let _name_offset = read_word(
                    &dt_struct[(next_offset + 4)..(next_offset + 8)]
                );

                next_offset = next_offset + 8 + length;
                self.next_token_offset = next_aligned(next_offset);

                Some(None)
            },
            FDT_END_NODE => {
                self.next_token_offset = next_offset;
                Some(None)
            },
            FDT_NOP => {
                self.next_token_offset = next_offset;
                Some(None)
            },
            _ => None,
        }
    }
}

fn read_word(buffer: &[u8]) -> u32 {
    let mut bytes = [0u8; 4];
    bytes.copy_from_slice(buffer);

    u32::from_be_bytes(bytes)
}

fn next_aligned(mut offset: usize) -> usize {
    // jump to next 32-bit aligned boundary
    while offset & 0x3 > 0 {
        offset += 1;
    }

    offset
}
