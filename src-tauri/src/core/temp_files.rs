/// 清空临时文件目录
pub fn clean() {
    let mut temp_dir_path = std::env::temp_dir();
    temp_dir_path.push("cutliner");
    if temp_dir_path.exists() {
        if let Err(e) = std::fs::remove_dir_all(&temp_dir_path) {
            eprintln!("failed to remove temp dir {:?}: {}", temp_dir_path, e);
        }
    }
}
