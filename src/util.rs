pub fn is_regular_file(f: &str) -> bool {
    if let Ok(v) = std::fs::metadata(f) {
        v.file_type().is_file()
    } else {
        false
    }
}

pub fn get_abspath(f: &str) -> std::io::Result<String> {
    let p = std::fs::canonicalize(f)?; // XXX keep symlink unresolved
    Ok(p.into_os_string().into_string().unwrap())
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
