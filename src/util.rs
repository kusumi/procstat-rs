use path_clean::PathClean;

macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        type_name_of(f).strip_suffix("::f").unwrap()
    }};
}
pub(crate) use function;

pub(crate) fn is_regular_file(f: &str) -> bool {
    if let Ok(v) = std::fs::metadata(f) {
        v.file_type().is_file()
    } else {
        false
    }
}

// This function
// * does not resolve symlink
// * works with non existent path
pub(crate) fn get_abspath(f: &str) -> std::io::Result<String> {
    let p = std::path::Path::new(f);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        std::env::current_dir()?.join(f)
    }
    .clean()
    .into_os_string()
    .into_string()
    .map_err(|_| std::io::Error::from(std::io::ErrorKind::InvalidInput))
}

// fails if f is "/" or equivalent
pub(crate) fn get_basename(f: &str) -> std::io::Result<String> {
    Ok(std::path::Path::new(&get_abspath(f)?)
        .file_name()
        .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::NotFound))?
        .to_str()
        .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::InvalidInput))?
        .to_string())
}

pub(crate) fn is_dir(f: &str) -> bool {
    if let Ok(v) = std::fs::metadata(f) {
        v.file_type().is_dir()
    } else {
        false
    }
}

pub(crate) fn join_path(f1: &str, f2: &str) -> String {
    std::path::Path::new(f1)
        .join(f2)
        .as_path()
        .to_str()
        .unwrap()
        .to_string()
}

pub(crate) fn get_home_path() -> String {
    home::home_dir()
        .unwrap()
        .into_os_string()
        .into_string()
        .unwrap()
}

pub(crate) fn error() -> std::io::Error {
    std::io::Error::from(std::io::ErrorKind::Other)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_get_abspath() {
        #[derive(Debug)]
        struct F {
            i: &'static str,
            o: &'static str,
        }
        let path_list = [
            F { i: "/", o: "/" },
            F { i: "/////", o: "/" },
            F { i: "/..", o: "/" },
            F { i: "/../", o: "/" },
            F {
                i: "/root",
                o: "/root",
            },
            F {
                i: "/root/",
                o: "/root",
            },
            F {
                i: "/root/..",
                o: "/",
            },
            F {
                i: "/root/../dev",
                o: "/dev",
            },
            F {
                i: "/does/not/exist",
                o: "/does/not/exist",
            },
            F {
                i: "/does/not/./exist",
                o: "/does/not/exist",
            },
            F {
                i: "/does/not/../NOT/exist",
                o: "/does/NOT/exist",
            },
        ];
        for x in &path_list {
            match super::get_abspath(x.i) {
                Ok(v) => assert_eq!(v, x.o),
                Err(e) => panic!("{e} {x:?}"),
            }
        }
    }
}
