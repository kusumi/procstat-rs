pub fn is_regular_file(f: &str) -> bool {
    let m = match std::fs::metadata(f) {
        Ok(v) => v,
        Err(_) => return false,
    };
    m.file_type().is_file()
}

pub fn join_path(f1: &str, f2: &str) -> String {
    let p = std::path::Path::new(f1);
    p.join(f2).as_path().to_str().unwrap().to_string()
}
