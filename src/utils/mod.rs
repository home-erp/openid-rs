use std::path::PathBuf;

fn push_all(p: &mut PathBuf, sub_paths: &[&str]) {
    for sub_path in sub_paths {
        p.push(sub_path);
    }
}


pub fn get_path(base: &PathBuf, sub_paths: &[&str]) -> PathBuf {
    let mut cloned = base.clone();
    push_all(&mut cloned, sub_paths);
    cloned
}
