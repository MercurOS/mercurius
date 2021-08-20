//! Page frame allocation.

#[cfg(test)]
#[path = "frame_tests.rs"]
mod frame_tests;

use core::ptr::addr_of_mut;

const FREE: u8 = 0u8;
const USED: u8 = 1u8;

// 256 bytes covering 1 page per bit, i.e. 8 pages per byte
const PAGE_COUNT: usize = 256 * 8;

#[repr(C)]
pub struct Buddy {
    l0: [u8; 256],
    l1: [u8; 128],
    l2: [u8; 64],
    l3: [u8; 32],
    base: u64,
    next_map: *const Buddy,
}

impl Buddy {
    /// Initializes a buddy-map for a memory region starting at `base`,
    /// and places the map in memory at `address`.
    ///
    /// Both `base` and `address` must be page aligned memory addresses.
    ///
    /// Upon initialization the whole map will be marked as occupied.
    pub unsafe fn new(base: u64, address: u64) -> &'static mut Self {
        let ptr = address as *mut Buddy;

        // initialize all layers as used
        addr_of_mut!((*ptr).l0).write_bytes(0xFFu8, 1);
        addr_of_mut!((*ptr).l1).write_bytes(0xFFu8, 1);
        addr_of_mut!((*ptr).l2).write_bytes(0xFFu8, 1);
        addr_of_mut!((*ptr).l3).write_bytes(0xFFu8, 1);

        addr_of_mut!((*ptr).base).write(base);
        addr_of_mut!((*ptr).next_map).write(core::ptr::null());

        // SAFETY: all fields of Buddy have been initialized
        let buddy = &mut *ptr;

        buddy
    }

    /// Attempt to allocate `page_count` consecutive pages.
    pub fn allocate(&mut self, page_count: usize) -> Option<*mut core::ffi::c_void> {
        // TODO

        if page_count == 1 {
            for byte_index in 0..self.l0.len() {
                if self.l0[byte_index] == 0xFF {
                    continue;
                }

                for offset in (byte_index * 8)..(byte_index * 8 + 8) {
                    if self.check_layer(offset, 0) {
                        let address = self.offset_to_address(offset);
                        self.mark(address, 1);
                        return Some(address as *mut core::ffi::c_void);
                    }
                }
            }
        }

        None
    }

    /// Mark specific page(s) as free.
    ///
    /// Both `base` and `address` must be page aligned memory addresses.
    pub fn free(&mut self, page_base: u64, page_count: usize) -> bool {
        if !self.in_range(page_base) {
            return false;
        }

        let offset = self.address_to_offset(page_base);
        for page in 0..page_count {
            let page_offset = offset + page;

            if !self.in_range(self.offset_to_address(page_offset)) {
                break;
            }

            if self.check(page_offset) {
                continue;
            }

            self.free_one(page_offset, 0);
            for layer in 0..=2 {
                if !self.try_join(page_offset, layer) {
                    break;
                }
            }
        }

        true
    }

    /// Mark specific page(s) as allocated.
    ///
    /// Both `base` and `address` must be page aligned memory addresses.
    pub fn mark(&mut self, page_base: u64, page_count: usize) -> bool {
        if !self.in_range(page_base) {
            return false;
        }

        let offset = self.address_to_offset(page_base);
        for page in 0..page_count {
            let page_offset = offset + page;

            if !self.in_range(self.offset_to_address(page_offset)) {
                break;
            }

            for layer in (1..=3).rev() {
                if self.check_layer(page_offset, layer) {
                    self.try_split(page_offset, layer);
                }
            }

            self.mark_one(page_offset, 0);
        }

        true
    }

    /// Convert page address to page offset within the map.
    fn address_to_offset(&self, address: u64) -> usize {
        ((address - self.base) >> 12) as usize
    }

    fn offset_to_address(&self, offset: usize) -> u64 {
        self.base + (offset << 12) as u64
    }

    fn in_range(&self, address: u64) -> bool {
        address >= self.base && address < self.base + (PAGE_COUNT << 12) as u64
    }

    /// Returns true if free, else false.
    fn check(&self, offset: usize) -> bool {
        for layer in (0..=3).rev() {
            if self.check_layer(offset, layer) {
                return true;
            }
        }

        false
    }

    /// Returns true if free, else false.
    fn check_layer(&self, offset: usize, layer: usize) -> bool {
        let layer_offset = offset >> layer;
        let byte_offset = layer_offset / 8;
        let bit_offset = 7 - (layer_offset % 8);

        let byte = match layer {
            0 => self.l0[byte_offset],
            1 => self.l1[byte_offset],
            2 => self.l2[byte_offset],
            3 => self.l3[byte_offset],
            _ => panic!("layer out of range"),
        };

        (byte >> bit_offset) & USED == FREE
    }

    fn try_join(&mut self, offset: usize, layer: usize) -> bool {
        if layer > 2 {
            return false;
        }

        let layer_offset = offset >> layer;
        let byte_offset = layer_offset / 8;
        // always pick the even bit to simplify the join check
        let bit_offset = (7 - (layer_offset % 8)) & !1;

        let upper_layer_offset = layer_offset >> 1;
        let upper_byte_offset = upper_layer_offset / 8;
        let upper_bit_offset = 7 - (upper_layer_offset % 8);

        let (byte, upper) = match layer {
            0 => (&mut self.l0[byte_offset], &mut self.l1[upper_byte_offset]),
            1 => (&mut self.l1[byte_offset], &mut self.l2[upper_byte_offset]),
            2 => (&mut self.l2[byte_offset], &mut self.l3[upper_byte_offset]),
            _ => panic!("layer out of range"),
        };

        if (*byte >> bit_offset) & 0b11 != FREE {
            return false;
        }

        // mark two layer l bits as used, freeing up one bit on layer l + 1
        *byte |= 0b11 << bit_offset;
        *upper &= !(USED << upper_bit_offset);

        true
    }

    fn try_split(&mut self, offset: usize, layer: usize) -> bool {
        if layer == 0 || layer > 3 {
            return false;
        }

        let layer_offset = offset >> layer;
        let byte_offset = layer_offset / 8;
        let bit_offset = 7 - (layer_offset % 8);

        let lower_layer_offset = layer_offset << 1;
        let lower_byte_offset = lower_layer_offset / 8;
        let lower_bit_offset = 7 - (lower_layer_offset % 8);

        let (byte, lower) = match layer {
            1 => (&mut self.l1[byte_offset], &mut self.l0[lower_byte_offset]),
            2 => (&mut self.l2[byte_offset], &mut self.l1[lower_byte_offset]),
            3 => (&mut self.l3[byte_offset], &mut self.l2[lower_byte_offset]),
            _ => panic!("layer out of range"),
        };

        if (*byte >> bit_offset) & 1 != FREE {
            return false;
        }

        // mark layer l bit as used, freeing up two bits on layer l - 1
        *byte |= USED << bit_offset;
        // layer l - 1 offset is always even given layer l offset is aligned
        // to layer size, so we can directly zero both ajacent bits
        *lower &= !(0b11 << (lower_bit_offset - 1));

        true
    }

    fn free_one(&mut self, offset: usize, layer: usize) {
        let layer_offset = offset >> layer;
        let byte_offset = layer_offset / 8;
        let bit_offset = 7 - (layer_offset % 8);
        let mask = !(USED << bit_offset);

        match layer {
            0 => self.l0[byte_offset] &= mask,
            1 => self.l1[byte_offset] &= mask,
            2 => self.l2[byte_offset] &= mask,
            3 => self.l3[byte_offset] &= mask,
            _ => panic!("layer out of range"),
        }
    }

    fn mark_one(&mut self, offset: usize, layer: usize) {
        let layer_offset = offset >> layer;
        let byte_offset = layer_offset / 8;
        let bit_offset = 7 - (layer_offset % 8);
        let mask = USED << bit_offset;

        match layer {
            0 => self.l0[byte_offset] |= mask,
            1 => self.l1[byte_offset] |= mask,
            2 => self.l2[byte_offset] |= mask,
            3 => self.l3[byte_offset] |= mask,
            _ => panic!("layer out of range"),
        }
    }
}

impl core::fmt::Debug for Buddy {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // FIXME: Line breaks!
        for layer in (0..=3).rev() {
            for offset in (0..PAGE_COUNT).step_by(1 << layer) {
                if self.check_layer(offset, layer) {
                    write!(
                        f,
                        "{:#018X}: {} page(s) free\r\n",
                        self.offset_to_address(offset),
                        1 << layer
                    )?;
                }
            }
        }

        Ok(())
    }
}
