//! `w3c-rs`: checks camera files and lists their cameras. See `usage`.

use std::process::ExitCode;

use w3c_rs::{Camera, Cameras, Layout, parse, parse_any};

fn usage() {
    eprintln!("usage: w3c-rs <war3map.w3c> [<war3map.w3i>]           check: read and write back");
    eprintln!(
        "       w3c-rs --list <war3map.w3c> [<war3map.w3i>]    print the cameras, tab-separated"
    );
    eprintln!();
    eprintln!("  The map's war3map.w3i says which layout the file has (1.31 added a local");
    eprintln!("  rotation); without it, the older layout is tried first, then the newer.");
}

fn read(path: &str, w3i: Option<&str>) -> Result<(Vec<u8>, Cameras), String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    let file = match w3i {
        Some(w3i) => {
            let info = std::fs::read(w3i).map_err(|e| format!("{w3i}: {e}"))?;
            let layout =
                Layout::from_w3i(&info).ok_or(format!("{w3i}: too short to say its version"))?;
            parse(&bytes, layout)
        }
        None => parse_any(&bytes),
    }
    .map_err(|e| format!("{path}: {e}"))?;
    Ok((bytes, file))
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let result = match args.as_slice() {
        ["--list", path, rest @ ..] if rest.len() <= 1 => {
            read(path, rest.first().copied()).map(|(_, file)| {
                // The local rotation only in the layout that has it.
                let count = if file.layout == Layout::Local { 13 } else { 10 };
                let fields = Camera::new("", 0.0, 0.0).fields();
                let names: Vec<&str> = fields[..count].iter().map(|(n, _)| *n).collect();
                println!("name\t{}", names.join("\t"));
                for c in &file.cameras {
                    let values: Vec<String> = c.fields()[..count]
                        .iter()
                        .map(|(_, v)| v.to_string())
                        .collect();
                    println!("{}\t{}", c.name, values.join("\t"));
                }
            })
        }
        [path, rest @ ..] if rest.len() <= 1 && !path.starts_with('-') => {
            read(path, rest.first().copied()).and_then(|(bytes, file)| {
                let written = file
                    .to_bytes()
                    .map_err(|e| format!("{path}: can't write it back: {e}"))?;
                if written != bytes {
                    return Err(format!("{path}: differs after writing"));
                }
                println!(
                    "{path}: {} cameras, layout for {}",
                    file.cameras.len(),
                    file.layout.describe()
                );
                Ok(())
            })
        }
        _ => {
            usage();
            return ExitCode::FAILURE;
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
