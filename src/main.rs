use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::env;
use std::io::Error;
use std::convert::TryFrom;

struct fdt_header {
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

fn as_u32_be(array: &[u8]) -> u32 {
    ((array[0] as u32) << 24) +
        ((array[1] as u32) << 16) +
        ((array[2] as u32) <<  8) +
        ((array[3] as u32) <<  0)
}

fn transform_u32_to_array_of_u8(x:u32) -> [u8;4] {
    let b1: u8 = ((x >> 24) & 0xff) as u8;
    let b2: u8 = ((x >> 16) & 0xff) as u8;
    let b3: u8 = ((x >> 8) & 0xff) as u8;
    let b4: u8 = (x & 0xff) as u8;
    return [b1, b2, b3, b4]
}

fn main() -> Result<(), Error> {
    let args = env::args();
    if args.len() <= 1 {
        panic!("Expected file path as arg");
    }
    let last_arg = args.last().expect("No last arg");

    let mut f = File::open(last_arg)?;

    let data = f.bytes().filter_map(|b| b.ok()).collect::<Vec<u8>>();

    //0xd00dfeed

    let chunks_i32 = data.chunks_exact(4);

    let data_u32 = chunks_i32.into_iter().map(|chunk| {
        as_u32_be(chunk)
    }).collect::<Vec<u32>>();

    let mut magic_indexes: Vec<u32> = vec![];

    // find magics
    for (index, part) in data_u32.clone().into_iter().enumerate() {
        if part == 0xd00dfeed {
            println!("found magic at 0x{:x?}", index * 4);
            magic_indexes.push(u32::try_from(index).unwrap());
        }
    }

    // chop data from one magic to other magic

    for (indexes_index, index) in magic_indexes.clone().into_iter().enumerate() {
        let next_magic = if indexes_index + 1 == magic_indexes.len() {
            data_u32.clone().len()
        } else {
            usize::try_from(magic_indexes[indexes_index + 1]).unwrap()
        };

        let dtb_slice = &data_u32[usize::try_from(index).unwrap()..next_magic];
        let mut dtb_file = File::create(format!("dtb_slice_{}", indexes_index))?;

        let mut new_data: Vec<[u8;4]> = vec![];

        for a in dtb_slice.iter() {
            new_data.push(transform_u32_to_array_of_u8(*a));
        }

        let mut write_data: Vec<u8> = vec![];

        for x in new_data {
            write_data.push(x[0].clone());
            write_data.push(x[1].clone());
            write_data.push(x[2].clone());
            write_data.push(x[3].clone());
        }

        dtb_file.write(write_data.as_slice());
        dtb_file.flush();
    }

    Ok(())
}
