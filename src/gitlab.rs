use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

use crate::provider::{Job, Pipeline, PipelineStatus, Provider};

pub struct GitLabClient {
    client: Client,
    token: String,
    base_url: String,
}

impl GitLabClient {
    pub fn new(token: String) -> Self {
        Self {
            client: Client::new(),
            token,
            base_url: "https://gitlab.com".to_string(),
        }
    }

    fn project_path(owner: &str, repo: &str) -> String {
        // GitLab API requires "namespace%2Fproject" in URL
        format!("{}/{}", owner, repo).replace('/', "%2F")
    }

    fn gl_request(&self, url: &str) -> reqwest::RequestBuilder {
        self.client.get(url).header("PRIVATE-TOKEN", &self.token)
    }
}

async fn check_status(resp: reqwest::Response) -> Result<reqwest::Response, String> {
    match resp.status().as_u16() {
        401 => Err("Invalid GitLab token (401)".to_string()),
        403 => Err("GitLab access forbidden (403) — check token scopes".to_string()),
        404 => Err("Project not found (404) — check owner/repo in config".to_string()),
        s if s >= 400 => Err(format!("GitLab API error: HTTP {}", s)),
        _ => Ok(resp),
    }
}

#[derive(Deserialize)]
struct GlPipeline {
    id: u64,
    #[serde(rename = "ref")]
    branch: String,
    status: String,
    created_at: String,
    web_url: String,
}

#[derive(Deserialize)]
struct GlJob {
    id: u64,
    name: String,
    status: String,
}

fn map_status(status: &str) -> PipelineStatus {
    match status {
        "success" => PipelineStatus::Success,
        "failed" => PipelineStatus::Failed,
        "running" => PipelineStatus::Running,
        "pending" | "created" | "waiting_for_resource" | "preparing" | "scheduled" => {
            PipelineStatus::Pending
        }
        "canceled" => PipelineStatus::Cancelled,
        _ => PipelineStatus::Unknown,
    }
}

#[async_trait]
impl Provider for GitLabClient {
    async fn list_pipelines(&self, owner: &str, repo: &str) -> Result<Vec<Pipeline>, String> {
        let path = Self::project_path(owner, repo);
        let url = format!("{}/api/v4/projects/{}/pipelines?per_page=20", self.base_url, path);
        let pipelines: Vec<GlPipeline> = check_status(
            self.gl_request(&url).send().await.map_err(|e| e.to_string())?
        ).await?.json().await.map_err(|e| e.to_string())?;

        Ok(pipelines.into_iter().map(|p| Pipeline {
            id: p.id.to_string(),
            name: p.branch,
            status: map_status(&p.status),
            created_at: p.created_at,
            url: p.web_url,
        }).collect())
    }

    async fn list_jobs(&self, owner: &str, repo: &str, pipeline_id: &str) -> Result<Vec<Job>, String> {
        let path = Self::project_path(owner, repo);
        let url = format!("{}/api/v4/projects/{}/pipelines/{}/jobs", self.base_url, path, pipeline_id);
        let jobs: Vec<GlJob> = check_status(
            self.gl_request(&url).send().await.map_err(|e| e.to_string())?
        ).await?.json().await.map_err(|e| e.to_string())?;

        Ok(jobs.into_iter().map(|j| Job {
            id: j.id.to_string(),
            name: j.name,
            status: map_status(&j.status),
        }).collect())
    }

    async fn get_logs(&self, owner: &str, repo: &str, job_id: &str) -> Result<String, String> {
        let path = Self::project_path(owner, repo);
        let url = format!("{}/api/v4/projects/{}/jobs/{}/trace", self.base_url, path, job_id);
        let text = check_status(
            self.gl_request(&url).send().await.map_err(|e| e.to_string())?
        ).await?.text().await.map_err(|e| e.to_string())?;

        Ok(text)
    }

    async fn rerun_pipeline(&self, owner: &str, repo: &str, pipeline_id: &str) -> Result<(), String> {
        let path = Self::project_path(owner, repo);
        let url = format!("{}/api/v4/projects/{}/pipelines/{}/retry", self.base_url, path, pipeline_id);
        let resp = self.client
            .post(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .send().await.map_err(|e| e.to_string())?;

        check_status(resp).await.map(|_| ())
    }
}
