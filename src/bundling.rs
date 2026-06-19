#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchiveEntry {
    pub path: Vec<u8>,
    pub is_dir: bool,
    pub data: Vec<u8>
}

/// Produce a single file from archive entries.
pub fn bundle(entries: &[ArchiveEntry]) -> Vec<u8> {
    let mut buf = Vec::new();

    for entry in entries {
        // path length (u16) + contents
        let path_len = entry.path.len() as u16;
        buf.extend_from_slice(&path_len.to_le_bytes());
        buf.extend_from_slice(&entry.path);

        // is_dir flag (u8)
        buf.push(entry.is_dir.then_some(1).unwrap_or(0));

        if !entry.is_dir {
            // data length (u64) + data
            let data_len = entry.data.len() as u64;
            buf.extend_from_slice(&data_len.to_le_bytes());
            buf.extend_from_slice(&entry.data);
        }
    }

    // end marker (path of 0 length)
    buf.extend_from_slice(&0u16.to_le_bytes());
    buf
}

/// Extract the archive entries from a file.
pub fn extract(archive: &[u8]) -> Option<Vec<ArchiveEntry>> {
    if archive.len() < 2 {
        return None;
    }

    let mut pos = 0;
    let mut entries = Vec::new();

    loop {
        if pos + 2 > archive.len() {
            return None;
        }

        let path_len = u16::from_le_bytes([archive[pos], archive[pos + 1]]) as usize;
        pos += 2;

        if path_len == 0 {
            break; // end marker
        }

        if pos + path_len + 1 > archive.len() {
            return None;
        }

        let path = archive[pos..pos+path_len].to_vec();
        pos += path_len;

        let is_dir = archive[pos] == 1;
        pos += 1;

        let mut data = Vec::new();

        if !is_dir {
            if pos + 8 > archive.len() {
                return None;
            }

            let data_len = u64::from_le_bytes([
                archive[pos],
                archive[pos + 1],
                archive[pos + 2],
                archive[pos + 3],
                archive[pos + 4],
                archive[pos + 5],
                archive[pos + 6],
                archive[pos + 7],
            ]) as usize;

            pos += 8;

            if pos + data_len > archive.len() {
                return None;
            }

            data = archive[pos..pos+data_len].to_vec();
            pos += data_len;
        }

        entries.push(ArchiveEntry {
            path,
            is_dir,
            data
        });
    }

    Some(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    /// Generate an arbitrary `ArchiveEntry`.
    fn arb_entry() -> impl Strategy<Value = ArchiveEntry> {
        let raw = (
            prop::collection::vec(any::<u8>(), 1..100), // path
            any::<bool>(), // is_dir
            prop::collection::vec(any::<u8>(), 0..200), // data
        );

        raw.prop_map(|(path, is_dir, data)| ArchiveEntry {
            path,
            is_dir,
            data: if is_dir {
                vec![]
            } else {
                data
            }
        })
    }

    proptest! {
        #[test]
        fn roundtrip_random_entries(entries in prop::collection::vec(arb_entry(), 0..30)) {
            let bundled = bundle(&entries);
            let extracted = extract(&bundled).expect("extraction failed");
            prop_assert_eq!(extracted.len(), entries.len());

            for (orig, ext) in entries.iter().zip(extracted.iter()) {
                prop_assert_eq!(&orig.path, &ext.path);
                prop_assert_eq!(orig.is_dir, ext.is_dir);
                prop_assert_eq!(&orig.data, &ext.data);
            }
        }
    }

    #[test]
    fn empty_archive() {
        let bundled = bundle(&[]);
        assert_eq!(bundled, vec![0u8, 0]); // end marker
        assert!(extract(&bundled).unwrap().is_empty());
    }

    #[test]
    fn single_file() {
        let entry = ArchiveEntry {
            path: b"foo.bar".to_vec(),
            is_dir: false,
            data: vec![1, 2, 3]
        };

        let bundled = bundle(&[entry.clone()]);
        let extracted = extract(&bundled).unwrap();
        assert_eq!(extracted, vec![entry]);
    }

    #[test]
    fn single_dir() {
        let entry = ArchiveEntry {
            path: b"foo".to_vec(),
            is_dir: true,
            data: vec![]
        };

        let bundled = bundle(&[entry.clone()]);
        let extracted = extract(&bundled).unwrap();
        assert_eq!(extracted, vec![entry]);
    }

    #[test]
    fn truncated_fails() {
        let entry = ArchiveEntry {
            path: b"x".to_vec(),
            is_dir: false,
            data: vec![0; 10]
        };

        let mut bundled = bundle(&[entry]);
        bundled.truncate(bundled.len() - 5); // cut off some data
        assert!(extract(&bundled).is_none());
    }
}
