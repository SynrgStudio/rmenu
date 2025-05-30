use std::env;
use std::fs;
use std::path::Path;

fn main() {
    #[cfg(windows)]
    {
        embed_resource::compile("resource.rc", embed_resource::NONE);
    }

    // Obtener el directorio de salida
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).parent().unwrap().parent().unwrap().parent().unwrap();
    
    // Copiar config_example.ini al directorio de salida
    let example_config = Path::new("config_example.ini");
    if example_config.exists() {
        println!("cargo:warning=Copiando config_example.ini al directorio de salida");
        let dest_file = dest_path.join("config_example.ini");
        fs::copy(example_config, dest_file).expect("No se pudo copiar config_example.ini");
    } else {
        println!("cargo:warning=No se encontró config_example.ini");
    }
    
    // También copiar README.md
    let readme = Path::new("README.md");
    if readme.exists() {
        println!("cargo:warning=Copiando README.md al directorio de salida");
        let dest_file = dest_path.join("README.md");
        fs::copy(readme, dest_file).expect("No se pudo copiar README.md");
    }
    
    println!("cargo:rerun-if-changed=config_example.ini");
    println!("cargo:rerun-if-changed=README.md");
} 