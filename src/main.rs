use clap::Parser;
use git2::Repository;
use serde::Deserialize;
use std::fs;
use std::path::Path;
// use std::thread;
// use std::time::Duration;
use tokio;
// Define a struct to deserialize the JSON config file
#[derive(Deserialize)]
struct Config {
    repositories: Vec<String>,
}

/// Clones repositories based on a schedule or immediately.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Sets a custom config file.
    #[clap(short, long, default_value = "config.json")]
    config: String,

    /// Sets the output directory for cloned repositories.
    #[clap(short, long, default_value = "./cloned_repos")]
    output: String,

    /// Sets the schedule for cloning (in hours).
    #[clap(short, long)]
    schedule: Option<u64>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let config_content = fs::read_to_string(&args.config).expect("Failed to read config file");
    let config: Config =
        serde_json::from_str(&config_content).expect("Failed to parse config file");

    let tasks = config.repositories.into_iter().map(|repo_url| {
        let output_dir = args.output.clone();
        tokio::spawn(async move {
            let repo_output_path = Path::new(&output_dir)
                .join(repo_url.split('/').last().unwrap().trim_end_matches(".git"));

            // Check if the output path exists and rename it if it does
            if repo_output_path.exists() {
                let old_path = repo_output_path.with_extension("old");
                match fs::rename(&repo_output_path, &old_path) {
                    Ok(_) => println!("Renamed existing directory to {:?}", old_path),
                    Err(e) => eprintln!("Error renaming existing directory: {}", e),
                }
            }

            // Attempt to clone the repository
            match Repository::clone(&repo_url, &repo_output_path) {
                Ok(_) => {
                    println!("Successfully cloned {}", repo_url);
                    // Remove the _old directory if it exists
                    let old_path = repo_output_path.with_extension("old");
                    if old_path.exists() {
                        match fs::remove_dir_all(&old_path) {
                            Ok(_) => println!("Removed old directory {:?}", old_path),
                            Err(e) => eprintln!("Error removing old directory: {}", e),
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to clone {}: {}", repo_url, e);
                    // Rename _old back to original if cloning failed
                    let old_path = repo_output_path.with_extension("old");
                    if old_path.exists() {
                        match fs::rename(&old_path, &repo_output_path) {
                            Ok(_) => println!("Restored old directory to {:?}", repo_output_path),
                            Err(e) => eprintln!("Error restoring old directory: {}", e),
                        }
                    }
                }
            }
        })
    });

    // Use join_all to await all tasks
    let _: Vec<_> = futures::future::join_all(tasks).await.into_iter().collect();

    // let clone_repos = |config_path: &str, output_dir: &str| {
    //     let config_content = fs::read_to_string(config_path).expect("Failed to read config file");
    //     let config: Config =
    //         serde_json::from_str(&config_content).expect("Failed to parse config file");

    //     for repo_url in &config.repositories {
    //         println!("Cloning repository: {}", repo_url);
    //         let repo_output_path = Path::new(output_dir)
    //             .join(repo_url.split('/').last().unwrap().trim_end_matches(".git"));

    //         // Check if the output path exists and rename it if it does
    //         if repo_output_path.exists() {
    //             let old_path = repo_output_path.with_extension("old");
    //             match fs::rename(&repo_output_path, &old_path) {
    //                 Ok(_) => println!("Renamed existing directory to {:?}", old_path),
    //                 Err(e) => eprintln!("Error renaming existing directory: {}", e),
    //             }
    //         }

    //         // Attempt to clone the repository
    //         match Repository::clone(repo_url, &repo_output_path) {
    //             Ok(_) => {
    //                 println!("Successfully cloned {}", repo_url);
    //                 // Remove the _old directory if it exists
    //                 let old_path = repo_output_path.with_extension("old");
    //                 if old_path.exists() {
    //                     match fs::remove_dir_all(&old_path) {
    //                         Ok(_) => println!("Removed old directory {:?}", old_path),
    //                         Err(e) => eprintln!("Error removing old directory: {}", e),
    //                     }
    //                 }
    //             }
    //             Err(e) => {
    //                 eprintln!("Failed to clone {}: {}", repo_url, e);
    //                 // Rename _old back to original if cloning failed
    //                 let old_path = repo_output_path.with_extension("old");
    //                 if old_path.exists() {
    //                     match fs::rename(&old_path, &repo_output_path) {
    //                         Ok(_) => println!("Restored old directory to {:?}", repo_output_path),
    //                         Err(e) => eprintln!("Error restoring old directory: {}", e),
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // };

    // match args.schedule {
    //     Some(schedule) => loop {
    //         clone_repos(&args.config, &args.output);
    //         thread::sleep(Duration::from_secs(schedule * 3600)); // Convert schedule from hours to seconds
    //     },
    //     None => clone_repos(&args.config, &args.output),
    // }
}
