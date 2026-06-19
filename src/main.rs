mod config;
mod github;
mod gitlab;
mod poller;
mod provider;
mod state;
mod ui;

use std::io::stdout;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use state::{AppState, Panel};

#[tokio::main]
async fn main() {
    let cfg = match config::load_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Config error: {}", e);
            std::process::exit(1);
        }
    };

    let state = Arc::new(Mutex::new(AppState::new(cfg.repos)));
    poller::start_polling(Arc::clone(&state)).await;

    enable_raw_mode().expect("enable raw mode");
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen).expect("enter alternate screen");

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).expect("create terminal");

    let result = run_app(&mut terminal, &state).await;

    disable_raw_mode().expect("disable raw mode");
    execute!(terminal.backend_mut(), LeaveAlternateScreen).expect("leave alternate screen");

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    state: &Arc<Mutex<AppState>>,
) -> Result<(), String> {
    loop {
        {
            let s = state.lock().unwrap();
            terminal.draw(|f| ui::draw(f, &s)).map_err(|e| e.to_string())?;
        }

        if event::poll(Duration::from_millis(100)).map_err(|e| e.to_string())? {
            if let Event::Key(key) = event::read().map_err(|e| e.to_string())? {
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    && key.code == KeyCode::Char('c')
                {
                    return Ok(());
                }

                let mut s = state.lock().unwrap();
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),

                    KeyCode::Tab => {
                        s.active_panel = match s.active_panel {
                            Panel::Repos => Panel::Pipelines,
                            Panel::Pipelines => Panel::Logs,
                            Panel::Logs => Panel::Repos,
                        };
                    }

                    KeyCode::Down | KeyCode::Char('j') => match s.active_panel {
                        Panel::Repos => {
                            if s.selected_repo + 1 < s.repos.len() {
                                s.selected_repo += 1;
                                s.selected_pipeline = 0;
                            }
                        }
                        Panel::Pipelines => {
                            let len = s.current_pipelines().len();
                            if s.selected_pipeline + 1 < len {
                                s.selected_pipeline += 1;
                            }
                        }
                        Panel::Logs => {}
                    },

                    KeyCode::Up | KeyCode::Char('k') => match s.active_panel {
                        Panel::Repos => {
                            if s.selected_repo > 0 {
                                s.selected_repo -= 1;
                                s.selected_pipeline = 0;
                            }
                        }
                        Panel::Pipelines => {
                            if s.selected_pipeline > 0 {
                                s.selected_pipeline -= 1;
                            }
                        }
                        Panel::Logs => {}
                    },

                    _ => {}
                }
            }
        }
    }
}
