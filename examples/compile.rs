use std::{fs::OpenOptions, io::Read, path::PathBuf};

pub mod ton {
    pub use ton_labs_assembler::*;
}

fn get_file() -> PathBuf {
    let mut args = std::env::args();
    args.next().expect("program-name CLA missing");
    let file = args
        .next()
        .unwrap_or_else(|| panic!("expected file path command-line argument, got nothing"));
    if args.len() > 0 {
        panic!(
            "expected exactly one argument, got `{}` followed by {} other(s)",
            file,
            args.len()
        )
    }
    let path = PathBuf::from(file);
    if !path.is_file() {
        panic!("`{}` is not a file or does not exist", path.display())
    }
    path
}

fn main() {
    let path = get_file();
    let path_str = path.display().to_string();
    let mut file = OpenOptions::new()
        .read(true)
        .open(&path)
        .unwrap_or_else(|e| panic!("failed to open `{}`: {}", path.display(), e));
    let content = {
        let mut buf = String::with_capacity(666);
        file.read_to_string(&mut buf)
            .unwrap_or_else(|e| panic!("failed to load `{}`: {}", path.display(), e));
        buf
    };
    let lines = {
        content
            .lines()
            .enumerate()
            .map(|(row, line)| ton::Line::new(line, &path_str, row + 1))
            .collect()
    };
    match ton::compile_code(&content) {
        Ok(slice) => {
            println!("slice data:");
            println!("{}", slice);
        }
        Err(e) => {
            panic!("compilation failed: {}", e)
        }
    }
    match ton::compile_code_debuggable(lines) {
        Ok((slice, info)) => {
            println!("slice data:");
            println!("{}", slice);
            println!("info:");
            println!("{:?}", info);
        }
        Err(e) => {
            panic!("compilation failed: {}", e)
        }
    }
}
