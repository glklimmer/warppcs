use std::env;

fn main() {
    let mode = env::var("MODE").unwrap();
    println!("cargo::rerun-if-env-changed=MODE");
    match mode.as_str() {
        "DEV" => {
            println!("cargo:rustc-cfg=dev");
        }
        "PROD" => {
            println!("cargo:rustc-cfg=prod");
        }
        _ => {
            println!("cargo:warning=Unknown MODE: {}, defaulting to PROD", mode);
            println!("cargo:rustc-cfg=prod");
        }
    }
}
