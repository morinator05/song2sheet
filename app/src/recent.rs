//! The list of recently opened or saved song files, kept between runs.

use std::path::{Path, PathBuf};

/// How many files the list remembers.
const MAX_RECENT: usize = 8;
/// Key in eframe's app storage.
const STORAGE_KEY: &str = "recent_files";

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RecentFiles {
    /// Most recent first.
    paths: Vec<PathBuf>,
}

impl RecentFiles {
    pub fn load(storage: Option<&dyn eframe::Storage>) -> Self {
        let mut paths: Vec<PathBuf> =
            storage.and_then(|s| eframe::get_value(s, STORAGE_KEY)).unwrap_or_default();
        paths.truncate(MAX_RECENT);
        Self { paths }
    }

    pub fn save(&self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, STORAGE_KEY, &self.paths);
    }

    /// Moves `path` to the front.
    pub fn add(&mut self, path: &Path) {
        self.remove(path);
        self.paths.insert(0, path.to_path_buf());
        self.paths.truncate(MAX_RECENT);
    }

    /// Forgets `path`, e.g. because it no longer exists.
    pub fn remove(&mut self, path: &Path) {
        self.paths.retain(|p| p != path);
    }

    pub fn paths(&self) -> &[PathBuf] {
        &self.paths
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn most_recent_first_without_duplicates() {
        let mut r = RecentFiles::default();
        r.add(Path::new("/a"));
        r.add(Path::new("/b"));
        r.add(Path::new("/a"));
        assert_eq!(r.paths(), [PathBuf::from("/a"), PathBuf::from("/b")]);
    }

    #[test]
    fn keeps_at_most_the_limit() {
        let mut r = RecentFiles::default();
        for i in 0..MAX_RECENT + 3 {
            r.add(Path::new(&format!("/song{i}")));
        }
        assert_eq!(r.paths().len(), MAX_RECENT);
        assert_eq!(r.paths()[0], PathBuf::from(format!("/song{}", MAX_RECENT + 2)));
    }
}
