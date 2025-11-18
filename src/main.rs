use std::process::Command;
use std::io::{self, Write};
use std::time::Duration;
use std::thread;
use std::env;

struct Config {
    dry_run: bool,
    verbose: bool,
}

fn print_help() {
    println!("A.S.S. - Automated System Setup");
    println!();
    println!("USAGE:");
    println!("    a-s-s [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    --help, -h       Show this help message");
    println!("    --dry-run        Show what would be done without executing");
    println!("    --verbose, -v    Show detailed output");
    println!();
    println!("EXAMPLES:");
    println!("    a-s-s                    # Run the setup");
    println!("    a-s-s --dry-run          # Test without making changes");
    println!("    a-s-s --verbose          # Run with detailed output");
}

fn parse_args() -> Config {
    let args: Vec<String> = env::args().collect();
    let mut config = Config {
        dry_run: false,
        verbose: false,
    };
    
    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            "--dry-run" => config.dry_run = true,
            "--verbose" | "-v" => config.verbose = true,
            _ => {
                eprintln!("Unknown option: {}", arg);
                eprintln!("Use --help for usage information");
                std::process::exit(1);
            }
        }
    }
    
    config
}


// For now will simply check for git installation
fn check_deps(config: &Config) {
    if config.verbose {
        println!("Checking for required dependencies...");
    }
    
    if config.dry_run {
        println!("[DRY RUN] Would check for git installation");
        return;
    }
    
    let output = Command::new("which")
        .arg("git")
        .output()
        .expect("Failed to execute which command");
    
    if output.stdout.is_empty() {
        println!("Git not found. Installing git...");
        let status = Command::new("sudo")
            .args(&["pacman", "-S", "--noconfirm", "git"])
            .status()
            .expect("Failed to install git");
        
        if !status.success() {
            eprintln!("Failed to install git");
            std::process::exit(1);
        }
        println!("✓ Git installed successfully");
    } else if config.verbose {
        println!("Path to git: {:?}", String::from_utf8_lossy(&output.stdout));
    }
}

// just wait 3 seconds and prompt user
fn check_connection(config: &Config) {
    if config.verbose {
        println!("Checking network connection...");
    }
    
    if config.dry_run {
        println!("[DRY RUN] Would wait 3 seconds and prompt for connection");
        return;
    }
    
    println!("Checking network connection...");
    thread::sleep(Duration::from_secs(3));
    
    println!("⚠ Could not verify connection (timeout after 3s)");
    print!("Continue anyway? [Y/n]: ");
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let input = input.trim().to_lowercase();
    
    if input == "n" || input == "no" {
        println!("Aborted.");
        std::process::exit(0);
    }
}

// proceed to install and setup paru (the greatest aur helper ever made)
fn install_paru(config: &Config) {
    println!("Installing paru...");
    
    if config.dry_run {
        println!("[DRY RUN] Would execute:");
        println!("  1. git clone https://aur.archlinux.org/paru.git");
        println!("  2. sudo pacman -Syyu --noconfirm rustup bat devtools");
        println!("  3. rustup default stable");
        println!("  4. cd paru && makepkg -si --noconfirm");
        return;
    }
    
    // Clone paru repo
    if config.verbose {
        println!("Cloning paru AUR repository...");
    }
    let status = Command::new("git")
        .args(&["clone", "https://aur.archlinux.org/paru.git"])
        .status()
        .expect("Failed to execute git clone");
    
    if !status.success() {
        eprintln!("Failed to clone paru repository");
        std::process::exit(1);
    }
    
    // Install dependencies
    if config.verbose {
        println!("Installing dependencies (rustup, bat, devtools)...");
    }
    let status = Command::new("sudo")
        .args(&["pacman", "-Syyu", "--noconfirm", "rustup", "bat", "devtools"])
        .status()
        .expect("Failed to execute pacman");
    
    if !status.success() {
        eprintln!("Failed to install dependencies");
        std::process::exit(1);
    }
    
    // Setup rust stable
    if config.verbose {
        println!("Setting up Rust stable toolchain...");
    }
    let status = Command::new("rustup")
        .args(&["default", "stable"])
        .status()
        .expect("Failed to execute rustup");
    
    if !status.success() {
        eprintln!("Failed to setup rust stable");
        std::process::exit(1);
    }
    
    // Build and install paru
    if config.verbose {
        println!("Building and installing paru...");
    }
    let status = Command::new("makepkg")
        .args(&["-si", "--noconfirm"])
        .current_dir("./paru")
        .status()
        .expect("Failed to execute makepkg");
    
    if !status.success() {
        eprintln!("Failed to build/install paru");
        std::process::exit(1);
    }
    
    println!("✓ Paru installed successfully!");
}

// Clone dotfiles and install packages
fn setup_dotfiles(config: &Config) {
    println!("Setting up dotfiles...");
    
    if config.dry_run {
        println!("[DRY RUN] Would execute:");
        println!("  1. cd ~");
        println!("  2. git clone https://github.com/jeebuscrossaint/dotfiles.git");
        println!("  3. cd dotfiles");
        println!("  4. paru -S --needed - < archpkglist.txt");
        return;
    }
    
    // Get home directory
    let home = env::var("HOME").expect("HOME environment variable not set");
    
    // Clone dotfiles repo
    if config.verbose {
        println!("Cloning dotfiles repository to {}...", home);
    }
    let status = Command::new("git")
        .args(&["clone", "https://github.com/jeebuscrossaint/dotfiles.git"])
        .current_dir(&home)
        .status()
        .expect("Failed to execute git clone");
    
    if !status.success() {
        eprintln!("Failed to clone dotfiles repository");
        std::process::exit(1);
    }
    
    // Install packages from pkglist.txt
    if config.verbose {
        println!("Installing packages from pkglist.txt...");
    }
    
    let dotfiles_path = format!("{}/dotfiles", home);
    let pkglist_path = format!("{}/pkglist.txt", dotfiles_path);
    
    let status = Command::new("paru")
        .args(&["-S", "--needed", "-"])
        .current_dir(&dotfiles_path)
        .stdin(std::fs::File::open(&pkglist_path).expect("Failed to open pkglist.txt"))
        .status()
        .expect("Failed to execute paru");
    
    if !status.success() {
        eprintln!("Failed to install packages from pkglist.txt");
        std::process::exit(1);
    }
    
    println!("✓ Dotfiles setup complete!");
}

// Install stow and deploy dotfiles
fn deploy_dotfiles(config: &Config) {
    println!("Deploying dotfiles with GNU Stow...");
    
    if config.dry_run {
        println!("[DRY RUN] Would execute:");
        println!("  1. sudo pacman -S --noconfirm stow");
        println!("  2. mkdir -p ~/.config");
        println!("  3. cd ~/dotfiles && stow home-manager");
        println!("  4. cd ~/dotfiles && stow nix");
        return;
    }
    
    // Install GNU Stow
    if config.verbose {
        println!("Installing GNU Stow...");
    }
    let status = Command::new("sudo")
        .args(&["pacman", "-S", "--noconfirm", "stow"])
        .status()
        .expect("Failed to execute pacman");
    
    if !status.success() {
        eprintln!("Failed to install stow");
        std::process::exit(1);
    }
    
    let home = env::var("HOME").expect("HOME environment variable not set");
    let config_path = format!("{}/.config", home);
    let dotfiles_path = format!("{}/dotfiles", home);
    
    // Create .config directory
    if config.verbose {
        println!("Creating ~/.config directory...");
    }
    let status = Command::new("mkdir")
        .args(&["-p", &config_path])
        .status()
        .expect("Failed to create .config directory");
    
    if !status.success() {
        eprintln!("Failed to create .config directory");
        std::process::exit(1);
    }
    
    // Stow home-manager
    if config.verbose {
        println!("Stowing home-manager...");
    }
    let status = Command::new("stow")
        .arg("home-manager")
        .current_dir(&dotfiles_path)
        .status()
        .expect("Failed to stow home-manager");
    
    if !status.success() {
        eprintln!("Failed to stow home-manager");
        std::process::exit(1);
    }
    
    // Stow nix
    if config.verbose {
        println!("Stowing nix...");
    }
    let status = Command::new("stow")
        .arg("nix")
        .current_dir(&dotfiles_path)
        .status()
        .expect("Failed to stow nix");
    
    if !status.success() {
        eprintln!("Failed to stow nix");
        std::process::exit(1);
    }
    
    println!("✓ Dotfiles deployed successfully!");
}

fn main() {
    let config = parse_args();
    
    if config.dry_run {
        println!("=== DRY RUN MODE ===");
        println!("No actual changes will be made\n");
    }
    
    println!("A.S.S. - Arch Setup Script");
    check_deps(&config);
    check_connection(&config);
    install_paru(&config);
    setup_dotfiles(&config);
    deploy_dotfiles(&config);
    
    if config.dry_run {
        println!("\n=== DRY RUN COMPLETE ===");
    } else {
        println!("\n✓ Setup complete!");
    }
}
