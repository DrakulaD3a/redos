use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{
    structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB},
    PhysAddr, VirtAddr,
};

/// Initializes a new [OffsetPageTable](https://docs.rs/x86_64/latest/x86_64/structures/paging/mapper/struct.OffsetPageTable.html).
///
/// # Safety
///
/// This function is unsafe because the caller must guarantee that
/// the complete physical memory is mapped to virtual memory at
/// the `physical_memory_offset`. This function can only be called
/// once to avoid aliasing &mut references (which results in UB).
#[must_use]
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    unsafe {
        OffsetPageTable::new(
            active_level4_table(physical_memory_offset),
            physical_memory_offset,
        )
    }
}

#[must_use]
unsafe fn active_level4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level4_table_frame, _) = Cr3::read();

    let phys = level4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe { &mut *page_table_ptr }
}

/// A temporary type so we can make a field in `BootInfoFrameAllocator` without known type
type UsableFrames = impl Iterator<Item = PhysFrame>;
/// A `FrameAllocator` that returns usable frames form bootloader's memory map.
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    usable_frames: UsableFrames,
}

impl BootInfoFrameAllocator {
    /// Create a `FrameAllocator` from the passed map.
    ///
    /// # Safety
    /// The caller must guarantee that the passed `memory_map` is valid.
    /// Meaning all the frames marked as `USABLE` in specified `memory_map` must be unused.
    #[must_use]
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        let frame_addresses = memory_map
            .iter()
            // Filter only usable regions
            .filter(|r| r.region_type == MemoryRegionType::Usable)
            // Get the address ranges
            .map(|r| r.range.start_addr()..r.range.end_addr())
            // Get the frame start addresses
            .flat_map(|r| r.step_by(4096));

        // Create `PhysFrame` from the start addresses
        let usable_frames =
            frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)));

        Self {
            memory_map,
            usable_frames,
        }
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        self.usable_frames.next()
    }
}
