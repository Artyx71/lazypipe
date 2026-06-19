use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::config::RepoConfig;
use crate::github::GitHubClient;
use crate::gitlab::GitLabClient;
use crate::provider::{PipelineStatus, Provider};
use crate::state::AppState;

fn make_provider(repo: &RepoConfig) -> Box<dyn Provider> {
    match repo.provider.as_str() {
        "gitlab" => Box::new(GitLabClient::new(repo.token.clone())),
        _ => Box::new(GitHubClient::new(repo.token.clone())),
    }
}

pub async fn start_polling(state: Arc<Mutex<AppState>>) {
    let state_p = Arc::clone(&state);
    let state_l = Arc::clone(&state);

    tokio::spawn(async move {
        loop {
            poll_pipelines(&state_p).await;
            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    });

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;
            poll_logs(&state_l).await;
        }
    });
}

async fn poll_pipelines(state: &Arc<Mutex<AppState>>) {
    let repos = state.lock().unwrap().repos.clone();

    let mut handles = vec![];
    for repo in repos {
        handles.push(tokio::spawn(async move {
            let provider = make_provider(&repo);
            let result = provider.list_pipelines(&repo.owner, &repo.repo).await;
            (repo.name.clone(), result)
        }));
    }

    // Collect all results outside the lock — can't .await while holding MutexGuard
    let mut results = vec![];
    for handle in handles {
        results.push(handle.await);
    }

    let mut s = state.lock().unwrap();
    for result in results {
        match result {
            Ok((name, Ok(pipelines))) => { s.pipelines.insert(name, pipelines); }
            Ok((_, Err(e))) => { s.error = Some(e); }
            Err(_) => {}
        }
    }
    s.last_updated = Some(std::time::Instant::now());
}

async fn poll_logs(state: &Arc<Mutex<AppState>>) {
    let info = {
        let s = state.lock().unwrap();
        let repo = s.current_repo().cloned();
        let job = s.current_jobs().get(s.selected_job).cloned();
        repo.zip(job)
    };

    if let Some((repo, job)) = info {
        if matches!(job.status, PipelineStatus::Running) {
            let provider = make_provider(&repo);
            if let Ok(logs) = provider.get_logs(&repo.owner, &repo.repo, &job.id).await {
                state.lock().unwrap().logs.insert(job.id, logs);
            }
        }
    }
}
