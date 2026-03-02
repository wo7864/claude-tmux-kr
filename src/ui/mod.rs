//! UI rendering for the TUI application
//!
//! This module provides all rendering functionality:
//! - Main layout and components (header, session list, preview, status, footer)
//! - Modal dialogs for user input
//! - Help screen and message overlays

mod dialogs;
mod help;

use ansi_to_tui::IntoText;
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, StatefulWidget},
    Frame,
};
use unicode_width::UnicodeWidthStr;

use crate::app::{App, Mode};
use crate::session::ClaudeCodeStatus;

/// Render the application UI
pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    // Main layout: header, center (session list + preview), status bar, footer
    let outer = Layout::vertical([
        Constraint::Length(1), // Header
        Constraint::Min(3),   // Center area
        Constraint::Length(1), // Status bar
        Constraint::Length(1), // Footer
    ])
    .split(area);

    // Center area: session list (30%) + preview (70%) side by side
    let center = Layout::horizontal([
        Constraint::Percentage(30), // Session list
        Constraint::Percentage(70), // Preview pane
    ])
    .split(outer[1]);

    render_header(frame, app, outer[0]);
    render_session_list(frame, app, center[0]);
    app.preview_height = center[1].height;
    render_preview(frame, app, center[1]);
    render_status_bar(frame, app, outer[2]);
    render_footer(frame, app, outer[3]);

    // Render modal overlays
    match &app.mode {
        Mode::ConfirmAction => {
            dialogs::render_confirm_action(frame, app);
        }
        Mode::NewSession {
            name,
            path,
            field,
            path_suggestions,
            path_selected,
            worktree_enabled,
            branch_input,
            selected_branch,
            ..
        } => {
            let filtered_branches = if *worktree_enabled {
                app.filtered_new_session_branches()
            } else {
                vec![]
            };
            dialogs::render_new_session_dialog(
                frame,
                name,
                path,
                *field,
                path_suggestions,
                *path_selected,
                *worktree_enabled,
                branch_input,
                &filtered_branches,
                *selected_branch,
            );
        }
        Mode::Rename { old_name, new_name } => {
            dialogs::render_rename_dialog(frame, old_name, new_name);
        }
        Mode::Commit { message } => {
            dialogs::render_commit_dialog(frame, message);
        }
        Mode::NewWorktree {
            branch_input,
            selected_branch,
            worktree_path,
            session_name,
            field,
            path_suggestions,
            path_selected,
            ..
        } => {
            dialogs::render_new_worktree_dialog(
                frame,
                app,
                branch_input,
                *selected_branch,
                worktree_path,
                session_name,
                *field,
                path_suggestions,
                *path_selected,
            );
        }
        Mode::Filter { input } => {
            render_filter_bar(frame, input, outer[2]);
        }
        Mode::Search { input } => {
            render_search_bar(frame, input, outer[2]);
        }
        Mode::CreatePullRequest {
            title,
            body,
            base_branch,
            field,
        } => {
            dialogs::render_create_pr_dialog(frame, title, body, base_branch, *field);
        }
        Mode::Help => {
            help::render_help(frame);
        }
        Mode::Normal | Mode::ActionMenu => {}
    }

    // Render error/message overlay
    if let Some(ref error) = app.error {
        help::render_message(frame, error, Color::Red);
    } else if let Some(ref message) = app.message {
        help::render_message(frame, message, Color::Green);
    }
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let current = app
        .current_session
        .as_ref()
        .map(|s| format!(" 연결됨: {} ", s))
        .unwrap_or_default();

    let title = format!(
        "─ claude-tmux ─{:─>width$}",
        current,
        width = area.width as usize - 15
    );

    let header = Paragraph::new(title)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

    frame.render_widget(header, area);
}

fn render_session_list(frame: &mut Frame, app: &mut App, area: Rect) {
    if app.grouped_view.enabled && !matches!(app.mode, Mode::ActionMenu) {
        render_grouped_session_list(frame, app, area);
    } else {
        render_flat_session_list(frame, app, area);
    }
}

fn render_grouped_session_list(frame: &mut Frame, app: &mut App, area: Rect) {
    let selected_index = app.compute_flat_list_index();
    let total_items = app.compute_total_list_items();
    let visible_height = (area.height as usize).max(1);

    let mut scroll_state = std::mem::take(&mut app.scroll_state);
    let filtered = app.filtered_sessions();

    if filtered.is_empty() {
        let empty_msg = if app.filter.is_empty() {
            "tmux 세션을 찾을 수 없습니다. 'n'을 눌러 새로 만드세요."
        } else {
            "필터와 일치하는 세션이 없습니다."
        };
        let paragraph = Paragraph::new(empty_msg)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
        app.scroll_state = scroll_state;
        return;
    }

    let mut items: Vec<ListItem> = Vec::new();
    let mut visual_pos = 0;

    for group in app.grouped_view.groups.iter() {
        let is_header_selected = visual_pos == app.grouped_selected;

        // Group header: ▼/▶ + path + (count)
        let arrow = if group.collapsed { "▶" } else { "▼" };
        let header_line = Line::from(vec![
            Span::styled(
                format!("{} ", arrow),
                Style::default().fg(Color::Cyan),
            ),
            Span::styled(
                &group.display_name,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" ({})", group.session_indices.len()),
                Style::default().fg(Color::DarkGray),
            ),
        ]);

        let header_style = if is_header_selected {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default()
        };
        items.push(ListItem::new(header_line).style(header_style));
        visual_pos += 1;

        // Sessions within the group (if expanded)
        if !group.collapsed {
            for &session_idx in &group.session_indices {
                let is_session_selected = visual_pos == app.grouped_selected;

                if let Some(session) = filtered.get(session_idx) {
                    let is_current = app
                        .current_session
                        .as_ref()
                        .is_some_and(|c| c == &session.name);

                    let status = &session.claude_code_status;
                    let status_color = match (status, is_session_selected) {
                        (ClaudeCodeStatus::Working, _) => Color::Green,
                        (ClaudeCodeStatus::WaitingInput, _) => Color::Yellow,
                        (ClaudeCodeStatus::Idle, true) => Color::White,
                        (ClaudeCodeStatus::Idle, false) => Color::DarkGray,
                        (ClaudeCodeStatus::Unknown, true) => Color::Gray,
                        (ClaudeCodeStatus::Unknown, false) => Color::DarkGray,
                    };

                    let marker = if is_session_selected { "▸" } else { " " };
                    let name_style = if is_current {
                        Style::default().add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };

                    // Build git info spans (branch + status only, path is in header)
                    let git_spans = build_git_info_spans(session);

                    let mut spans = vec![
                        Span::raw(format!("   {} ", marker)),
                        Span::styled(&session.name, name_style),
                        Span::raw("  "),
                        Span::styled(status.symbol(), Style::default().fg(status_color)),
                        Span::raw(" "),
                        Span::styled(
                            format!("{:<8}", status.label()),
                            Style::default().fg(status_color),
                        ),
                    ];
                    spans.extend(git_spans);

                    let style = if is_session_selected {
                        Style::default().bg(Color::DarkGray)
                    } else {
                        Style::default()
                    };
                    items.push(ListItem::new(Line::from(spans)).style(style));
                }
                visual_pos += 1;
            }
        }
    }

    {
        let list = List::new(items);
        let list_state = scroll_state.update(selected_index, total_items, visible_height);
        StatefulWidget::render(list, area, frame.buffer_mut(), list_state);
    }

    app.scroll_state = scroll_state;
}

/// Build git info spans (branch + status indicators) for a session
fn build_git_info_spans<'a>(session: &'a crate::session::Session) -> Vec<Span<'a>> {
    let Some(ref git) = session.git_context else {
        return vec![];
    };

    let (open, close) = if git.is_worktree {
        ("[", "]")
    } else {
        ("(", ")")
    };
    let bracket_color = if git.is_worktree {
        Color::Magenta
    } else {
        Color::Cyan
    };

    let mut status_str = String::new();
    if git.has_staged {
        status_str.push('+');
    }
    if git.has_unstaged {
        status_str.push('*');
    }
    let status_spans = if !status_str.is_empty() {
        let color = if git.has_staged && !git.has_unstaged {
            Color::Green
        } else {
            Color::Yellow
        };
        vec![Span::styled(
            format!(" {}", status_str),
            Style::default().fg(color),
        )]
    } else {
        vec![]
    };

    let mut spans = vec![
        Span::raw(" "),
        Span::styled(open, Style::default().fg(bracket_color)),
        Span::styled(&git.branch, Style::default().fg(Color::Cyan)),
        Span::styled(close, Style::default().fg(bracket_color)),
    ];
    spans.extend(status_spans);
    spans
}

fn render_flat_session_list(frame: &mut Frame, app: &mut App, area: Rect) {
    // Compute scroll state values before borrowing for items
    let selected_index = app.compute_flat_list_index();
    let total_items = app.compute_total_list_items();
    // Each session ListItem renders as 2 rows, so approximate visible item count
    // by halving the pixel height. This keeps centering and max_offset correct.
    let visible_height = (area.height as usize).max(2) / 2;

    // Take scroll_state out of app to avoid borrow conflicts
    // (items building borrows app immutably, scroll_state needs mutable access)
    let mut scroll_state = std::mem::take(&mut app.scroll_state);

    let filtered = app.filtered_sessions();

    if filtered.is_empty() {
        let empty_msg = if app.filter.is_empty() {
            "tmux 세션을 찾을 수 없습니다. 'n'을 눌러 새로 만드세요."
        } else {
            "필터와 일치하는 세션이 없습니다."
        };
        let paragraph = Paragraph::new(empty_msg)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
        // Put scroll_state back before returning
        app.scroll_state = scroll_state;
        return;
    }

    // Calculate column widths
    let max_name_len = filtered
        .iter()
        .map(|s| s.name.width())
        .max()
        .unwrap_or(10)
        .max(10);

    let mut items: Vec<ListItem> = Vec::new();

    for (i, session) in filtered.iter().enumerate() {
        let is_selected = i == app.selected;
        let is_current = app
            .current_session
            .as_ref()
            .is_some_and(|c| c == &session.name);

        // Show ▾ when action menu is open for this session, ▸ when selected but collapsed
        let is_expanded = is_selected && matches!(app.mode, Mode::ActionMenu);
        let marker = if is_selected {
            if is_expanded {
                "▾"
            } else {
                "▸"
            }
        } else {
            " "
        };
        let status = &session.claude_code_status;

        // Use brighter colors when selected so text is readable on dark background
        let status_color = match (status, is_selected) {
            (ClaudeCodeStatus::Working, _) => Color::Green,
            (ClaudeCodeStatus::WaitingInput, _) => Color::Yellow,
            (ClaudeCodeStatus::Idle, true) => Color::White,
            (ClaudeCodeStatus::Idle, false) => Color::DarkGray,
            (ClaudeCodeStatus::Unknown, true) => Color::Gray,
            (ClaudeCodeStatus::Unknown, false) => Color::DarkGray,
        };

        let path_color = if is_selected {
            Color::White
        } else {
            Color::DarkGray
        };

        let name_style = if is_current {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        // Build git info spans
        let git_spans = build_git_info_spans(session);

        // Line 1: marker + session name + status symbol/label
        let line1 = Line::from(vec![
            Span::raw(format!(" {} ", marker)),
            Span::styled(
                format!("{:<width$}", session.name, width = max_name_len),
                name_style,
            ),
            Span::raw("  "),
            Span::styled(status.symbol(), Style::default().fg(status_color)),
            Span::raw(" "),
            Span::styled(
                format!("{:<8}", status.label()),
                Style::default().fg(status_color),
            ),
        ]);

        // Line 2: indented path + git info (branch + status)
        let mut line2_spans = vec![
            Span::raw("     "),
            Span::styled(session.display_path(), Style::default().fg(path_color)),
        ];
        line2_spans.extend(git_spans);
        let line2 = Line::from(line2_spans);

        let style = if is_selected {
            Style::default().bg(Color::DarkGray)
        } else {
            Style::default()
        };

        items.push(ListItem::new(vec![line1, line2]).style(style));

        // Show expanded content when in action menu mode for this session
        if is_expanded {
            render_expanded_session_content(app, session, &mut items);
        }
    }

    // Scope the list rendering so borrows are released before we restore scroll_state
    {
        let list = List::new(items);

        // Update scroll state with centered scrolling behavior
        let list_state = scroll_state.update(selected_index, total_items, visible_height);

        // Render with stateful widget for proper scrolling
        StatefulWidget::render(list, area, frame.buffer_mut(), list_state);
    }

    // Put scroll_state back into app (list borrows are now released)
    app.scroll_state = scroll_state;
}

/// Render the expanded content for a session in action menu mode
fn render_expanded_session_content<'a>(
    app: &'a App,
    session: &'a crate::session::Session,
    items: &mut Vec<ListItem<'a>>,
) {
    let label_style = Style::default().fg(Color::DarkGray);
    let value_style = Style::default().fg(Color::White);

    // Session metadata row
    let attached_str = if session.attached { "예" } else { "아니오" };
    let pane_count = session.panes.len();

    let meta_line = Line::from(vec![
        Span::raw("     "),
        Span::styled("윈도우: ", label_style),
        Span::styled(format!("{}", session.window_count), value_style),
        Span::raw("  "),
        Span::styled("패널: ", label_style),
        Span::styled(format!("{}", pane_count), value_style),
        Span::raw("  "),
        Span::styled("가동시간: ", label_style),
        Span::styled(session.duration(), value_style),
        Span::raw("  "),
        Span::styled("연결됨: ", label_style),
        Span::styled(attached_str, value_style),
    ]);
    items.push(ListItem::new(meta_line));

    // Git metadata row (if available)
    if let Some(ref git) = session.git_context {
        let mut git_spans = vec![
            Span::raw("     "),
            Span::styled("브랜치: ", label_style),
            Span::styled(&git.branch, Style::default().fg(Color::Cyan)),
        ];

        if git.ahead > 0 || git.behind > 0 {
            git_spans.push(Span::raw("  "));
            if git.ahead > 0 {
                git_spans.push(Span::styled(
                    format!("↑{}", git.ahead),
                    Style::default().fg(Color::Green),
                ));
            }
            if git.behind > 0 {
                if git.ahead > 0 {
                    git_spans.push(Span::raw(" "));
                }
                git_spans.push(Span::styled(
                    format!("↓{}", git.behind),
                    Style::default().fg(Color::Red),
                ));
            }
        }

        // Show staged/unstaged status
        if git.has_staged {
            git_spans.push(Span::raw("  "));
            git_spans.push(Span::styled("스테이지: ", label_style));
            git_spans.push(Span::styled("예", Style::default().fg(Color::Green)));
        }

        if git.has_unstaged {
            git_spans.push(Span::raw("  "));
            git_spans.push(Span::styled("미스테이지: ", label_style));
            git_spans.push(Span::styled("예", Style::default().fg(Color::Yellow)));
        }

        if git.is_worktree {
            git_spans.push(Span::raw("  "));
            git_spans.push(Span::styled("워크트리: ", label_style));
            git_spans.push(Span::styled("예", Style::default().fg(Color::Magenta)));
        }

        items.push(ListItem::new(Line::from(git_spans)));

        // PR status row (if available)
        if let Some(ref pr_info) = app.pr_info {
            let mut pr_spans = vec![
                Span::raw("     "),
                Span::styled("PR #", label_style),
                Span::styled(
                    format!("{}", pr_info.number),
                    Style::default().fg(Color::Cyan),
                ),
                Span::raw(": "),
            ];

            // State with color
            let (state_text, state_color) = match pr_info.state.as_str() {
                "OPEN" => ("열림", Color::Green),
                "CLOSED" => ("닫힘", Color::Red),
                "MERGED" => ("병합됨", Color::Magenta),
                _ => (pr_info.state.as_str(), Color::Gray),
            };
            pr_spans.push(Span::styled(state_text, Style::default().fg(state_color)));

            // Mergeable status (only show for open PRs)
            if pr_info.state == "OPEN" {
                pr_spans.push(Span::raw("  "));
                let (merge_text, merge_color) = match pr_info.mergeable.as_str() {
                    "MERGEABLE" => ("병합 가능", Color::Green),
                    "CONFLICTING" => ("충돌 있음", Color::Red),
                    _ => ("병합 상태 알수없음", Color::Yellow),
                };
                pr_spans.push(Span::styled(merge_text, Style::default().fg(merge_color)));
            }

            items.push(ListItem::new(Line::from(pr_spans)));
        }
    }

    // Separator
    let sep_line = Line::from(Span::styled(
        "     ────────────────────────",
        Style::default().fg(Color::DarkGray),
    ));
    items.push(ListItem::new(sep_line));

    // Action items
    for (action_idx, action) in app.available_actions.iter().enumerate() {
        let is_action_selected = action_idx == app.selected_action;
        let action_marker = if is_action_selected { "▸" } else { " " };
        let action_style = if is_action_selected {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };

        let action_line = Line::from(vec![
            Span::raw("     "),
            Span::styled(format!("{} {}", action_marker, action.label()), action_style),
        ]);
        items.push(ListItem::new(action_line));
    }

    // White separator at end of submenu
    let end_sep = Line::from(Span::styled("", Style::default().fg(Color::White)));
    items.push(ListItem::new(end_sep));
}

fn render_preview(frame: &mut Frame, app: &App, area: Rect) {
    // Clear the entire preview area first to prevent stale content
    frame.render_widget(Clear, area);

    // Use a Block with left border to visually separate from session list
    let block = Block::default()
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" 미리보기 ")
        .title_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let content = match &app.preview_content {
        Some(text) if !text.is_empty() => text,
        _ => {
            let msg = Paragraph::new("  미리보기 없음")
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(msg, inner);
            return;
        }
    };

    // Parse ANSI escape sequences into styled ratatui Text
    let styled_text = match content.into_text() {
        Ok(text) => text,
        Err(_) => {
            // Fallback to plain text if parsing fails
            Text::raw(content)
        }
    };

    // Take only the last N lines that fit in the content area
    let available_lines = inner.height as usize;
    let total_lines = styled_text.lines.len();
    let start = total_lines.saturating_sub(available_lines);
    let visible_lines: Vec<Line> = styled_text.lines.into_iter().skip(start).collect();

    let preview = Paragraph::new(visible_lines);
    frame.render_widget(preview, inner);
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let (working, waiting, _idle) = app.status_counts();
    let total = app.sessions.len();

    let mut parts = vec![format!("{}개 세션", total)];

    if app.grouped_view.enabled {
        parts.push(format!("{}개 프로젝트", app.grouped_view.groups.len()));
    }

    if working > 0 {
        parts.push(format!("{}개 작업중", working));
    }
    if waiting > 0 {
        parts.push(format!("{}개 입력대기", waiting));
    }

    let status = parts.join(" │ ");

    let filter_info = if !app.filter.is_empty() {
        format!(" │ 필터: \"{}\"", app.filter)
    } else {
        String::new()
    };

    let text = format!("  {}{}", status, filter_info);

    let bar = Paragraph::new(text).style(Style::default().fg(Color::DarkGray));

    frame.render_widget(bar, area);
}

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let hints = match app.mode {
        Mode::Normal => {
            "  ? 도움말  jk 이동  l 액션  ⏎ 전환  g 그룹  n 새세션  K 종료  R 새로고침  / 필터  : 검색  q 나가기"
        }
        Mode::ActionMenu => "  jk 이동  ⏎/l 선택  h/esc 뒤로  q 나가기",
        Mode::Filter { .. } => "  ⏎ 적용  esc 취소",
        Mode::Search { .. } => "  jk 이동  ⏎ 확정  esc 취소",
        Mode::ConfirmAction => "  y/⏎ 확인  n/esc 취소",
        Mode::NewSession { .. } => {
            "  ⏎ 생성  ^W 워크트리  tab 전환  ↑↓ 선택  → 수락  esc 취소"
        }
        Mode::Rename { .. } => "  ⏎ 확인  esc 취소",
        Mode::Commit { .. } => "  ⏎ 커밋  esc 취소",
        Mode::NewWorktree { .. } => "  ⏎ 생성  tab 전환  ↑↓ 선택  → 수락  esc 취소",
        Mode::CreatePullRequest { .. } => "  ⏎ PR 생성  tab 전환  esc 취소",
        Mode::Help => "  q 닫기",
    };

    let footer = Paragraph::new(hints).style(Style::default().fg(Color::DarkGray));

    frame.render_widget(footer, area);
}

fn render_filter_bar(frame: &mut Frame, input: &str, area: Rect) {
    frame.render_widget(Clear, area);
    let text = format!("  / {}", input);
    let bar = Paragraph::new(text).style(Style::default().fg(Color::Yellow));
    frame.render_widget(bar, area);
}

fn render_search_bar(frame: &mut Frame, input: &str, area: Rect) {
    frame.render_widget(Clear, area);
    let text = Line::from(vec![
        Span::styled("  : ", Style::default().fg(Color::Cyan)),
        Span::styled(input, Style::default().fg(Color::Cyan)),
        Span::styled("▏", Style::default().fg(Color::Cyan)),
    ]);
    let bar = Paragraph::new(text);
    frame.render_widget(bar, area);
}
