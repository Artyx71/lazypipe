use std::collections::HashMap;
use std::time::Instant;

use crate::config::RepoConfig;
use crate::provider::{Job, Pipeline};

#[derive(Clone, Debug, PartialEq)]
pub enum Panel {
    Repos,
    Pipelines,
    Logs,
}

pub struct AppState {
    pub repos: Vec<RepoConfig>,
    pub show_help: bool,
    pub pipelines: HashMap<String, Vec<Pipeline>>,  // key: repo name
    pub jobs: HashMap<String, Vec<Job>>,             // key: pipeline id
    pub logs: HashMap<String, String>,               // key: job id
    pub selected_repo: usize,
    pub selected_pipeline: usize,
    pub selected_job: usize,
    pub active_panel: Panel,
    pub error: Option<String>,
    pub last_updated: Option<Instant>,
}

impl AppState {
    pub fn new(repos: Vec<RepoConfig>) -> Self {
        Self {
            repos,
            pipelines: HashMap::new(),
            jobs: HashMap::new(),
            logs: HashMap::new(),
            selected_repo: 0,
            selected_pipeline: 0,
            selected_job: 0,
            active_panel: Panel::Repos,
            error: None,
            last_updated: None,
            show_help: false,
        }
    }

    pub fn current_repo(&self) -> Option<&RepoConfig> {
        self.repos.get(self.selected_repo)
    }

    pub fn current_pipelines(&self) -> &[Pipeline] {
        self.current_repo()
            .and_then(|r| self.pipelines.get(&r.name))
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn current_pipeline(&self) -> Option<&Pipeline> {
        self.current_pipelines().get(self.selected_pipeline)
    }

    pub fn current_jobs(&self) -> &[Job] {
        self.current_pipeline()
            .and_then(|p| self.jobs.get(&p.id))
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn current_logs(&self) -> Option<&str> {
        self.current_jobs()
            .get(self.selected_job)
            .and_then(|j| self.logs.get(&j.id))
            .map(|s| s.as_str())
    }
}
