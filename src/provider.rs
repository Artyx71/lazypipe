use async_trait::async_trait;

#[derive(Clone, Debug)]
pub enum PipelineStatus {
    Success,
    Failed,
    Running,
    Pending,
    Cancelled,
    Unknown,
}

#[derive(Clone, Debug)]
pub struct Pipeline {
    pub id: String,
    pub name: String,
    pub status: PipelineStatus,
    pub created_at: String,
    pub url: String,
}

#[derive(Clone, Debug)]
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
    async fn cancel_pipeline(&self, owner: &str, repo: &str, pipeline_id: &str) -> Result<(), String>;
}
