use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::provider::PipelineStatus;
use crate::state::{AppState, ConfirmAction, Panel};

pub fn draw(frame: &mut Frame, state: &AppState) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(frame.area());

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(35),
            Constraint::Percentage(40),
        ])
        .split(rows[0]);

    draw_repos(frame, state, chunks[0]);
    draw_pipelines(frame, state, chunks[1]);
    draw_logs(frame, state, chunks[2]);
    draw_statusbar(frame, state, rows[1]);

    if state.show_help {
        draw_help_popup(frame);
    }

    if let Some(action) = &state.confirm {
        draw_confirm_popup(frame, action);
    }
}

fn border_style(active: bool) -> Style {
    if active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    }
}

fn status_color(status: &PipelineStatus) -> Color {
    match status {
        PipelineStatus::Success => Color::Green,
        PipelineStatus::Failed => Color::Red,
        PipelineStatus::Running => Color::Cyan,
        PipelineStatus::Pending => Color::Yellow,
        PipelineStatus::Cancelled => Color::DarkGray,
        PipelineStatus::Unknown => Color::White,
    }
}

fn status_icon(status: &PipelineStatus) -> &'static str {
    match status {
        PipelineStatus::Success => "✓",
        PipelineStatus::Failed => "✗",
        PipelineStatus::Running => "⟳",
        PipelineStatus::Pending => "…",
        PipelineStatus::Cancelled => "⊘",
        PipelineStatus::Unknown => "?",
    }
}

fn draw_repos(frame: &mut Frame, state: &AppState, area: Rect) {
    let active = state.active_panel == Panel::Repos;
    let block = Block::default()
        .title(" Repos ")
        .borders(Borders::ALL)
        .border_style(border_style(active));

    let items: Vec<ListItem> = state.repos.iter().map(|repo| {
        let pipelines = state.pipelines.get(&repo.name);
        let status = pipelines
            .and_then(|p| p.first())
            .map(|p| &p.status)
            .unwrap_or(&PipelineStatus::Unknown);

        let icon = status_icon(status);
        let color = status_color(status);
        let provider_icon = match repo.provider.as_str() {
            "gitlab" => "GL",
            _ => "GH",
        };

        ListItem::new(Line::from(vec![
            Span::styled(format!("{} ", icon), Style::default().fg(color)),
            Span::styled(format!("[{}] ", provider_icon), Style::default().fg(Color::DarkGray)),
            Span::raw(&repo.name),
        ]))
    }).collect();

    let mut list_state = ListState::default();
    if active {
        list_state.select(Some(state.selected_repo));
    }

    frame.render_stateful_widget(
        List::new(items)
            .block(block)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED)),
        area,
        &mut list_state,
    );
}

fn draw_pipelines(frame: &mut Frame, state: &AppState, area: Rect) {
    let active = state.active_panel == Panel::Pipelines;
    let title = match state.active_panel {
        Panel::Pipelines => " Pipelines ",
        _ => " Pipelines ",
    };
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style(active));

    let pipelines = state.current_pipelines();
    let items: Vec<ListItem> = pipelines.iter().map(|p| {
        let icon = status_icon(&p.status);
        let color = status_color(&p.status);
        let date = p.created_at.get(..10).unwrap_or(&p.created_at);

        ListItem::new(Line::from(vec![
            Span::styled(format!("{} ", icon), Style::default().fg(color)),
            Span::raw(format!("{} ", date)),
            Span::styled(&p.name, Style::default().fg(Color::White)),
        ]))
    }).collect();

    let mut list_state = ListState::default();
    if active {
        list_state.select(Some(state.selected_pipeline));
    }

    frame.render_stateful_widget(
        List::new(items)
            .block(block)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED)),
        area,
        &mut list_state,
    );
}

fn draw_logs(frame: &mut Frame, state: &AppState, area: Rect) {
    let active = state.active_panel == Panel::Logs;
    let block = Block::default()
        .title(" Logs ")
        .borders(Borders::ALL)
        .border_style(border_style(active));

    let content = state.current_logs().unwrap_or("No logs available.");

    frame.render_widget(
        Paragraph::new(content)
            .block(block)
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn draw_statusbar(frame: &mut Frame, state: &AppState, area: Rect) {
    let updated = state.last_updated
        .map(|t| {
            let secs = t.elapsed().as_secs();
            if secs < 60 {
                format!("updated {}s ago", secs)
            } else {
                format!("updated {}m ago", secs / 60)
            }
        })
        .unwrap_or_else(|| "loading…".to_string());

    let hints = " Tab:panel  j/k:nav  R:rerun  q:quit";

    let line = if let Some(err) = &state.error {
        Line::from(vec![
            Span::styled(format!(" ✗ {} ", err), Style::default().fg(Color::Red)),
            Span::styled(hints, Style::default().fg(Color::DarkGray)),
        ])
    } else {
        Line::from(vec![
            Span::styled(format!(" {} ", updated), Style::default().fg(Color::DarkGray)),
            Span::styled(hints, Style::default().fg(Color::DarkGray)),
        ])
    };

    frame.render_widget(Paragraph::new(line), area);
}

fn draw_help_popup(frame: &mut Frame) {
    let area = frame.area();
    let popup = Rect {
        x: area.width / 2 - 20,
        y: area.height / 2 - 6,
        width: 40,
        height: 12,
    };

    let lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled("  Tab      ", Style::default().fg(Color::Yellow)), Span::raw("switch panel")]),
        Line::from(vec![Span::styled("  j / ↓   ", Style::default().fg(Color::Yellow)), Span::raw("move down")]),
        Line::from(vec![Span::styled("  k / ↑   ", Style::default().fg(Color::Yellow)), Span::raw("move up")]),
        Line::from(vec![Span::styled("  R        ", Style::default().fg(Color::Yellow)), Span::raw("rerun pipeline")]),
        Line::from(vec![Span::styled("  ?        ", Style::default().fg(Color::Yellow)), Span::raw("toggle this help")]),
        Line::from(vec![Span::styled("  q / Esc  ", Style::default().fg(Color::Yellow)), Span::raw("quit")]),
        Line::from(""),
        Line::from(Span::styled("  Press ? to close", Style::default().fg(Color::DarkGray))),
    ];

    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .title(" Keybindings ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        ),
        popup,
    );
}

fn draw_confirm_popup(frame: &mut Frame, action: &ConfirmAction) {
    let area = frame.area();
    let popup = Rect {
        x: area.width / 2 - 18,
        y: area.height / 2 - 3,
        width: 36,
        height: 6,
    };

    let (title, msg, color) = match action {
        ConfirmAction::Rerun => (" Confirm Rerun ", "Rerun this pipeline?", Color::Cyan),
        ConfirmAction::Cancel => (" Confirm Cancel ", "Cancel this pipeline?", Color::Red),
    };

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(format!("  {}", msg), Style::default().fg(color))),
        Line::from(""),
        Line::from(vec![
            Span::styled("  y", Style::default().fg(Color::Green)),
            Span::raw(" confirm   "),
            Span::styled("n/Esc", Style::default().fg(Color::DarkGray)),
            Span::raw(" cancel"),
        ]),
    ];

    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(color)),
        ),
        popup,
    );
}
