//! Project-based session grouping
//!
//! Groups sessions by their project root directory so that sessions
//! from the same git repository (including worktrees) are displayed together.

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use crate::session::{ClaudeCodeStatus, Session};

/// An item in the grouped view's visual list
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupedItem {
    /// A group header row
    GroupHeader { group_index: usize },
    /// A session row within a group
    Session {
        group_index: usize,
        session_index: usize,
    },
}

/// A project group containing multiple sessions
#[derive(Debug, Clone)]
pub struct ProjectGroup {
    /// The project root path used as group key
    pub project_root: PathBuf,
    /// Display name (shortened path with ~ substitution)
    pub display_name: String,
    /// Whether this group is collapsed
    pub collapsed: bool,
    /// Indices into the filtered sessions list
    pub session_indices: Vec<usize>,
}

/// Manages the grouped view state
pub struct GroupedView {
    /// All project groups
    pub groups: Vec<ProjectGroup>,
    /// Whether grouped view is enabled
    pub enabled: bool,
    /// Set of collapsed project roots (preserved across rebuilds)
    collapsed_roots: HashSet<PathBuf>,
}

impl Default for GroupedView {
    fn default() -> Self {
        Self::new()
    }
}

impl GroupedView {
    pub fn new() -> Self {
        Self {
            groups: Vec::new(),
            enabled: true,
            collapsed_roots: HashSet::new(),
        }
    }

    /// Toggle grouped view on/off
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    /// Rebuild groups from the current filtered session list.
    /// Preserves collapsed state across rebuilds.
    /// Favorite sessions are collected into a special "★ 즐겨찾기" group at the top.
    pub fn rebuild(&mut self, sessions: &[&Session], favorites: &HashSet<String>) {
        let fav_root: PathBuf = PathBuf::from("★ 즐겨찾기");

        // Separate favorite and non-favorite session indices
        let mut fav_indices: Vec<usize> = Vec::new();
        let mut non_fav_indices: Vec<usize> = Vec::new();
        for (i, session) in sessions.iter().enumerate() {
            if favorites.contains(&session.name) {
                fav_indices.push(i);
            } else {
                non_fav_indices.push(i);
            }
        }

        // Group non-favorite sessions by project root
        let mut groups_map: BTreeMap<PathBuf, Vec<usize>> = BTreeMap::new();
        for &i in &non_fav_indices {
            let root = sessions[i].project_root();
            groups_map.entry(root).or_default().push(i);
        }

        // Convert to ProjectGroup vec
        let mut groups: Vec<ProjectGroup> = groups_map
            .into_iter()
            .map(|(root, indices)| {
                let display_name = shorten_path(&root);
                let collapsed = self.collapsed_roots.contains(&root);
                ProjectGroup {
                    project_root: root,
                    display_name,
                    collapsed,
                    session_indices: indices,
                }
            })
            .collect();

        // Sort: groups with connected (Working/WaitingInput) sessions first,
        // then alphabetically by display_name
        groups.sort_by(|a, b| {
            let a_has_active = a
                .session_indices
                .iter()
                .any(|&i| has_active_status(sessions, i));
            let b_has_active = b
                .session_indices
                .iter()
                .any(|&i| has_active_status(sessions, i));

            // Active groups first
            match (a_has_active, b_has_active) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.display_name.cmp(&b.display_name),
            }
        });

        // Insert favorites group at the top if there are any
        if !fav_indices.is_empty() {
            let collapsed = self.collapsed_roots.contains(&fav_root);
            groups.insert(
                0,
                ProjectGroup {
                    project_root: fav_root,
                    display_name: "★ 즐겨찾기".to_string(),
                    collapsed,
                    session_indices: fav_indices,
                },
            );
        }

        self.groups = groups;
    }

    /// Total visible items (headers + expanded sessions)
    pub fn visible_item_count(&self) -> usize {
        let mut count = 0;
        for group in &self.groups {
            count += 1; // header
            if !group.collapsed {
                count += group.session_indices.len();
            }
        }
        count
    }

    /// Map a visual index to a GroupedItem
    pub fn item_at(&self, visual_index: usize) -> Option<GroupedItem> {
        let mut pos = 0;
        for (gi, group) in self.groups.iter().enumerate() {
            if pos == visual_index {
                return Some(GroupedItem::GroupHeader { group_index: gi });
            }
            pos += 1;

            if !group.collapsed {
                for (si, _) in group.session_indices.iter().enumerate() {
                    if pos == visual_index {
                        return Some(GroupedItem::Session {
                            group_index: gi,
                            session_index: si,
                        });
                    }
                    pos += 1;
                }
            }
        }
        None
    }

    /// Toggle collapse state for a group
    pub fn toggle_group(&mut self, group_index: usize) {
        if let Some(group) = self.groups.get_mut(group_index) {
            group.collapsed = !group.collapsed;
            if group.collapsed {
                self.collapsed_roots.insert(group.project_root.clone());
            } else {
                self.collapsed_roots.remove(&group.project_root);
            }
        }
    }

    /// Get the visual index of a group header
    pub fn visual_index_of_group(&self, group_index: usize) -> usize {
        let mut pos = 0;
        for (gi, group) in self.groups.iter().enumerate() {
            if gi == group_index {
                return pos;
            }
            pos += 1; // header
            if !group.collapsed {
                pos += group.session_indices.len();
            }
        }
        pos
    }

    /// Get the filtered session index for a GroupedItem::Session
    pub fn session_index_for(&self, item: GroupedItem) -> Option<usize> {
        match item {
            GroupedItem::Session {
                group_index,
                session_index,
            } => self
                .groups
                .get(group_index)
                .and_then(|g| g.session_indices.get(session_index))
                .copied(),
            GroupedItem::GroupHeader { .. } => None,
        }
    }
}

/// Check if a session at the given index has active status
fn has_active_status(sessions: &[&Session], index: usize) -> bool {
    sessions.get(index).is_some_and(|s| {
        matches!(
            s.claude_code_status,
            ClaudeCodeStatus::Working | ClaudeCodeStatus::WaitingInput
        )
    })
}

/// Shorten a path for display (replace home dir with ~)
fn shorten_path(path: &Path) -> String {
    if let Some(home) = dirs::home_dir() {
        if let Ok(stripped) = path.strip_prefix(&home) {
            return format!("~/{}", stripped.display());
        }
    }
    path.display().to_string()
}
