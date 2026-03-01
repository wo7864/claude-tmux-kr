//! Help screen and message overlays

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub fn render_help(frame: &mut Frame) {
    let area = centered_rect(60, 21, frame.area());

    let block = Block::default()
        .title(" 도움말 ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let help_text = vec![
        Line::from(Span::styled(
            "탐색",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::raw("  j / ↓       아래로 이동"),
        Line::raw("  k / ↑       위로 이동"),
        Line::raw("  l / →       액션 메뉴 열기"),
        Line::raw("  Enter       세션으로 전환"),
        Line::raw(""),
        Line::from(Span::styled(
            "동작",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::raw("  n           새 세션"),
        Line::raw("  K           세션 종료"),
        Line::raw("  r           세션 이름 변경"),
        Line::raw("  /           세션 필터"),
        Line::raw("  R           목록 새로고침"),
        Line::raw(""),
        Line::from(Span::styled(
            "액션 메뉴",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::raw("  h / ←       뒤로 가기"),
        Line::raw("  Enter       액션 실행"),
        Line::raw(""),
        Line::from(Span::styled(
            "기타",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::raw("  ?           이 도움말 표시"),
        Line::raw("  q / Esc     종료"),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .wrap(Wrap { trim: true });

    frame.render_widget(Clear, area);
    frame.render_widget(paragraph, area);
}

pub fn render_message(frame: &mut Frame, message: &str, color: Color) {
    let area = frame.area();

    // Calculate height needed (at least 1, up to 3 for longer messages)
    let max_width = area.width.saturating_sub(6) as usize;
    let lines_needed = if max_width > 0 {
        (message.len() / max_width + 1).min(3)
    } else {
        1
    };
    let height = lines_needed as u16;

    let msg_area = Rect {
        x: 2,
        y: area.height.saturating_sub(2 + height),
        width: area.width.saturating_sub(4),
        height,
    };

    let text = format!(" {} ", message);
    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::White).bg(color))
        .wrap(Wrap { trim: true });

    frame.render_widget(Clear, msg_area);
    frame.render_widget(paragraph, msg_area);
}

/// Create a centered rectangle of the given size within the parent area
pub fn centered_rect(width: u16, height: u16, parent: Rect) -> Rect {
    let x = parent.x + (parent.width.saturating_sub(width)) / 2;
    let y = parent.y + (parent.height.saturating_sub(height)) / 2;

    Rect {
        x,
        y,
        width: width.min(parent.width),
        height: height.min(parent.height),
    }
}
