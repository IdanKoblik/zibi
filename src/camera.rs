pub fn list_cameras() -> Vec<String> {
    let mut cameras: Vec<String> = std::fs::read_dir("/dev")
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name.starts_with("video"))
        .map(|name| format!("/dev/{name}"))
        .collect();
    cameras.sort();

    if cameras.is_empty() {
        cameras.push("No camera found".to_string());
    }
    cameras
}

