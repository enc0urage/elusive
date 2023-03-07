#![no_main]
#![no_std]
#![feature(abi_efiapi)]
#![allow(stable_features)]

use core::fmt::Write;
use uefi::prelude::*;
use uefi::table::boot::MemoryType;
use uefi::table::runtime::ResetType;

static mut SYSTEM_TABLE: Option<SystemTable<Boot>> = None;

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    if let Some(st) = unsafe { SYSTEM_TABLE.as_mut() } {
        let stdout = st.stdout();
        write!(stdout, "[PANIC]: {}", info);
        st.boot_services().stall(10_000_000);
        st.runtime_services()
            .reset(ResetType::Shutdown, Status::ABORTED, None);
    }
    loop {}
}

#[entry]
fn main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    let mut total_size: usize = 0;

    unsafe {
        SYSTEM_TABLE = Some(system_table.unsafe_clone());
    }

    system_table.stdout().reset(true).unwrap();
    write!(
        unsafe { system_table.unsafe_clone() }.stdout(),
        "{} {} on {} {}\n",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        system_table.firmware_vendor(),
        system_table.firmware_revision()
    )
    .unwrap();

    let memory_map_entry_size = system_table.boot_services().memory_map_size().entry_size;
    let memory_map_size =
        system_table.boot_services().memory_map_size().map_size + (memory_map_entry_size * 10);
    let memory_map_addr = system_table
        .boot_services()
        .allocate_pool(MemoryType::LOADER_DATA, memory_map_size)
        .unwrap();
    write!(
        system_table.stdout(),
        "memory_map_entry_size={} memory_map_size={} memory_map_addr={:?}\n",
        memory_map_entry_size,
        memory_map_size,
        memory_map_addr
    )
    .unwrap();
    let memory_map_buf = core::ptr::slice_from_raw_parts_mut(memory_map_addr, memory_map_size);

    let (_memory_key, memory_map) = system_table
        .boot_services()
        .memory_map(unsafe { &mut *memory_map_buf })
        .unwrap();

    for desc in memory_map {
        if desc.ty == MemoryType::CONVENTIONAL {
            let size: usize = (desc.page_count * 4096) as usize;
            write!(
                system_table.stdout(),
                "erasing: {:#x} ({})\n",
                desc.phys_start,
                size
            )
            .unwrap();

            unsafe {
                system_table
                    .boot_services()
                    .set_mem(desc.phys_start as *mut u8, size, 0);
            }

            total_size += size;
        }
    }

    write!(system_table.stdout(), "total_mem_size={}\n", total_size).unwrap();

    system_table
        .runtime_services()
        .reset(ResetType::Shutdown, Status::SUCCESS, None);
}
