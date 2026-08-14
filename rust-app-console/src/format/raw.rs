pub fn sanitize_raw_path(path: &str) -> String {
    let clean = path.trim_start_matches('/');
    format!("/{}", clean)
}
