use async_trait::async_trait;

pub enum PipelineStatus {
    Success,
    Failed,
    Running,
    Pending,
    Cancelled,
    Unknown,
}

pub struct Pipeline {
    pub id: String,
    pub name: String,
    pub status: PipelineStatus,
    pub created_at: String,
    pub url: String,
}

pub struct Job {
    pub id: String,
    pub name: String,
    pub status: PipelineStatus,
}

#[async_trait]
pub trait Provider: Send + Sync {
    async fn list_pipelines(&self, owner: &str, repo: &str) -> Result<Vec<Pipeline>, String>;
    async fn list_jobs(&self, owner: &str, repo: &str, pipeline_id: &str) -> Result<Vec<Job>, String>;
    async fn get_logs(&self, owner: &str, repo: &str, job_id: &str) -> Result<String, String>;
    async fn rerun_pipeline(&self, owner: &str, repo: &str, pipeline_id: &str) -> Result<(), String>;
}
