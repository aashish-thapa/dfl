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

    // Get the first command (e.g., "init", "link", "add").
    let command = &args[1];

    match command.as_str() {
        "init" => {
            handle_init_command()?;
        }
        "link" => {
            handle_link_command()?;
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

    // Get the user's home directory to determine the path for our dotfiles repo.
    let mut dotm_path = PathBuf::new();
    if let Some(home_dir) = dirs::home_dir() {
        dotm_path.push(home_dir);
        dotm_path.push(".dotm");
    } else {
        return Err(io::Error::new(io::ErrorKind::Other, "Could not find home directory"));
    }

    // Create the .dotm directory if it doesn't already exist.
    if let Err(e) = std::fs::create_dir_all(&dotm_path) {
        eprintln!("Error creating directory: {}", e);
        return Err(e);
    };
    println!("✅ Created directory: {:?}", dotm_path);

    // Run 'git init' inside the new directory to initialize the Git repository.
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
/// It moves a file into the dotm repository and creates a symlink in its place.
fn handle_add_command(file_path: &str) -> io::Result<()> {
    println!("Adding file: {}", file_path);

    // Get the user's home directory.
    let home_dir = dirs::home_dir().ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Could not find home directory"))?;
    let dotm_path = home_dir.join(".dotm");

    // Check if the dotm repository exists.
    if !dotm_path.exists() {
        eprintln!("Error: .dotm repository not found. Please run 'dotm init' first.");
        return Ok(());
    }

    // Get the source path, canonicalize it to get a full path.
    let source_path = PathBuf::from(file_path).canonicalize()?;
    let file_name = source_path.file_name().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid file path"))?;
    
    // Determine the destination path inside the .dotm repository.
    let destination_path = dotm_path.join(file_name);

    // Check if the source path exists and is a file.
    if !source_path.exists() || !source_path.is_file() {
        eprintln!("Error: Source file '{}' does not exist or is not a file.", file_path);
        return Ok(());
    }

    // Move the file from the source to the destination in the repository.
    if let Err(e) = fs::rename(&source_path, &destination_path) {
        eprintln!("Error moving file: {}", e);
        return Err(e);
    }
    println!("✅ Moved file to repository: {:?}", destination_path);

    // Create a symbolic link from the original location pointing to the new one.
    // Note: The `symlink` function is from `std::os::unix::fs`, so this tool
    // is currently specific to Unix-like operating systems (Linux/macOS).
    if let Err(e) = symlink(&destination_path, &source_path) {
        eprintln!("Error creating symlink: {}", e);
        return Err(e);
    }
    println!("✅ Created symlink at: {:?}", source_path);

    println!("\n🚀 Dotfile added and linked successfully!");
    println!("Don't forget to 'git add' and 'git commit' your changes.");

    Ok(())
}

/// Handles the 'link' command.
/// It creates symlinks for all files in the dotm repository to the home directory.
fn handle_link_command() -> io::Result<()> {
    println!("Linking dotfiles...");

    // Get the user's home directory and the dotm repository path.
    let home_dir = dirs::home_dir().ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Could not find home directory"))?;
    let dotm_path = home_dir.join(".dotm");

    // Check if the dotm repository exists.
    if !dotm_path.exists() {
        eprintln!("Error: .dotm repository not found. Please run 'dotm init' or clone your repository first.");
        return Ok(());
    }
    
    // Read the contents of the repository directory.
    for entry in fs::read_dir(&dotm_path)? {
        let entry = entry?;
        let file_path_in_repo = entry.path();

        // Skip directories and the .git directory.
        if !file_path_in_repo.is_file() || file_path_in_repo.file_name().and_then(|f| f.to_str()) == Some(".git") {
            continue;
        }

        let file_name = file_path_in_repo.file_name().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid file path"))?;
        let symlink_path = home_dir.join(file_name);

        // Check if a file with the same name already exists in the home directory.
        if symlink_path.exists() {
            println!("  - Skipping '{}': file already exists at link destination.", symlink_path.display());
            continue;
        }

        // Create the symbolic link.
        if let Err(e) = symlink(&file_path_in_repo, &symlink_path) {
            eprintln!("  - Error creating symlink for '{}': {}", file_path_in_repo.display(), e);
        } else {
            println!("✅ Linked '{}' to '{}'", file_path_in_repo.display(), symlink_path.display());
        }
    }

    println!("\n🚀 All managed dotfiles have been linked!");

    Ok(())
}

/// Prints the usage information for the program.
fn print_usage(program_name: &str) {
    println!("\nA command-line tool for managing your dotfiles with Git.");
    println!("\nUsage: {} <command> [arguments]", program_name);
    println!("\nCommands:");
    println!("  init            Initializes a new dotm repository in ~/.dotm.");
    println!("  add <file>      Moves a file to ~/.dotm and creates a symlink in its place.");
    println!("  link            Creates symlinks for all dotfiles from the repository to your home directory.");
    println!("  -h, --help      Prints this help message.");
    println!("");
}
