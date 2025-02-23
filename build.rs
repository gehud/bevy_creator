use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=crates");
    copy_to_to_build("crates");
    copy_to_to_build("templates");
    copy_to_to_build("assets/materials");
}

fn copy_to_to_build<P: AsRef<Path>>(path: P) {
    let input_path = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join(&path);
    let output_path = Path::new(&get_build_path()).join(&path);
    println!("cargo:warning={:#?}", copy_dir_all(input_path, output_path));
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

fn get_build_path() -> PathBuf {
    //<root or manifest path>/target/<profile>/
    let manifest_dir_string = env::var("CARGO_MANIFEST_DIR").unwrap();
    let build_type = env::var("PROFILE").unwrap();
    let path = Path::new(&manifest_dir_string)
        .join("target")
        .join(build_type);
    return PathBuf::from(path);
}
