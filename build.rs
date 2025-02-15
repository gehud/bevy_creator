use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=crates");
    let output_path = get_output_path();
    let input_path = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("crates");
    let output_path = Path::new(&output_path).join("crates");
    let res = copy_dir_all(input_path, output_path);
    println!("cargo:warning={:#?}",res)
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    
    Ok(())
}

fn get_output_path() -> PathBuf {
    let manifest_dir_string = env::var("CARGO_MANIFEST_DIR").unwrap();
    let path = Path::new(&manifest_dir_string).join("target").join("bin");
    return PathBuf::from(path);
}