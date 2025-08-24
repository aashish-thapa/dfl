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

    // Get the first command (e.g., "init", "link", "add", "remote").
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
        "link" => {
            handle_link_command()?;
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

/// Handles the 'init' command.
/// It creates the ~/.dotm directory and initializes a Git repository inside it.
fn handle_init_command() -> io::Result<()> {
    println!("Initializing dotm...");

    let mut dotm_path = PathBuf::new();
    if let Some(home_dir) = dirs::home_dir() {
        dotm_path.push(home_dir);
        dotm_path.push(".dotm");
    } else {
        return Err(io::Error::new(io::ErrorKind::Other, "Could not find home directory"));
    }

    if let Err(e) = std::fs::create_dir_all(&dotm_path) {
        eprintln!("Error creating directory: {}", e);
        return Err(e);
    };
    println!("✅ Created directory: {:?}", dotm_path);

    if let Err(e) = cmd!("git", "init").dir(&dotm_path).run() {
        eprintln!("Error initializing git repository: {}", e);
        return Err(e);
    }
    println!("✅ Git repository initialized.");

    println!("\n🚀 dotm initialized successfully! You can now add your dotfiles.");
    println!("Your repository is located at: {:?}", dotm_path);

    Ok(())
}

/// Handles the 'add' command.
/// It moves a file, creates a symlink, and automatically commits the change.
fn handle_add_command(file_path: &str) -> io::Result<()> {
    println!("Adding file: {}", file_path);

    let home_dir = dirs::home_dir().ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Could not find home directory"))?;
    let dotm_path = home_dir.join(".dotm");

    if !dotm_path.exists() {
        eprintln!("Error: .dotm repository not found. Please run 'dotm init' first.");
        return Ok(());
    }

    let source_path = PathBuf::from(file_path).canonicalize()?;
    let file_name = source_path.file_name().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid file path"))?;
    let destination_path = dotm_path.join(file_name);

    if !source_path.exists() || !source_path.is_file() {
        eprintln!("Error: Source file '{}' does not exist or is not a file.", file_path);
        return Ok(());
    }

    // Move the file into the repository
    if let Err(e) = fs::rename(&source_path, &destination_path) {
        eprintln!("Error moving file: {}", e);
        return Err(e);
    }
    println!("✅ Moved file to repository: {:?}", destination_path);

    // Create a symbolic link
    if let Err(e) = symlink(&destination_path, &source_path) {
        eprintln!("Error creating symlink: {}", e);
        return Err(e);
    }
    println!("✅ Created symlink at: {:?}", source_path);

    // Automatically stage and commit the change
    println!("Automatically committing changes...");
    if let Err(e) = cmd!("git", "add", ".").dir(&dotm_path).run() {
        eprintln!("Error staging changes: {}", e);
        return Err(e);
    }
    if let Err(e) = cmd!("git", "commit", "-m", &format!("feat: Add {}", file_name.to_string_lossy())).dir(&dotm_path).run() {
        eprintln!("Error committing changes: {}", e);
        return Err(e);
    }
    println!("✅ Changes committed.");

    println!("\n🚀 Dotfile added and linked successfully!");
    println!("Remember to 'dotm remote add <url>' and 'git push' to sync your changes.");

    Ok(())
}

/// Handles the 'link' command.
/// It pulls the latest changes and creates symlinks for all files in the dotm repository.
fn handle_link_command() -> io::Result<()> {
    println!("Linking dotfiles...");

    let home_dir = dirs::home_dir().ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Could not find home directory"))?;
    let dotm_path = home_dir.join(".dotm");

    if !dotm_path.exists() {
        eprintln!("Error: .dotm repository not found. Please run 'dotm init' or clone your repository first.");
        return Ok(());
    }

    // Pull the latest changes from the remote repository
    println!("Pulling latest changes from remote...");
    match cmd!("git", "pull").dir(&dotm_path).run() {
        Ok(_) => println!("✅ Pulled latest changes."),
        Err(e) => eprintln!("Warning: Failed to pull from remote: {}. Continuing without pull.", e),
    }

    // Link all files in the repository
    for entry in fs::read_dir(&dotm_path)? {
        let entry = entry?;
        let file_path_in_repo = entry.path();

        if !file_path_in_repo.is_file() || file_path_in_repo.file_name().and_then(|f| f.to_str()) == Some(".git") {
            continue;
        }

        let file_name = file_path_in_repo.file_name().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid file path"))?;
        let symlink_path = home_dir.join(file_name);

        if symlink_path.exists() {
            println!("  - Skipping '{}': file already exists at link destination.", symlink_path.display());
            continue;
        }

        if let Err(e) = symlink(&file_path_in_repo, &symlink_path) {
            eprintln!("  - Error creating symlink for '{}': {}", file_path_in_repo.display(), e);
        } else {
            println!("✅ Linked '{}' to '{}'", file_path_in_repo.display(), symlink_path.display());
        }
    }

    println!("\n🚀 All managed dotfiles have been linked!");
    Ok(())
}

/// Handles the new 'remote' command.
/// It adds a remote URL to the repository.
fn handle_remote_command(url: &str) -> io::Result<()> {
    println!("Adding remote origin...");

    let home_dir = dirs::home_dir().ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Could not find home directory"))?;
    let dotm_path = home_dir.join(".dotm");

    if !dotm_path.exists() {
        eprintln!("Error: .dotm repository not found. Please run 'dotm init' first.");
        return Ok(());
    }

    // Add the remote origin
    if let Err(e) = cmd!("git", "remote", "add", "origin", url).dir(&dotm_path).run() {
        eprintln!("Error adding remote origin: {}", e);
        return Err(e);
    }
    println!("✅ Remote 'origin' added: {}", url);

    println!("\n🚀 Your local repository is now connected to your remote! You can now use 'git push' to upload your dotfiles.");

    Ok(())
}

/// Prints the usage information for the program.
fn print_usage(program_name: &str) {
    println!("\nA command-line tool for managing your dotfiles with Git.");
    println!("\nUsage: {} <command> [arguments]", program_name);
    println!("\nCommands:");
    println!("  init            Initializes a new dotm repository in ~/.dotm.");
    println!("  add <file>      Moves a file to ~/.dotm and creates a symlink, then automatically commits the change.");
    println!("  link            Pulls the latest changes and creates symlinks for all dotfiles from the repository.");
    println!("  remote add <url> Adds a remote URL (e.g., a GitHub repository) to your dotm repository.");
    println!("  -h, --help      Prints this help message.");
    println!("");
}
