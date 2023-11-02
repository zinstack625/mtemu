fn main() {
    eprintln!("{:?}", std::env::current_dir());
    println!("cargo:rustc-link-search=native={}/build/src/emulator/implementation", std::env::current_dir().unwrap().into_os_string().into_string().unwrap());
}
