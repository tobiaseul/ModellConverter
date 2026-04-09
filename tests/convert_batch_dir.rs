use std::fs;
use tempfile::TempDir;
use modell_converter::convert;
use modell_converter::format::Format;

#[test]
fn batch_dir_converts_all_matching_files() {
    let input_dir = TempDir::new().unwrap();
    let output_dir = TempDir::new().unwrap();

    // Copy two EdgeTX fixtures into input dir
    let fixture = include_bytes!("fixtures/sample_edgetx.yml");
    fs::write(input_dir.path().join("model_a.yml"), fixture).unwrap();
    fs::write(input_dir.path().join("model_b.yml"), fixture).unwrap();
    // A non-matching file that should be ignored
    fs::write(input_dir.path().join("readme.txt"), b"ignore me").unwrap();

    convert::run_batch(Format::Edgetx, Format::Edgetx, input_dir.path(), output_dir.path()).unwrap();

    let out_a = output_dir.path().join("model_a.yml");
    let out_b = output_dir.path().join("model_b.yml");
    assert!(out_a.exists(), "model_a.yml should be converted");
    assert!(out_b.exists(), "model_b.yml should be converted");
    assert!(fs::metadata(&out_a).unwrap().len() > 0);
    assert!(fs::metadata(&out_b).unwrap().len() > 0);

    // Non-matching file must not appear in output
    assert!(!output_dir.path().join("readme.txt").exists());
}
