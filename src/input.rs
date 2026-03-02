use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, CreatePullRequestField, Mode, NewSessionField, NewWorktreeField};
use crate::group::GroupedItem;

/// Handle a key event and update the application state
pub fn handle_key(app: &mut App, key: KeyEvent) {
    // Clear messages on any key press
    app.clear_messages();

    match &app.mode {
        Mode::Normal => handle_normal_mode(app, key),
        Mode::ActionMenu => handle_action_menu_mode(app, key),
        Mode::Filter { .. } => handle_filter_mode(app, key),
        Mode::Search { .. } => handle_search_mode(app, key),
        Mode::ConfirmAction => handle_confirm_action_mode(app, key),
        Mode::NewSession { .. } => handle_new_session_mode(app, key),
        Mode::Rename { .. } => handle_rename_mode(app, key),
        Mode::Commit { .. } => handle_commit_mode(app, key),
        Mode::NewWorktree { .. } => handle_new_worktree_mode(app, key),
        Mode::CreatePullRequest { .. } => handle_create_pr_mode(app, key),
        Mode::Help => handle_help_mode(app, key),
    }
}

fn handle_normal_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        // Quit
        KeyCode::Char('q') | KeyCode::Esc => {
            app.should_quit = true;
        }

        // Navigation
        KeyCode::Char('j') | KeyCode::Down => {
            app.select_next();
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.select_prev();
        }

        // Enter action menu / expand group
        KeyCode::Char('l') | KeyCode::Right => {
            if app.grouped_view.enabled {
                match app.grouped_selected_item() {
                    Some(GroupedItem::GroupHeader { group_index }) => {
                        // Expand collapsed group
                        if app.grouped_view.groups.get(group_index).is_some_and(|g| g.collapsed) {
                            app.grouped_view.toggle_group(group_index);
                        }
                    }
                    Some(GroupedItem::Session { .. }) => {
                        app.enter_action_menu();
                    }
                    None => {}
                }
            } else {
                app.enter_action_menu();
            }
        }

        // Collapse group / go back
        KeyCode::Char('h') | KeyCode::Left => {
            if app.grouped_view.enabled {
                match app.grouped_selected_item() {
                    Some(GroupedItem::Session { group_index, .. }) => {
                        // Move to the group header and collapse
                        let header_pos = app.grouped_view.visual_index_of_group(group_index);
                        app.grouped_selected = header_pos;
                        app.grouped_view.toggle_group(group_index);
                        // Ensure collapsed state is reflected
                        if !app.grouped_view.groups.get(group_index).is_some_and(|g| g.collapsed) {
                            app.grouped_view.toggle_group(group_index);
                        }
                        app.sync_selected_from_grouped();
                        app.update_preview();
                    }
                    Some(GroupedItem::GroupHeader { group_index }) => {
                        // Collapse if expanded
                        if app.grouped_view.groups.get(group_index).is_some_and(|g| !g.collapsed) {
                            app.grouped_view.toggle_group(group_index);
                        }
                    }
                    None => {}
                }
            }
        }

        // Switch to session / toggle group (quick action)
        KeyCode::Enter => {
            if app.grouped_view.enabled {
                match app.grouped_selected_item() {
                    Some(GroupedItem::GroupHeader { group_index }) => {
                        app.grouped_view.toggle_group(group_index);
                        // After expanding, clamp grouped_selected
                        let count = app.grouped_view.visible_item_count();
                        if app.grouped_selected >= count && count > 0 {
                            app.grouped_selected = count - 1;
                        }
                    }
                    Some(GroupedItem::Session { .. }) => {
                        app.switch_to_selected();
                    }
                    None => {}
                }
            } else {
                app.switch_to_selected();
            }
        }

        // Toggle grouped view
        KeyCode::Char('g') => {
            app.toggle_grouped_view();
        }

        // New session
        KeyCode::Char('n') => {
            app.start_new_session();
        }

        // Kill session (capital K to avoid accidents)
        KeyCode::Char('K') => {
            app.start_kill();
        }

        // Rename session
        KeyCode::Char('r') => {
            app.start_rename();
        }

        // Filter
        KeyCode::Char('/') => {
            app.start_filter();
        }

        // Real-time search (k9s-style)
        KeyCode::Char(':') => {
            app.start_search();
        }

        // Clear filter
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.clear_filter();
        }

        // Refresh
        KeyCode::Char('R') => {
            app.refresh();
        }

        // Help
        KeyCode::Char('?') => {
            app.show_help();
        }

        _ => {}
    }
}

fn handle_filter_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.cancel();
        }
        KeyCode::Enter => {
            app.apply_filter();
        }
        KeyCode::Backspace => {
            if let Mode::Filter { ref mut input } = app.mode {
                input.pop();
            }
        }
        KeyCode::Char(c) => {
            if let Mode::Filter { ref mut input } = app.mode {
                input.push(c);
            }
        }
        _ => {}
    }
}

fn handle_search_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.cancel_search();
        }
        KeyCode::Enter => {
            // Confirm search: keep current filter, return to normal
            app.mode = Mode::Normal;
        }
        KeyCode::Backspace => {
            if let Mode::Search { ref mut input } = app.mode {
                input.pop();
            }
            app.update_search_filter();
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.select_next();
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.select_prev();
        }
        KeyCode::Char(c) => {
            if let Mode::Search { ref mut input } = app.mode {
                input.push(c);
            }
            app.update_search_filter();
        }
        _ => {}
    }
}

fn handle_action_menu_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        // Navigate actions
        KeyCode::Char('j') | KeyCode::Down => {
            app.select_next_action();
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.select_prev_action();
        }

        // Execute selected action
        KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
            app.execute_selected_action();
        }

        // Back to session list
        KeyCode::Char('h') | KeyCode::Left | KeyCode::Esc => {
            app.cancel();
        }

        // Quit entirely
        KeyCode::Char('q') => {
            app.should_quit = true;
        }

        _ => {}
    }
}

fn handle_confirm_action_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => {
            app.confirm_action();
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.cancel();
        }
        _ => {}
    }
}

fn handle_new_session_mode(app: &mut App, key: KeyEvent) {
    // Get current field and worktree state to determine behavior
    let (current_field, worktree_enabled) =
        if let Mode::NewSession { field, worktree_enabled, .. } = &app.mode {
            (*field, *worktree_enabled)
        } else {
            return;
        };

    match key.code {
        KeyCode::Esc => {
            app.cancel();
        }
        // Ctrl+W: toggle worktree mode
        KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.toggle_new_session_worktree();
        }
        KeyCode::Tab => {
            // Cycle through fields: Name → Path → (Branch if worktree) → Name
            if let Mode::NewSession {
                ref mut field,
                worktree_enabled,
                ..
            } = app.mode
            {
                *field = match field {
                    NewSessionField::Name => NewSessionField::Path,
                    NewSessionField::Path => {
                        if worktree_enabled {
                            NewSessionField::Branch
                        } else {
                            NewSessionField::Name
                        }
                    }
                    NewSessionField::Branch => NewSessionField::Name,
                };
            }
        }
        KeyCode::BackTab => {
            // Cycle backwards through fields
            if let Mode::NewSession {
                ref mut field,
                worktree_enabled,
                ..
            } = app.mode
            {
                *field = match field {
                    NewSessionField::Name => {
                        if worktree_enabled {
                            NewSessionField::Branch
                        } else {
                            NewSessionField::Path
                        }
                    }
                    NewSessionField::Path => NewSessionField::Name,
                    NewSessionField::Branch => NewSessionField::Path,
                };
            }
        }
        KeyCode::Enter => {
            app.confirm_new_session(true); // Start claude by default
        }
        // Path completion navigation (only when path field is active)
        KeyCode::Up if current_field == NewSessionField::Path => {
            app.select_prev_new_session_path();
        }
        KeyCode::Down if current_field == NewSessionField::Path => {
            app.select_next_new_session_path();
        }
        // Accept completion with Right arrow (only when path field is active)
        KeyCode::Right if current_field == NewSessionField::Path => {
            app.accept_new_session_path_completion();
        }
        // Branch navigation (only when branch field is active in worktree mode)
        KeyCode::Down if current_field == NewSessionField::Branch && worktree_enabled => {
            let filtered_count = app.filtered_new_session_branches().len();
            if filtered_count > 0 {
                if let Mode::NewSession {
                    ref mut selected_branch,
                    ..
                } = app.mode
                {
                    *selected_branch =
                        Some(selected_branch.map(|i| (i + 1) % filtered_count).unwrap_or(0));
                }
            }
        }
        KeyCode::Up if current_field == NewSessionField::Branch && worktree_enabled => {
            let filtered_count = app.filtered_new_session_branches().len();
            if filtered_count > 0 {
                if let Mode::NewSession {
                    ref mut selected_branch,
                    ..
                } = app.mode
                {
                    *selected_branch = Some(
                        selected_branch
                            .map(|i| if i == 0 { filtered_count - 1 } else { i - 1 })
                            .unwrap_or(filtered_count - 1),
                    );
                }
            }
        }
        // Accept branch completion with Right arrow
        KeyCode::Right if current_field == NewSessionField::Branch && worktree_enabled => {
            // Collect to owned strings to avoid borrow conflict
            let filtered: Vec<String> = app
                .filtered_new_session_branches()
                .into_iter()
                .map(|s| s.to_string())
                .collect();
            if let Mode::NewSession {
                ref mut branch_input,
                ref mut selected_branch,
                ..
            } = app.mode
            {
                if let Some(idx) = *selected_branch {
                    if let Some(branch) = filtered.get(idx) {
                        *branch_input = branch.clone();
                        *selected_branch = None;
                    }
                } else if let Some(first) = filtered.first() {
                    *branch_input = first.clone();
                }
            }
        }
        KeyCode::Backspace => {
            if let Mode::NewSession {
                ref mut name,
                ref mut path,
                ref field,
                ref mut path_selected,
                ref mut branch_input,
                ref mut selected_branch,
                ..
            } = app.mode
            {
                match field {
                    NewSessionField::Name => {
                        name.pop();
                    }
                    NewSessionField::Path => {
                        path.pop();
                        *path_selected = None; // Reset selection on edit
                    }
                    NewSessionField::Branch => {
                        branch_input.pop();
                        *selected_branch = None;
                    }
                }
            }
            if current_field == NewSessionField::Path {
                app.update_new_session_path_suggestions();
                app.update_new_session_branches();
            }
        }
        KeyCode::Char(c) => {
            if let Mode::NewSession {
                ref mut name,
                ref mut path,
                ref field,
                ref mut path_selected,
                ref mut branch_input,
                ref mut selected_branch,
                ..
            } = app.mode
            {
                match field {
                    NewSessionField::Name => {
                        // Only allow valid session name characters
                        if c.is_alphanumeric() || c == '-' || c == '_' {
                            name.push(c);
                        }
                    }
                    NewSessionField::Path => {
                        path.push(c);
                        *path_selected = None; // Reset selection on edit
                    }
                    NewSessionField::Branch => {
                        branch_input.push(c);
                        *selected_branch = None;
                    }
                }
            }
            if current_field == NewSessionField::Path {
                app.update_new_session_path_suggestions();
                app.update_new_session_branches();
            }
        }
        _ => {}
    }
}

fn handle_rename_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.cancel();
        }
        KeyCode::Enter => {
            app.confirm_rename();
        }
        KeyCode::Backspace => {
            if let Mode::Rename { ref mut new_name, .. } = app.mode {
                new_name.pop();
            }
        }
        KeyCode::Char(c) => {
            if let Mode::Rename { ref mut new_name, .. } = app.mode {
                // Only allow valid session name characters
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    new_name.push(c);
                }
            }
        }
        _ => {}
    }
}

fn handle_commit_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.cancel();
        }
        KeyCode::Enter => {
            app.confirm_commit();
        }
        KeyCode::Backspace => {
            if let Mode::Commit { ref mut message } = app.mode {
                message.pop();
            }
        }
        KeyCode::Char(c) => {
            if let Mode::Commit { ref mut message } = app.mode {
                message.push(c);
            }
        }
        _ => {}
    }
}

fn handle_new_worktree_mode(app: &mut App, key: KeyEvent) {
    // Get current field to determine behavior
    let current_field = if let Mode::NewWorktree { field, .. } = &app.mode {
        *field
    } else {
        return;
    };

    match key.code {
        KeyCode::Esc => {
            app.cancel();
        }
        KeyCode::Tab => {
            // Cycle through fields
            if let Mode::NewWorktree { ref mut field, .. } = app.mode {
                *field = match field {
                    NewWorktreeField::Branch => NewWorktreeField::Path,
                    NewWorktreeField::Path => NewWorktreeField::SessionName,
                    NewWorktreeField::SessionName => NewWorktreeField::Branch,
                };
            }
        }
        KeyCode::BackTab => {
            // Cycle backwards through fields
            if let Mode::NewWorktree { ref mut field, .. } = app.mode {
                *field = match field {
                    NewWorktreeField::Branch => NewWorktreeField::SessionName,
                    NewWorktreeField::Path => NewWorktreeField::Branch,
                    NewWorktreeField::SessionName => NewWorktreeField::Path,
                };
            }
        }
        KeyCode::Enter => {
            app.confirm_new_worktree();
        }
        KeyCode::Backspace => {
            if let Mode::NewWorktree {
                ref mut branch_input,
                ref mut worktree_path,
                ref mut session_name,
                ref mut path_selected,
                field,
                ..
            } = app.mode
            {
                match field {
                    NewWorktreeField::Branch => {
                        branch_input.pop();
                    }
                    NewWorktreeField::Path => {
                        worktree_path.pop();
                        *path_selected = None; // Reset selection on edit
                    }
                    NewWorktreeField::SessionName => {
                        session_name.pop();
                    }
                }
            }
            // Update suggestions after input changes
            if current_field == NewWorktreeField::Branch {
                app.update_worktree_suggestions();
            } else if current_field == NewWorktreeField::Path {
                app.update_worktree_path_suggestions();
            }
        }
        KeyCode::Char(c) => {
            if let Mode::NewWorktree {
                ref mut branch_input,
                ref mut worktree_path,
                ref mut session_name,
                ref mut path_selected,
                field,
                ..
            } = app.mode
            {
                match field {
                    NewWorktreeField::Branch => {
                        branch_input.push(c);
                    }
                    NewWorktreeField::Path => {
                        worktree_path.push(c);
                        *path_selected = None; // Reset selection on edit
                    }
                    NewWorktreeField::SessionName => {
                        // Only allow valid session name characters
                        if c.is_alphanumeric() || c == '-' || c == '_' {
                            session_name.push(c);
                        }
                    }
                }
            }
            // Update suggestions after input changes
            if current_field == NewWorktreeField::Branch {
                app.update_worktree_suggestions();
            } else if current_field == NewWorktreeField::Path {
                app.update_worktree_path_suggestions();
            }
        }
        // Navigate branch suggestions when in Branch field
        KeyCode::Down if current_field == NewWorktreeField::Branch => {
            let filtered_count = app.filtered_branches().len();
            if filtered_count > 0 {
                if let Mode::NewWorktree {
                    ref mut selected_branch,
                    ..
                } = app.mode
                {
                    *selected_branch =
                        Some(selected_branch.map(|i| (i + 1) % filtered_count).unwrap_or(0));
                }
                app.update_worktree_suggestions();
            }
        }
        KeyCode::Up if current_field == NewWorktreeField::Branch => {
            let filtered_count = app.filtered_branches().len();
            if filtered_count > 0 {
                if let Mode::NewWorktree {
                    ref mut selected_branch,
                    ..
                } = app.mode
                {
                    *selected_branch = Some(
                        selected_branch
                            .map(|i| if i == 0 { filtered_count - 1 } else { i - 1 })
                            .unwrap_or(filtered_count - 1),
                    );
                }
                app.update_worktree_suggestions();
            }
        }
        // Accept branch completion with Right arrow
        KeyCode::Right if current_field == NewWorktreeField::Branch => {
            app.accept_branch_completion();
        }
        // Navigate path suggestions when in Path field
        KeyCode::Down if current_field == NewWorktreeField::Path => {
            app.select_next_worktree_path();
        }
        KeyCode::Up if current_field == NewWorktreeField::Path => {
            app.select_prev_worktree_path();
        }
        // Accept path completion with Right arrow
        KeyCode::Right if current_field == NewWorktreeField::Path => {
            app.accept_worktree_path_completion();
        }
        _ => {}
    }
}

fn handle_create_pr_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.cancel();
        }
        KeyCode::Tab => {
            // Cycle through fields
            if let Mode::CreatePullRequest { ref mut field, .. } = app.mode {
                *field = match field {
                    CreatePullRequestField::Title => CreatePullRequestField::Body,
                    CreatePullRequestField::Body => CreatePullRequestField::BaseBranch,
                    CreatePullRequestField::BaseBranch => CreatePullRequestField::Title,
                };
            }
        }
        KeyCode::BackTab => {
            // Cycle backwards through fields
            if let Mode::CreatePullRequest { ref mut field, .. } = app.mode {
                *field = match field {
                    CreatePullRequestField::Title => CreatePullRequestField::BaseBranch,
                    CreatePullRequestField::Body => CreatePullRequestField::Title,
                    CreatePullRequestField::BaseBranch => CreatePullRequestField::Body,
                };
            }
        }
        KeyCode::Enter => {
            app.confirm_create_pull_request();
        }
        KeyCode::Backspace => {
            if let Mode::CreatePullRequest {
                ref mut title,
                ref mut body,
                ref mut base_branch,
                field,
            } = app.mode
            {
                match field {
                    CreatePullRequestField::Title => {
                        title.pop();
                    }
                    CreatePullRequestField::Body => {
                        body.pop();
                    }
                    CreatePullRequestField::BaseBranch => {
                        base_branch.pop();
                    }
                }
            }
        }
        KeyCode::Char(c) => {
            if let Mode::CreatePullRequest {
                ref mut title,
                ref mut body,
                ref mut base_branch,
                field,
            } = app.mode
            {
                match field {
                    CreatePullRequestField::Title => {
                        title.push(c);
                    }
                    CreatePullRequestField::Body => {
                        body.push(c);
                    }
                    CreatePullRequestField::BaseBranch => {
                        // Branch names have specific allowed characters
                        if c.is_alphanumeric() || c == '-' || c == '_' || c == '/' {
                            base_branch.push(c);
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

fn handle_help_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('?') => {
            app.cancel();
        }
        _ => {}
    }
}
