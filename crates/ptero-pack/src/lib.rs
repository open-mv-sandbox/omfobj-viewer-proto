//! Pterodactil Bring-Your-Own-IO dacti reading and writing library.

use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    mem::size_of,
};

use anyhow::{bail, Error};
use dacti_pack::{
    IndexComponentHeader, IndexEntry, IndexGroupEncoding, IndexGroupHeader, INDEX_COMPONENT_UUID,
};
use daicon::{ComponentEntry, ComponentTableHeader, RegionData};
use stewart::{task::Recipe, Context};
use tracing::{event, Level};
use uuid::Uuid;

pub fn create_add_data_recipe(package: File, data: Vec<u8>, uuid: Uuid) -> Recipe {
    Recipe::new(move |c| start_add_data_task(c, package, data, uuid))
}

fn start_add_data_task(
    _context: Context,
    mut package: File,
    data: Vec<u8>,
    uuid: Uuid,
) -> Result<(), Error> {
    event!(Level::DEBUG, "adding data to package");

    // The first 64kb is reserved for components and indices
    let data_start = 1024 * 64;

    add_index(&mut package, uuid, data_start as u32, data.len() as u32)?;

    // Write the file to the package
    package.seek(SeekFrom::Start(data_start))?;
    package.write_all(&data)?;

    Ok(())
}

pub enum IoMessage {
    Write { start: u64, data: Vec<u8> },
}

fn add_index(package: &mut File, uuid: Uuid, offset: u32, size: u32) -> Result<(), Error> {
    // TODO: Find a free slot rather than just assuming there's no files yet

    // Find the current location of the index component
    let (table_region_offset, entry) = find_component_entry(package, INDEX_COMPONENT_UUID)?;
    let region = RegionData::from_bytes(entry.value.data());
    let component_offset = table_region_offset + region.offset() as u64;

    // Add entries for the new file's location and size
    let entry_offset = find_next_free_index(package, component_offset)?;

    let mut entry = IndexEntry::zeroed();
    entry.set_uuid(uuid);
    entry.set_offset(offset);
    entry.set_size(size);

    package.seek(SeekFrom::Start(entry_offset))?;
    package.write_all(entry.as_bytes())?;

    Ok(())
}

fn find_next_free_index(package: &mut File, component_offset: u64) -> Result<u64, Error> {
    // TODO: Find a free slot rather than just assuming there's no groups yet

    let mut header = IndexComponentHeader::zeroed();
    package.seek(SeekFrom::Start(component_offset))?;
    package.read_exact(header.as_bytes_mut())?;
    header.set_groups(1);
    package.seek(SeekFrom::Start(component_offset))?;
    package.write_all(header.as_bytes())?;

    let mut group = IndexGroupHeader::zeroed();
    group.set_encoding(IndexGroupEncoding::None);
    group.set_length(1);
    package.write_all(group.as_bytes())?;

    let offset = package.stream_position()?;
    Ok(offset)
}

fn find_component_entry(
    package: &mut File,
    uuid: Uuid,
) -> Result<(u64, Indexed<ComponentEntry>), Error> {
    let mut header = ComponentTableHeader::zeroed();
    package.seek(SeekFrom::Start(8))?;
    package.read_exact(header.as_bytes_mut())?;

    // TODO: Follow extensions

    let mut entry_offset = package.stream_position()?;
    for _ in 0..header.length() {
        let mut entry = ComponentEntry::zeroed();
        package.read_exact(entry.as_bytes_mut())?;

        // Continue until we find the correct component
        if entry.type_uuid() != uuid {
            entry_offset = package.seek(SeekFrom::Current(size_of::<ComponentEntry>() as i64))?;
            continue;
        }

        let entry_i = Indexed {
            offset: entry_offset,
            value: entry,
        };
        return Ok((header.entries_offset(), entry_i));
    }

    bail!("unable to find index component");
}

/// Combination value and its index as byte offset.
struct Indexed<T> {
    #[allow(dead_code)]
    offset: u64,
    value: T,
}
