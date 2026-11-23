//! The open song file: load, save, export, and unsaved-changes tracking.

use std::path::{Path, PathBuf};

use model::{FILE_EXTENSION, PartFilter, Project};

pub struct Document {
    pub project: Project,
    pub path: Option<PathBuf>,
    /// The project as last loaded or saved, to detect unsaved changes.
    saved: Project,
}

impl Document {
    pub fn new() -> Self {
        Self { project: Project::default(), path: None, saved: Project::default() }
    }

    pub fn is_dirty(&self) -> bool {
        self.project != self.saved
    }

    /// Window title: file name plus a marker for unsaved changes.
    pub fn title(&self) -> String {
        let name = self
            .path
            .as_deref()
            .and_then(Path::file_name)
            .map_or("Untitled".to_string(), |n| n.to_string_lossy().into_owned());
        let dirty = if self.is_dirty() { " •" } else { "" };
        format!("{name}{dirty} — song2sheet")
    }

    pub fn open(path: &Path) -> Result<Self, String> {
        let json = std::fs::read_to_string(path)
            .map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
        let project = Project::from_json(&json).map_err(|e| e.to_string())?;
        Ok(Self { saved: project.clone(), project, path: Some(path.to_path_buf()) })
    }

    /// Saves via a temporary file so a failed write never destroys the
    /// previous version.
    pub fn save_to(&mut self, path: &Path) -> Result<(), String> {
        let err = |e: std::io::Error| format!("Cannot write {}: {e}", path.display());
        let mut tmp = path.as_os_str().to_owned();
        tmp.push(".tmp");
        std::fs::write(&tmp, self.project.to_json()).map_err(err)?;
        std::fs::rename(&tmp, path).map_err(err)?;
        self.saved = self.project.clone();
        self.path = Some(path.to_path_buf());
        Ok(())
    }

    /// Suggested file name stem for dialogs, from the song title, with
    /// characters that are invalid in file names replaced.
    pub fn file_stem(&self) -> String {
        let title = self.project.meta.title.trim();
        if title.is_empty() {
            return "song".to_string();
        }
        title.chars().map(|c| if r#"/\:*?"<>|"#.contains(c) { '-' } else { c }).collect()
    }

    pub fn export_pdf(&self, path: &Path, filter: PartFilter) -> Result<(), String> {
        let list = engrave::layout_sheet(&self.project, filter);
        let bytes = pdf::export(&list, &self.file_stem()).map_err(|e| e.to_string())?;
        std::fs::write(path, bytes).map_err(|e| format!("Cannot write {}: {e}", path.display()))
    }
}

/// Ensures `path` ends in `.s2s.json`, replacing a plain `.json`.
pub fn with_song_extension(path: PathBuf) -> PathBuf {
    let full = format!(".{FILE_EXTENSION}");
    let Some(name) = path.file_name().map(|n| n.to_string_lossy().into_owned()) else {
        return path;
    };
    if name.ends_with(&full) {
        return path;
    }
    let stem = name.strip_suffix(".json").unwrap_or(&name);
    path.with_file_name(format!("{stem}{full}"))
}

/// File dialogs. Each returns `None` if the user cancelled.
pub mod dialogs {
    use super::*;

    fn song_filter(d: rfd::FileDialog) -> rfd::FileDialog {
        d.add_filter("Song sheet", &[FILE_EXTENSION, "json"])
    }

    pub fn open() -> Option<PathBuf> {
        song_filter(rfd::FileDialog::new()).pick_file()
    }

    pub fn save(stem: &str) -> Option<PathBuf> {
        let dialog =
            song_filter(rfd::FileDialog::new()).set_file_name(format!("{stem}.{FILE_EXTENSION}"));
        dialog.save_file().map(with_song_extension)
    }

    pub fn export_pdf(stem: &str) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .add_filter("PDF", &["pdf"])
            .set_file_name(format!("{stem}.pdf"))
            .save_file()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ext(name: &str) -> String {
        with_song_extension(PathBuf::from("/x").join(name))
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned()
    }

    #[test]
    fn song_extension_is_added_or_fixed() {
        assert_eq!(ext("song.s2s.json"), "song.s2s.json");
        assert_eq!(ext("song"), "song.s2s.json");
        assert_eq!(ext("song.json"), "song.s2s.json");
        assert_eq!(ext("songs2s.json"), "songs2s.s2s.json");
    }

    #[test]
    fn file_stem_replaces_path_characters() {
        let mut doc = Document::new();
        doc.project.meta.title = "AC/DC: Live?".into();
        assert_eq!(doc.file_stem(), "AC-DC- Live-");
    }

    #[test]
    fn save_and_reopen_is_clean() {
        let dir = std::env::temp_dir().join(format!("s2s-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("t.s2s.json");
        let mut doc = Document::new();
        doc.project.meta.title = "T".into();
        assert!(doc.is_dirty());
        doc.save_to(&path).unwrap();
        assert!(!doc.is_dirty());
        let reopened = Document::open(&path).unwrap();
        assert!(!reopened.is_dirty());
        assert_eq!(reopened.project, doc.project);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
