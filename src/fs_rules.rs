use std::path::{Component, Path};

const EXCLUDED_DIRS: &[&str] = &[".git", "node_modules", "target"];
const EXCLUDED_FILES: &[&str] = &[".now.json", ".DS_Store"];

pub fn is_excluded_path(root: &Path, path: &Path) -> bool {
    let relative = path.strip_prefix(root).unwrap_or(path);
    is_excluded_relative(relative)
}

pub fn is_excluded_relative(path: &Path) -> bool {
    for component in path.components() {
        let Component::Normal(value) = component else {
            continue;
        };
        let name = value.to_string_lossy();
        if EXCLUDED_DIRS.contains(&name.as_ref()) {
            return true;
        }
    }

    let Some(file_name) = path.file_name().map(|value| value.to_string_lossy()) else {
        return false;
    };

    EXCLUDED_FILES.contains(&file_name.as_ref())
        || file_name == ".env"
        || file_name.starts_with(".env.")
        || is_temp_name(&file_name)
}

fn is_temp_name(name: &str) -> bool {
    name.ends_with('~')
        || name.ends_with(".tmp")
        || name.ends_with(".temp")
        || name.ends_with(".swp")
        || name.ends_with(".bak")
        || name.starts_with(".#")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn excludes_runtime_and_temp_paths() {
        assert!(is_excluded_relative(Path::new(".now.json")));
        assert!(is_excluded_relative(Path::new("node_modules/pkg/index.js")));
        assert!(is_excluded_relative(Path::new("target/release/now")));
        assert!(is_excluded_relative(Path::new(".env")));
        assert!(is_excluded_relative(Path::new(".env.local")));
        assert!(is_excluded_relative(Path::new("index.html.tmp")));
        assert!(!is_excluded_relative(Path::new("public/index.html")));
    }

    #[test]
    fn excludes_paths_relative_to_root() {
        let root = PathBuf::from("/tmp/site");
        assert!(is_excluded_path(&root, Path::new("/tmp/site/.git/config")));
    }
}
