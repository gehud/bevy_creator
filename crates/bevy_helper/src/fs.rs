use std::{
    fs::{copy, create_dir_all, read_dir},
    io::Result as IoResult,
    path::Path,
};

pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> IoResult<()> {
    create_dir_all(&dst)?;
    for entry in read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }

    Ok(())
}
