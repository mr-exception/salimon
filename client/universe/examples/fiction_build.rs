//! Offline adapter. Runtime never chooses a seed or regenerates publication.
use salimon_universe::{
    Catalog, MAX_SCHEMA_JSON_BYTES,
    synthesis::{Request, build},
};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

fn read(path: &Path, limit: usize) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut bytes = vec![];
    fs::File::open(path)?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(format!("{} exceeds {} bytes", path.display(), limit).into());
    }
    Ok(bytes)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 3 {
        return Err("usage: fiction_build BASE_CATALOG.json REQUEST.json OUTPUT.json".into());
    }
    let base = Catalog::decode(&read(Path::new(&args[0]), MAX_SCHEMA_JSON_BYTES)?)?;
    let request: Request = serde_json::from_slice(&read(Path::new(&args[1]), 1024 * 1024)?)?;
    let publication = build(&base, request)?;
    let bytes = publication.encode()?;
    let output = PathBuf::from(&args[2]);
    if output.exists()
        && (fs::canonicalize(&output)? == fs::canonicalize(&args[0])?
            || fs::canonicalize(&output)? == fs::canonicalize(&args[1])?)
    {
        return Err("output must differ from input paths".into());
    }
    let mut temporary = output.as_os_str().to_owned();
    temporary.push(format!(".{}.tmp", std::process::id()));
    let temporary = PathBuf::from(temporary);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    let result = (|| -> std::io::Result<()> {
        file.write_all(&bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, &output)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result?;
    eprintln!(
        "{} systems; {} bytes; publication SHA256 {}",
        publication.sources().len(),
        bytes.len(),
        publication.sha256()?
    );
    Ok(())
}
