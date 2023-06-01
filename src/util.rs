pub fn is_regular_file(f: &str) -> bool {
    match std::fs::metadata(f) {
        Ok(v) => v.file_type().is_file(),
        Err(_) => false,
    }
}

pub fn join_path(f1: &str, f2: &str) -> String {
    let p = std::path::Path::new(f1);
    p.join(f2).as_path().to_str().unwrap().to_string()
}

pub fn get_home_path() -> String {
    dirs::home_dir()
        .unwrap()
        .into_os_string()
        .into_string()
        .unwrap()
}

pub fn error() -> std::io::Error {
    std::io::Error::from(std::io::ErrorKind::Other)
}
