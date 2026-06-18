use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

use crate::provider::{Job, Pipeline, PipelineStatus, Provider};

pub struct GitHubClient {
    client: Client,
    token: String,
}

impl GitHubClient {
    pub fn new(token: String) -> Self {
        Self { client: Client::new(), token }
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.token)
    }
}

#[derive(Deserialize)]
struct RunsResponse {
    workflow_runs: Vec<GhRun>,
}

#[derive(Deserialize)]
struct GhRun {
    id: u64,
    name: Option<String>,
    status: Option<String>,
    conclusion: Option<String>,
    created_at: String,
    html_url: String,
}

#[derive(Deserialize)]
struct JobsResponse {
    jobs: Vec<GhJob>,
}

#[derive(Deserialize)]
struct GhJob {
    id: u64,
    name: String,
    status: String,
    conclusion: Option<String>,
}

fn map_status(status: Option<&str>, conclusion: Option<&str>) -> PipelineStatus {
    match status {
        Some("completed") => match conclusion {
            Some("success") => PipelineStatus::Success,
            Some("failure") => PipelineStatus::Failed,
            Some("cancelled") => PipelineStatus::Cancelled,
            _ => PipelineStatus::Unknown,
        },
        Some("in_progress") => PipelineStatus::Running,
        Some("queued") | Some("waiting") => PipelineStatus::Pending,
        _ => PipelineStatus::Unknown,
    }
}

#[async_trait]
impl Provider for GitHubClient {
    async fn list_pipelines(&self, owner: &str, repo: &str) -> Result<Vec<Pipeline>, String> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/actions/runs?per_page=20",
            owner, repo
        );
        let resp: RunsResponse = self.client
            .get(&url)
            .header("Authorization", self.auth_header())
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "lazypipe/0.1")
            .send().await.map_err(|e| e.to_string())?
            .json().await.map_err(|e| e.to_string())?;

        Ok(resp.workflow_runs.into_iter().map(|r| Pipeline {
            id: r.id.to_string(),
            name: r.name.unwrap_or_else(|| "unknown".to_string()),
            status: map_status(r.status.as_deref(), r.conclusion.as_deref()),
            created_at: r.created_at,
            url: r.html_url,
        }).collect())
    }

    async fn list_jobs(&self, owner: &str, repo: &str, pipeline_id: &str) -> Result<Vec<Job>, String> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/actions/runs/{}/jobs",
            owner, repo, pipeline_id
        );
        let resp: JobsResponse = self.client
            .get(&url)
            .header("Authorization", self.auth_header())
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "lazypipe/0.1")
            .send().await.map_err(|e| e.to_string())?
            .json().await.map_err(|e| e.to_string())?;

        Ok(resp.jobs.into_iter().map(|j| Job {
            id: j.id.to_string(),
            name: j.name,
            status: map_status(Some(&j.status), j.conclusion.as_deref()),
        }).collect())
    }

    async fn get_logs(&self, owner: &str, repo: &str, job_id: &str) -> Result<String, String> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/actions/jobs/{}/logs",
            owner, repo, job_id
        );
        // GitHub redirects to plain text log — reqwest follows redirect automatically
        let text = self.client
            .get(&url)
            .header("Authorization", self.auth_header())
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "lazypipe/0.1")
            .send().await.map_err(|e| e.to_string())?
            .text().await.map_err(|e| e.to_string())?;

        Ok(text)
    }

    async fn rerun_pipeline(&self, owner: &str, repo: &str, pipeline_id: &str) -> Result<(), String> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/actions/runs/{}/rerun",
            owner, repo, pipeline_id
        );
        let resp = self.client
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "lazypipe/0.1")
            .send().await.map_err(|e| e.to_string())?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(format!("Rerun failed: HTTP {}", resp.status()))
        }
    }
}
