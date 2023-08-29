use x86_64::{structures::paging::PageTable, PhysAddr, VirtAddr};

/// Returns a mutable reference to the active level 4 table.
///
/// # Safety
///
/// The caller must guarantee that the complete physical memory is
/// mapped to virtual memory at the `physical_memory_offset`. This
/// function can only be called once to avoid aliasing
/// &mut references (which results in UB).
#[must_use]
pub unsafe fn active_level4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level4_table_frame, _) = Cr3::read();

    let phys = level4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    // unsafe
    &mut *page_table_ptr
}

/// Translate given virtual address to physical address or `None`
/// if the address is not mapped.
///
/// # Safety
///
/// This function is unsafe because the caller must guarantee that
/// the complete physical memory is mapped to virtual memory at
/// the `physical_memory_offset`. This function can only be called
/// once to avoid aliasing &mut references (which results in UB).
///
/// # Panics
///
/// This function panics when encountering huge page.
#[must_use]
pub unsafe fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    use x86_64::{registers::control::Cr3, structures::paging::page_table::FrameError};

    // Read the active level4 from the Cr3 register
    let (level4_table_frame, _) = Cr3::read();

    let table_indices = [
        addr.p4_index(),
        addr.p3_index(),
        addr.p2_index(),
        addr.p1_index(),
    ];
    let mut frame = level4_table_frame;

    // Traverse the multilevel page table
    for &index in &table_indices {
        let virt = physical_memory_offset + frame.start_address().as_u64();
        let table_ptr: *const PageTable = virt.as_ptr();
        let table = unsafe { &*table_ptr };

        let entry = &table[index];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("Huge pages not supported!"),
        };
    }

    Some(frame.start_address() + u64::from(addr.page_offset()))
}
