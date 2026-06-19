pub mod coding;
pub mod bundling;

use bundling::ArchiveEntry;

use std::{
    fs,
    io::{self, Read},
    path::Path
};

/// Produce an archive given a list of files and directories.
pub fn create_archive<I, P>(paths: I) -> io::Result<Vec<u8>>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    let mut all_entries = Vec::new();

    for path in paths {
        let entries = read_entries(path)?;
        all_entries.extend(entries);
    }

    Ok(archive_from_entries(&all_entries))
}

pub fn archive_from_entries(entries: &[ArchiveEntry]) -> Vec<u8> {
    let bundle = bundling::bundle(entries);
    coding::compress(&bundle)
}

pub fn entries_from_archive(compressed: &[u8]) -> Option<Vec<ArchiveEntry>> {
    let decompressed = coding::decompress(compressed)?;
    bundling::extract(&decompressed)
}

pub fn read_entries(path: impl AsRef<Path>) -> io::Result<Vec<ArchiveEntry>> {
    let root = path.as_ref().to_path_buf();

    let (base, root_name) = if root.is_dir() {
        let parent = root.parent().unwrap_or_else(|| Path::new("."));

        let name = root.file_name().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "directory has no name")
        })?;

        (parent.to_path_buf(), name.to_os_string())
    } else {
        let parent = root.parent().unwrap_or_else(|| Path::new("."));

        let name = root.file_name().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "file has no name")
        })?;

        (parent.to_path_buf(), name.to_os_string())
    };

    let mut entries = Vec::new();
    let mut stack = vec![root];

    while let Some(current) = stack.pop() {
        let meta = fs::metadata(&current)?;

        let rel_path = if current == stack.first().cloned().unwrap_or_default() && stack.is_empty() {
            root_name.clone().into_string().map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, "non‑UTF‑8 root name")
            })?
        } else {
            current.strip_prefix(&base)
                .map_err(|_| io::Error::other("path stripping failed"))?
                .to_str()
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "non‑UTF‑8 path"))?
                .to_owned()
        };

        if meta.is_dir() {
            if !rel_path.is_empty() {
                entries.push(ArchiveEntry {
                    path: rel_path.as_bytes().to_vec(),
                    is_dir: true,
                    data: Vec::new(),
                });
            }

            for child in fs::read_dir(&current)? {
                let child = child?;
                stack.push(child.path());
            }
        } else {
            let mut data = Vec::new();
            fs::File::open(&current)?.read_to_end(&mut data)?;

            entries.push(ArchiveEntry {
                path: rel_path.as_bytes().to_vec(),
                is_dir: false,
                data,
            });
        }
    }

    Ok(entries)
}

pub fn write_entries_to_disk(entries: &[ArchiveEntry], output_dir: &Path) -> io::Result<()> {
    for entry in entries {
        let target = output_dir.join(
            std::str::from_utf8(&entry.path)
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "bad path"))?,
        );

        if entry.is_dir {
            fs::create_dir_all(&target)?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::write(&target, &entry.data)?;
        }
    }

    Ok(())
}
