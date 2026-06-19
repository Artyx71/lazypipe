mod config;
mod provider;
mod github;
mod gitlab;
mod state;
mod poller;

fn main() {
    match config::load_config() {
        Ok(cfg) => {
            println!("Config loaded. {} repo(s):", cfg.repos.len());
            for repo in &cfg.repos {
                println!("  [{:}] {}/{}", repo.provider, repo.owner, repo.repo);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
