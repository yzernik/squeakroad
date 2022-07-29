use crate::util;
use rexiv2::Metadata;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn get_stripped_image_bytes(image_bytes: &Vec<u8>) -> Result<Vec<u8>, String> {
    let path_string = format!("/tmp/image_file_{}.data", util::create_uuid());
    let path: &Path = Path::new(&path_string);
    fs::write(path, image_bytes).map_err(|_| "failed to write image to file.")?;

    match get_stripped_image_from_path(path) {
        Ok(data) => {
            fs::remove_file(path).ok();
            Ok(data)
        }
        Err(e) => {
            fs::remove_file(path).ok();
            Err(e)
        }
    }
}

pub fn get_stripped_image_from_path(file_path: &Path) -> Result<Vec<u8>, String> {
    let metadata = Metadata::new_from_path(file_path)
        .map_err(|_| "failed to load metadata from image buffer.")?;
    metadata.clear();
    metadata
        .save_to_file(file_path)
        .map_err(|_| "failed to save cleared metadata back to file.")?;
    let mut f = File::open(file_path).map_err(|_| "failed to open the cleared image file.")?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)
        .map_err(|_| "failed to read cleared image file to bytes.")?;

    Ok(buffer)
}
