use crate::util;
use rexiv2::Metadata;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn get_stripped_image_bytes(image_bytes: &Vec<u8>) -> Result<Vec<u8>, String> {
    // Step 1: Save the image bytes into a file.
    let path_string = format!("/tmp/image_file_path_{}.data", util::create_uuid());
    let path: &Path = Path::new(&path_string);
    fs::write(path, image_bytes).map_err(|_| "failed to write image to file.")?;

    // Step 2: Load the file into rexiv2 struct.
    let metadata =
        Metadata::new_from_path(path).map_err(|_| "failed to load metadata from image buffer.")?;

    // Step 3: Strip the metadata from the struct.
    metadata.clear();

    // Step 4: Save the rexiv2 struct back to the file.
    metadata
        .save_to_file(path)
        .map_err(|_| "failed to save cleared metadata back to file.")?;

    // Step 5: Read the file back into memory.
    let mut f = File::open(path).map_err(|_| "failed to open the cleared image file.")?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)
        .map_err(|_| "failed to read cleared image file to bytes.")?;

    // Step 6: Delete all intermediate files.

    Ok(buffer)
}
