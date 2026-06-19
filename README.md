# Huffman archiver

![test and lint status](https://img.shields.io/github/actions/workflow/status/maxicot/huffman_archiver/ci.yml?label=tests+%26+lints)
[![docs.rs status](https://img.shields.io/docsrs/huffman_archiver)](https://docs.rs/huffman_archiver)

A minimal archiver based on Huffman coding.

## CLI usage

Creating an archive:

```console
huffman_archiver -c <output name> <files/directories>
```

Extracting an archive's contents:

```console
huffman_archiver -x <archive filename> <output directory>
```

Example:

```console
huffman_archiver -c archive.harc foo.txt bar
huffman_archiver -x archive.harc foobar
```

## Building the CLI utility

Use `cargo run` (specifying the release profile to apply optimizations) in the project directory:

```console
cargo run -r
```

The binary can be found in `target/release`.

Make sure you [have a Rust toolchain installed](https://rust-lang.org/tools/install).
