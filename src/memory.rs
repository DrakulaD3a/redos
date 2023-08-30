use x86_64::{
    structures::paging::{OffsetPageTable, PageTable},
    VirtAddr,
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
    let level4_table = active_level4_table(physical_memory_offset);
    OffsetPageTable::new(level4_table, physical_memory_offset)
}

#[must_use]
unsafe fn active_level4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level4_table_frame, _) = Cr3::read();

    let phys = level4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    // unsafe
    &mut *page_table_ptr
}
