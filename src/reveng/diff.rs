use anyhow::Result;
use colored::Colorize;
use std::path::Path;

pub fn run(file_a: &Path, file_b: &Path, context: Option<usize>) -> Result<()> {
    let a = std::fs::read(file_a)?;
    let b = std::fs::read(file_b)?;
    print_diff(&a, &b, context);
    Ok(())
}

/// Byte-level diff of two slices. Highlights differing regions with optional
/// `context` bytes of surrounding identical bytes shown around each diff.
pub fn print_diff(a: &[u8], b: &[u8], context: Option<usize>) {
    let len = a.len().max(b.len());
    if len == 0 {
        println!("(both files are empty)");
        return;
    }

    // Build a list of (offset, same: bool) spans
    let mut spans: Vec<(usize, usize, bool)> = Vec::new(); // (start, end_exclusive, same)
    let mut i = 0;
    while i < len {
        let same = i < a.len() && i < b.len() && a[i] == b[i];
        let start = i;
        while i < len {
            let cur_same = i < a.len() && i < b.len() && a[i] == b[i];
            if cur_same != same {
                break;
            }
            i += 1;
        }
        spans.push((start, i, same));
    }

    let ctx = context.unwrap_or(0);

    for (span_idx, &(start, end, same)) in spans.iter().enumerate() {
        if same {
            let run_len = end - start;
            if ctx == 0 || run_len <= ctx * 2 {
                // Show the whole identical run if small or no context requested
                if context.is_some() {
                    print_region(a, b, start, end, true);
                }
            } else {
                // Suppress middle, show border context
                let show_start = start;
                let show_end = start + ctx;
                let show_start2 = end - ctx;
                let show_end2 = end;

                // Only show leading context if there's a diff before us
                if span_idx > 0 {
                    print_region(a, b, show_start, show_end, true);
                }
                let suppressed = show_start2 - show_end;
                if suppressed > 0 {
                    println!("{}", format!("  ... {} identical bytes ...", suppressed).dimmed());
                }
                // Only show trailing context if there's a diff after us
                if span_idx + 1 < spans.len() {
                    print_region(a, b, show_start2, show_end2, true);
                }
            }
        } else {
            print_region(a, b, start, end, false);
        }
    }

    let diff_count = spans.iter().filter(|&&(_, _, same)| !same).count();
    if diff_count == 0 {
        println!("{}", "Files are identical.".green());
    }
}

fn print_region(a: &[u8], b: &[u8], start: usize, end: usize, same: bool) {
    let width = 16;
    let label = if same {
        format!("[{:#010x}..{:#010x}] SAME", start, end - 1).dimmed().to_string()
    } else {
        format!("[{:#010x}..{:#010x}] DIFFER  ({} bytes)", start, end - 1, end - start)
            .red()
            .bold()
            .to_string()
    };
    println!("{}", label);

    if same {
        return;
    }

    for chunk_start in (start..end).step_by(width) {
        let chunk_end = (chunk_start + width).min(end);
        let addr = format!("  {:#010x}", chunk_start).cyan();

        let hex_a = format_hex_row(a, chunk_start, chunk_end, width);
        let hex_b = format_hex_row(b, chunk_start, chunk_end, width);
        let ascii_a = format_ascii_row(a, chunk_start, chunk_end);
        let ascii_b = format_ascii_row(b, chunk_start, chunk_end);

        println!("{}  A: {}  |{}|", addr, hex_a.red(), ascii_a.red());
        println!("{}  B: {}  |{}|", addr, hex_b.green(), ascii_b.green());
    }
}

fn format_hex_row(data: &[u8], start: usize, end: usize, width: usize) -> String {
    let mut s = String::new();
    for i in start..end {
        if i < data.len() {
            s.push_str(&format!("{:02x} ", data[i]));
        } else {
            s.push_str("   ");
        }
    }
    // Pad to full width
    let used = (end - start) * 3;
    let total = width * 3;
    for _ in used..total {
        s.push(' ');
    }
    s
}

fn format_ascii_row(data: &[u8], start: usize, end: usize) -> String {
    (start..end)
        .map(|i| {
            if i < data.len() {
                let b = data[i];
                if b.is_ascii_graphic() || b == b' ' { b as char } else { '.' }
            } else {
                ' '
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_files() {
        // Should not panic
        print_diff(&[1, 2, 3], &[1, 2, 3], None);
    }

    #[test]
    fn different_files() {
        print_diff(&[0x41, 0x42, 0x43], &[0x41, 0xFF, 0x43], Some(1));
    }

    #[test]
    fn different_lengths() {
        print_diff(&[1, 2, 3, 4], &[1, 2], None);
    }
}
