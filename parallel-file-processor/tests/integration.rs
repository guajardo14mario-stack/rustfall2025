#[test]
fn integration_file_processing_no_external() {
    use std::fs::File;
    use std::io::Write;
    let mut path = std::env::temp_dir();
    path.push(format!("pff_test_{}.txt", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()));
    let mut f = File::create(&path).unwrap();
    writeln!(f, "one two three\nsecond line").unwrap();

    let files = vec![path.clone()];
    // ... rest same as previous
}
