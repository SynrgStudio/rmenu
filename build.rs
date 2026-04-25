use std::env;
use std::fs;
use std::path::Path;

fn main() {
    #[cfg(windows)]
    {
        embed_resource::compile("resource.rc", embed_resource::NONE);
    }

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    copy_if_exists("config_example.ini", dest_path);
    copy_if_exists("README.md", dest_path);

    println!("cargo:rerun-if-changed=config_example.ini");
    println!("cargo:rerun-if-changed=README.md");
}

fn copy_if_exists(file_name: &str, dest_path: &Path) {
    let source = Path::new(file_name);
    if !source.exists() {
        return;
    }

    let destination = dest_path.join(file_name);
    fs::copy(source, destination).unwrap_or_else(|err| {
        panic!("Could not copy {file_name} to the output directory: {err}");
    });
}
