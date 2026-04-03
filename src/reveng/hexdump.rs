use anyhow::Result;
use colored::Colorize;
use std::path::Path;

pub fn run(file: &Path, offset: u64, len: Option<usize>, width: usize) -> Result<()> {
    let data = std::fs::read(file)?;
    let start = offset as usize;
    let end = len
        .map(|l| (start + l).min(data.len()))
        .unwrap_or(data.len());
    let slice = &data[start.min(data.len())..end];
    print_hexdump(slice, start as u64, width);
    Ok(())
}

/// Print an xxd-style hex dump of `data` starting at logical `base_offset`.
pub fn print_hexdump(data: &[u8], base_offset: u64, width: usize) {
    for (row_idx, chunk) in data.chunks(width).enumerate() {
        let addr = base_offset + (row_idx * width) as u64;
        let hex: String = chunk
            .iter()
            .enumerate()
            .map(|(i, b)| {
                let s = format!("{:02x}", b);
                if i == width / 2 - 1 { format!("{} ", s) } else { format!("{} ", s) }
            })
            .collect();
        let ascii: String = chunk
            .iter()
            .map(|&b| if b.is_ascii_graphic() || b == b' ' { b as char } else { '.' })
            .collect();
        println!(
            "{}  {}  {}",
            format!("{:08x}", addr).cyan(),
            format!("{:width$}", hex, width = width * 3 + 1),
            ascii.green()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hexdump_does_not_panic_on_empty() {
        print_hexdump(&[], 0, 16);
    }

    #[test]
    fn hexdump_prints_ascii() {
        // smoke test: just ensure it runs without panic
        let data = b"Hello, world!";
        print_hexdump(data, 0, 16);
    }
}
