use rand::Rng;
use std::path::PathBuf;
use std::{env, fs};

#[cfg(feature = "guess-layout")]
fn find_max_vaddr_bits<const NB_TRIES: usize>() -> usize {
    let mut rng = rand::rng();
    let page_size = page_size::get();

    assert_eq!(page_size.count_ones(), 1);

    let mut bits_min: usize = page_size.trailing_zeros() as usize; // log2(page_size)
    let mut bits_max: usize = size_of::<usize>() * 8; // size in bits of max addressable memory

    while bits_min != bits_max {
        let bits_current = (bits_min + bits_max) / 2;

        let mut is_mappable = false;
        for _ in 0..NB_TRIES {
            let current_addr_min = 1usize << (bits_current - 1);
            let current_addr_max = 1usize << bits_current;
            let current_addr_sz = current_addr_max - current_addr_min;

            assert_eq!(current_addr_sz % page_size, 0);

            let max_page = current_addr_sz / page_size;

            let rdm_page = rng.random_range(0..max_page);

            let map_addr = current_addr_min + (page_size * rdm_page);

            let map_addr_ptr = unsafe {
                libc::mmap(
                    map_addr as *mut libc::c_void,
                    page_size,
                    libc::PROT_READ,
                    libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | libc::MAP_FIXED,
                    -1,
                    0,
                )
            };

            if map_addr_ptr != (-1isize as *mut libc::c_void) {
                unsafe {
                    libc::munmap(map_addr_ptr, page_size);
                }
                is_mappable = true;
                break;
            }
        }

        if is_mappable {
            bits_min = bits_current + 1;
        } else {
            bits_max = bits_current;
        }
    }

    bits_min
}

fn write_host_info() {
    let target_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let host_info = target_dir.join("host_info.rs");

    let nb_vaddr_bits = find_max_vaddr_bits::<8>();

    println!("cargo:warning=nb_vaddr_bits {nb_vaddr_bits}");

    let b = (1usize << (nb_vaddr_bits - 1));

    println!("cargo:warning=b = {b:#x}");

    // lower bound of HighShadow
    let a = 0x02008fff7000usize;

    println!("cargo:warning=a = {a:#x}");

    println!("cargo:warning=b - a = {:#x}", b - a);

    let r = (b - a) % 9;

    // if not true, ideal space does not exist
    assert_eq!(r, 0);

    let k = (b - a) / 9;
    let x = a + k;

    let host_info_content = format!(
        r#"
        pub const HOST_INFO_MAX_VADDR_BITS: usize = {nb_vaddr_bits};

        pub const HOST_INFO_HIGH_SHADOW_SIZE: usize = {};
        pub const HOST_INFO_HIGH_MEM_OFFSET: usize = {};
        pub const HOST_INFO_HIGH_MEM_SIZE: usize = {};
    "#,
        x - 1 - a,
        x,
        b - 1
    );

    fs::write(host_info.as_path(), host_info_content.as_bytes()).unwrap();
}

pub fn main() {
    write_host_info();
}
