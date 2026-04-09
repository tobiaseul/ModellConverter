use std::io::Write as _;
use tempfile::TempDir;
use modell_converter::convert;
use modell_converter::format::Format;

#[test]
fn batch_zip_converts_all_matching_entries() {
    let output_dir = TempDir::new().unwrap();
    let zip_dir = TempDir::new().unwrap();
    let zip_path = zip_dir.path().join("models.zip");

    let fixture = include_bytes!("fixtures/sample_edgetx.yml");

    // Build a zip in memory with two EdgeTX entries and one non-matching
    {
        let file = std::fs::File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();
        zip.start_file("model_a.yml", options).unwrap();
        zip.write_all(fixture).unwrap();
        zip.start_file("model_b.yml", options).unwrap();
        zip.write_all(fixture).unwrap();
        zip.start_file("readme.txt", options).unwrap();
        zip.write_all(b"ignore").unwrap();
        zip.finish().unwrap();
    }

    convert::run_batch(Format::Edgetx, Format::Edgetx, &zip_path, output_dir.path()).unwrap();

    let out_a = output_dir.path().join("model_a.yml");
    let out_b = output_dir.path().join("model_b.yml");
    assert!(out_a.exists(), "model_a.yml should be converted");
    assert!(out_b.exists(), "model_b.yml should be converted");
    assert!(std::fs::metadata(&out_a).unwrap().len() > 0);
    assert!(std::fs::metadata(&out_b).unwrap().len() > 0);
    assert!(!output_dir.path().join("readme.txt").exists());
}
