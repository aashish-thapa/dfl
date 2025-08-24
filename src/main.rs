// main.rs
use std::env;
use std::fs;
use std::io::{self};
use std::path::PathBuf;
use std::os::unix::fs::symlink;
use duct::cmd;
use dirs;

fn main() -> io::Result<()> {
    // Collect command-line arguments into a vector of strings.
    let args: Vec<String> = env::args().collect();

    // The first argument is the program name itself.
    if args.len() < 2 {
        print_usage(&args[0]);
        return Ok(());
    }

    // Get the first command (e.g., "init", "sync", "add", "remote").
    let command = &args[1];

    match command.as_str() {
        "init" => {
            handle_init_command()?;
        }
        "add" => {
            if args.len() < 3 {
                eprintln!("Error: 'add' command requires a file path.");
                print_usage(&args[0]);
                return Ok(());
            }
            let file_path = &args[2];
            handle_add_command(file_path)?;
        }
        "sync" => {
            handle_sync_command()?;
        }
        "remote" => {
            if args.len() < 4 || args[2].as_str() != "add" {
                eprintln!("Error: 'remote' command requires 'add' and a URL.");
                print_usage(&args[0]);
                return Ok(());
            }
            let url = &args[3];
            handle_remote_command(url)?;
        }
        "push" => {
            handle_push_command()?;
        }
        "pull" => {
            handle_pull_command()?;
        }
        "-h" | "--help" => {
            print_usage(&args[0]);
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage(&args[0]);
        }
    }

    Ok(())
}

/// A friendly and conversational message box function.
fn message_box(title: &str, message: &str) {
    println!("\n--- {} ---", title);
    println!("{}\n", message);
}

/// Handles the 'init' command.
/// It creates the ~/.dfl directory and initializes a Git repository inside it.
fn handle_init_command() -> io::Result<()> {
    message_box("Initializing dfl...", "Creating repository directory and initializing Git.");

    let mut dfl_path = PathBuf::new();
    if let Some(home_dir) = dirs::home_dir() {
        dfl_path.push(home_dir);
        dfl_path.push(".dfl");
    } else {
        return Err(io::Error::new(io::ErrorKind::Other, "Could not find home directory"));
    }

    if let Err(e) = std::fs::create_dir_all(&dfl_path) {
        return Err(io::Error::new(io::ErrorKind::Other, format!("Error creating directory: {}", e)));
    };
    println!("✅ Created directory: {:?}", dfl_path);

    if let Err(e) = cmd!("git", "init").dir(&dfl_path).run() {
        return Err(io::Error::new(io::ErrorKind::Other, format!("Error initializing git repository: {}", e)));
    }
    println!("✅ Git repository initialized.");

    message_box("dfl Initialized", &format!("You can now add your dotfiles. Your repository is at: {:?}", dfl_path));

    Ok(())
}

/// Handles the 'add' command.
/// It moves a file, creates a symlink, and automatically commits the change.
fn handle_add_command(file_path: &str) -> io::Result<()> {
    println!("Adding file: {}", file_path);

    let home_dir = dirs::home_dir().ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Could not find home directory"))?;
    let dfl_path = home_dir.join(".dfl");

    if !dfl_path.exists() {
        message_box("Error", "dfl repository not found. Please run 'dfl init' first.");
        return Ok(());
    }

    let source_path = PathBuf::from(file_path);
    let source_path_canonical = source_path.canonicalize()?;
    let file_name = source_path_canonical.file_name().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid file path"))?;
    let destination_path = dfl_path.join(file_name);

    if !source_path.exists() || !source_path.is_file() {
        message_box("Error", &format!("Source file '{}' does not exist or is not a file.", file_path));
        return Ok(());
    }

    // Move the file into the repository
    if let Err(e) = fs::rename(&source_path_canonical, &destination_path) {
        return Err(io::Error::new(io::ErrorKind::Other, format!("Error moving file: {}", e)));
    }
    println!("✅ Moved file to repository: {:?}", destination_path);

    // Create a symbolic link
    if let Err(e) = symlink(&destination_path, &source_path_canonical) {
        // If symlink creation fails, move the original file back to prevent data loss
        let _ = fs::rename(&destination_path, &source_path_canonical);
        return Err(io::Error::new(io::ErrorKind::Other, format!("Error creating symlink: {}. Original file has been restored.", e)));
    }
    println!("✅ Created symlink at: {:?}", source_path_canonical);

    // Automatically stage and commit the change
    println!("Automatically committing changes...");
    if let Err(e) = cmd!("git", "add", ".").dir(&dfl_path).run() {
        return Err(io::Error::new(io::ErrorKind::Other, format!("Error staging changes: {}", e)));
    }
    if let Err(e) = cmd!("git", "commit", "-m", &format!("feat: Add {}", file_name.to_string_lossy())).dir(&dfl_path).run() {
        return Err(io::Error::new(io::ErrorKind::Other, format!("Error committing changes: {}", e)));
    }
    println!("✅ Changes committed.");

    message_box("Success", "Dotfile added and linked successfully!");
    println!("Remember to add a remote and 'dfl push' to sync your changes.");

    Ok(())
}

/// Handles the 'sync' command.
/// It creates symlinks for all files in the dfl repository.
fn handle_sync_command() -> io::Result<()> {
    println!("Syncing dotfiles...");

    let home_dir = dirs::home_dir().ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Could not find home directory"))?;
    let dfl_path = home_dir.join(".dfl");

    if !dfl_path.exists() {
        message_box("Error", "dfl repository not found. Please run 'dfl init' or clone your repository first.");
        return Ok(());
    }

    // Link all files in the repository
    for entry in fs::read_dir(&dfl_path)? {
        let entry = entry?;
        let file_path_in_repo = entry.path();

        if file_path_in_repo.file_name().and_then(|f| f.to_str()) == Some(".git") {
            continue;
        }

        let file_name = file_path_in_repo.file_name().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid file path"))?;
        let symlink_path = home_dir.join(file_name);
        
        // Skip hidden directories and non-dotfiles
        if file_name.to_string_lossy().starts_with('.') && file_path_in_repo.is_dir() {
            continue;
        }

        if symlink_path.exists() {
            message_box("Warning", &format!("'{}' already exists. Backing up and creating symlink.", symlink_path.display()));
            let backup_path = home_dir.join(format!("{}.backup", file_name.to_string_lossy()));
            fs::rename(&symlink_path, &backup_path)?;
        }

        if let Err(e) = symlink(&file_path_in_repo, &symlink_path) {
            eprintln!("  - Error creating symlink for '{}': {}", file_path_in_repo.display(), e);
        } else {
            println!("✅ Synced '{}' to '{}'", file_path_in_repo.display(), symlink_path.display());
        }
    }

    message_box("Success", "All managed dotfiles have been synced!");
    Ok(())
}

/// Handles the new 'remote' command.
/// It adds a remote URL to the repository.
fn handle_remote_command(url: &str) -> io::Result<()> {
    message_box("Adding Remote Origin", "Connecting your local repository to a remote URL.");

    let home_dir = dirs::home_dir().ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Could not find home directory"))?;
    let dfl_path = home_dir.join(".dfl");

    if !dfl_path.exists() {
        message_box("Error", "dfl repository not found. Please run 'dfl init' first.");
        return Ok(());
    }

    // Add the remote origin
    if let Err(e) = cmd!("git", "remote", "add", "origin", url).dir(&dfl_path).run() {
        return Err(io::Error::new(io::ErrorKind::Other, format!("Error adding remote origin: {}", e)));
    }
    println!("✅ Remote 'origin' added: {}", url);

    message_box("Remote Added", "Your local repository is now connected to your remote!");

    Ok(())
}

/// Handles the new 'push' command.
/// It pushes committed changes to the remote repository.
fn handle_push_command() -> io::Result<()> {
    message_box("Pushing to Remote", "Uploading your committed changes to the remote repository.");
    let home_dir = dirs::home_dir().ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Could not find home directory"))?;
    let dfl_path = home_dir.join(".dfl");

    if !dfl_path.exists() {
        message_box("Error", "dfl repository not found. Please run 'dfl init' or clone your repository first.");
        return Ok(());
    }

    // Check if a remote named 'origin' exists
    let remotes_output = cmd!("git", "remote").dir(&dfl_path).read()?;
    if !remotes_output.contains("origin") {
        return Err(io::Error::new(io::ErrorKind::Other, "Error: No remote named 'origin' found. Please run 'dfl remote add <url>' first."));
    }

    // Check if the current branch has an upstream set
    let status_output = cmd!("git", "status", "-sb").dir(&dfl_path).read()?;
    let is_upstream_set = status_output.contains("...");

    if is_upstream_set {
        // Upstream is set, just do a normal push
        if let Err(e) = cmd!("git", "push").dir(&dfl_path).run() {
            return Err(io::Error::new(io::ErrorKind::Other, format!("Error pushing to remote: {}", e)));
        }
    } else {
        // No upstream set, perform an initial push
        message_box("Initial Push", "No upstream branch found. Setting upstream for you.");
        if let Err(e) = cmd!("git", "push", "--set-upstream", "origin", "master").dir(&dfl_path).run() {
            return Err(io::Error::new(io::ErrorKind::Other, format!("Error performing initial push: {}", e)));
        }
    }

    println!("✅ Changes pushed successfully!");

    message_box("Success", "Your dotfiles are now synced with your remote repository!");

    Ok(())
}
/// Handles the new 'pull' command.
/// It pulls changes from the remote repository.
fn handle_pull_command() -> io::Result<()> {
    message_box("Pulling from Remote", "Fetching the latest changes from your remote repository.");

    let home_dir = dirs::home_dir().ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Could not find home directory"))?;
    let dfl_path = home_dir.join(".dfl");

    if !dfl_path.exists() {
        message_box("Error", "dfl repository not found. Please run 'dfl init' or clone your repository first.");
        return Ok(());
    }

    if let Err(e) = cmd!("git", "pull").dir(&dfl_path).run() {
        return Err(io::Error::new(io::ErrorKind::Other, format!("Error pulling from remote: {}", e)));
    }
    println!("✅ Pulled latest changes successfully!");

    message_box("Success", "Your local dotfiles repository is now up-to-date. Run 'dfl sync' to apply the changes.");

    Ok(())
}

/// Prints the usage information for the program.
fn print_usage(program_name: &str) {
    println!("\nA command-line tool for managing your dotfiles with Git.");
    println!("\nUsage: {} <command> [arguments]", program_name);
    println!("\nCommands:");
    println!("  init            Initializes a new dfl repository in ~/.dfl.");
    println!("  add <file>      Moves a file to ~/.dfl and creates a symlink, then automatically commits the change.");
    println!("  sync            Creates symlinks for all dotfiles from the repository to your home directory.");
    println!("  remote add <url> Adds a remote URL (e.g., a GitHub repository) to your dfl repository.");
    println!("  push            Pushes your committed changes to the remote repository.");
    println!("  pull            Pulls the latest changes from the remote repository.");
    println!("  -h, --help      Prints this help message.");
    println!("");
}
