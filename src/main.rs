use std::path::Path;
use huffman_archiver;

const INSTRUCTIONS: &'static str = "\
    Usage:\n  \
      huffman_archiver -c <output name> <files/directories>\n  \
      huffman_archiver -x <archive filename> <output directory>\
";

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 4 {
        println!("{INSTRUCTIONS}");
        return Ok(());
    }

    match args[1].as_str() {
        "-c" => {
            let output = &args[2];
            let paths = &args[3..];

            let archive = huffman_archiver::create_archive(paths)?;
            std::fs::write(output, archive)?;
            println!("Archive written to {}", output);
        },
        "-x" => {
            let archive_file = &args[2];
            let output_dir = &args[3];
            let data = std::fs::read(archive_file)?;

            let entries = huffman_archiver::entries_from_archive(&data)
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "decompression failed"))?;

            huffman_archiver::write_entries_to_disk(&entries, Path::new(&output_dir))?;
            println!("Archive extracted to {}", output_dir);
        },
        _ => {
            println!("{INSTRUCTIONS}");
        }
    }

    Ok(())
}
