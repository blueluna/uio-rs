use crate::error::Error;
use std::fs::{self, File, OpenOptions};

/// Format path for devfs UIO device file
pub(crate) fn devfs_uio_file_path(uio_number: usize) -> std::path::PathBuf {
    std::path::PathBuf::from(format!("/dev/uio{}", uio_number))
}

/// Open the `/dev/uio<number>`
pub(crate) fn devfs_uio_open(uio_number: usize) -> Result<File, Error> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .open(devfs_uio_file_path(uio_number))
        .map_err(|e| e.into())
}

/// Format path for UIO sysfs directory
pub(crate) fn sysfs_uio_dir(uio_number: usize) -> std::path::PathBuf {
    std::path::PathBuf::from(format!("/sys/class/uio/uio{}", uio_number))
}

/// Format path for UIO maps sysfs directory
pub(crate) fn sysfs_uio_maps_dir(uio_number: usize) -> std::path::PathBuf {
    sysfs_uio_dir(uio_number).join("maps")
}

/// Format path for UIO map sysfs directory
pub(crate) fn sysfs_uio_map_dir(uio_number: usize, map_number: usize) -> std::path::PathBuf {
    sysfs_uio_maps_dir(uio_number).join(format!("map{}", map_number))
}

/// Read string from file and trim it
pub(crate) fn read_string<P: AsRef<std::path::Path>>(path: P) -> Result<String, Error> {
    let text = fs::read_to_string(path)?;
    Ok(String::from(text.trim()))
}

/// Read hexadecimal u64 from file. Will remove `0x` prefix.
pub(crate) fn read_hexadecimal_u64<P: AsRef<std::path::Path>>(path: P) -> Result<u64, Error> {
    let string = read_string(path)?;
    let string = if string.starts_with("0x") {
        let (_, b) = string.split_at(2);
        b
    } else {
        &string
    };
    u64::from_str_radix(string, 16).map_err(|e| e.into())
}

/// Read hexadecimal usize from file. Will remove `0x` prefix.
pub(crate) fn read_hexadecimal_usize<P: AsRef<std::path::Path>>(path: P) -> Result<usize, Error> {
    let text = read_string(path)?;
    let text = if text.starts_with("0x") {
        let (_, b) = text.split_at(2);
        b
    } else {
        &text
    };
    usize::from_str_radix(text, 16).map_err(|e| e.into())
}
