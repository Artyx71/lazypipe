use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::provider::PipelineStatus;
use crate::state::{AppState, Panel};

pub fn draw(frame: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(35),
            Constraint::Percentage(40),
        ])
        .split(frame.area());

    draw_repos(frame, state, chunks[0]);
    draw_pipelines(frame, state, chunks[1]);
    draw_logs(frame, state, chunks[2]);
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
