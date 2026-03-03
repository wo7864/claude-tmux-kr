//! Favorites persistence
//!
//! Stores favorite session names as a plain text file (one name per line)
//! in the platform config directory (`claude-tmux/favorites`).

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use anyhow::Result;

/// Return the path to the favorites file.
fn favorites_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("claude-tmux").join("favorites"))
}

/// Load favorites from disk. Returns an empty set on any error.
pub fn load_favorites() -> HashSet<String> {
    let Some(path) = favorites_path() else {
        return HashSet::new();
    };
    let Ok(content) = fs::read_to_string(&path) else {
        return HashSet::new();
    };
    content
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect()
}

/// Save favorites to disk (sorted, one per line). Creates parent dirs if needed.
pub fn save_favorites(favorites: &HashSet<String>) -> Result<()> {
    let path = favorites_path().ok_or_else(|| anyhow::anyhow!("config dir not found"))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut names: Vec<&String> = favorites.iter().collect();
    names.sort();
    let content = names
        .into_iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&path, content)?;
    Ok(())
}
