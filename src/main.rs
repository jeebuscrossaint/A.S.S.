use std::process::Command;
use std::io::{self, Write};
use std::time::Duration;
use std::thread;
use std::env;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;

struct Config {
    dry_run: bool,
    verbose: bool,
}

fn print_help() {
    println!("A.S.S. - Automated System Setup");
    println!();
    println!("USAGE:");
    println!("    ass [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    --help, -h       Show this help message");
    println!("    --dry-run        Show what would be done without executing");
    println!("    --verbose, -v    Show detailed output");
    println!();
    println!("EXAMPLES:");
    println!("    ass                    # Run the setup");
    println!("    ass --dry-run          # Test without making changes");
    println!("    ass --verbose          # Run with detailed output");
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
        println!("[DRY RUN] Would check for: git, curl, sudo, systemctl");
        return;
    }
    
    let mut missing_deps = Vec::new();
    
    // Check for git
    let output = Command::new("which")
        .arg("git")
        .output()
        .expect("Failed to execute which command");
    
    if output.stdout.is_empty() {
        missing_deps.push("git");
    } else if config.verbose {
        println!("✓ Found git: {}", String::from_utf8_lossy(&output.stdout).trim());
    }
    
    // Check for curl (needed for Nix installer)
    let output = Command::new("which")
        .arg("curl")
        .output()
        .expect("Failed to execute which command");
    
    if output.stdout.is_empty() {
        missing_deps.push("curl");
    } else if config.verbose {
        println!("✓ Found curl: {}", String::from_utf8_lossy(&output.stdout).trim());
    }
    
    // Check for sudo
    let output = Command::new("which")
        .arg("sudo")
        .output()
        .expect("Failed to execute which command");
    
    if output.stdout.is_empty() {
        eprintln!("ERROR: sudo is required but not found");
        std::process::exit(1);
    } else if config.verbose {
        println!("✓ Found sudo: {}", String::from_utf8_lossy(&output.stdout).trim());
    }
    
    // Check for systemctl (needed for Nix daemon)
    let output = Command::new("which")
        .arg("systemctl")
        .output()
        .expect("Failed to execute which command");
    
    if output.stdout.is_empty() {
        eprintln!("ERROR: systemctl is required but not found (are you on systemd?)");
        std::process::exit(1);
    } else if config.verbose {
        println!("✓ Found systemctl: {}", String::from_utf8_lossy(&output.stdout).trim());
    }
    
    // Install missing dependencies
    if !missing_deps.is_empty() {
        println!("Installing missing dependencies: {}", missing_deps.join(", "));
        let mut args = vec!["-S", "--noconfirm"];
        args.extend(missing_deps.iter().map(|s| *s));
        
        let status = Command::new("sudo")
            .arg("pacman")
            .args(&args)
            .status()
            .expect("Failed to install dependencies");
        
        if !status.success() {
            eprintln!("Failed to install dependencies");
            std::process::exit(1);
        }
        println!("✓ Dependencies installed successfully");
    } else if config.verbose {
        println!("✓ All required dependencies are installed");
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
        println!("[DRY RUN] Would check if paru is installed, if not:");
        println!("  1. git clone https://aur.archlinux.org/paru.git");
        println!("  2. sudo pacman -Syyu --noconfirm rustup bat devtools");
        println!("  3. rustup default stable");
        println!("  4. cd paru && makepkg -si --noconfirm");
        return;
    }
    
    // Check if paru is already installed
    let output = Command::new("which")
        .arg("paru")
        .output()
        .expect("Failed to execute which command");
    
    if !output.stdout.is_empty() {
        if config.verbose {
            println!("✓ Paru is already installed: {}", String::from_utf8_lossy(&output.stdout).trim());
        } else {
            println!("✓ Paru already installed, skipping installation");
        }
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
        println!("  1. Check if ~/dotfiles exists");
        println!("  2. cd ~");
        println!("  3. git clone --depth=1 https://github.com/jeebuscrossaint/dotfiles.git");
        println!("  4. cd dotfiles");
        println!("  5. Filter out invalid packages and run paru -S --needed --noconfirm --skipreview --batchinstall");
        return;
    }
    
    // Get home directory
    let home = env::var("HOME").expect("HOME environment variable not set");
    let dotfiles_path = format!("{}/dotfiles", home);
    
    // Check if dotfiles already exists
    if Path::new(&dotfiles_path).exists() {
        if config.verbose {
            println!("✓ Dotfiles directory already exists at {}", dotfiles_path);
        } else {
            println!("✓ Dotfiles already cloned, skipping clone");
        }
    } else {
        // Clone dotfiles repo with --depth=1
        if config.verbose {
            println!("Cloning dotfiles repository to {} (shallow clone)...", home);
        }
        let status = Command::new("git")
            .args(&["clone", "--depth=1", "https://github.com/jeebuscrossaint/dotfiles.git"])
            .current_dir(&home)
            .status()
            .expect("Failed to execute git clone");
        
        if !status.success() {
            eprintln!("Failed to clone dotfiles repository");
            std::process::exit(1);
        }
    }
    
    // Install packages from archpkglist.txt
    if config.verbose {
        println!("Installing packages from archpkglist.txt...");
    }
    
    let pkglist_path = format!("{}/archpkglist.txt", dotfiles_path);
    
    // Read the package list and filter out problematic packages
    let pkglist_content = std::fs::read_to_string(&pkglist_path)
        .expect("Failed to read archpkglist.txt");
    
    let filtered_packages: Vec<&str> = pkglist_content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter(|line| *line != "paru-debug") // Filter out paru-debug
        .collect();
    
    if config.verbose {
        println!("Installing {} packages (filtered out invalid packages)", filtered_packages.len());
    }
    
    // Create a temporary filtered package list
    let temp_pkglist = "/tmp/ass-filtered-pkglist.txt";
    std::fs::write(temp_pkglist, filtered_packages.join("\n"))
        .expect("Failed to write temporary package list");
    
    let status = Command::new("paru")
        .args(&["-S", "--needed", "--noconfirm", "--skipreview", "--batchinstall", "-"])
        .current_dir(&dotfiles_path)
        .stdin(std::fs::File::open(temp_pkglist).expect("Failed to open temp package list"))
        .status()
        .expect("Failed to execute paru");
    
    // Clean up temp file
    let _ = std::fs::remove_file(temp_pkglist);
    
    if !status.success() {
        eprintln!("Failed to install packages from archpkglist.txt");
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
    
    println!("✓ Stow installed and directories prepared!");
}

// Stow custom configs after initial home-manager generation
fn stow_custom_configs(config: &Config) {
    println!("Deploying custom dotfiles with GNU Stow...");
    
    if config.dry_run {
        println!("[DRY RUN] Would execute:");
        println!("  1. Remove default ~/.config/home-manager");
        println!("  2. Remove default ~/.config/nix");
        println!("  3. cd ~/dotfiles && stow home-manager");
        println!("  4. cd ~/dotfiles && stow nix");
        return;
    }
    
    let home = env::var("HOME").expect("HOME environment variable not set");
    let dotfiles_path = format!("{}/dotfiles", home);
    let hm_config_path = format!("{}/.config/home-manager", home);
    let nix_config_path = format!("{}/.config/nix", home);
    
    // Remove default home-manager config
    if Path::new(&hm_config_path).exists() {
        if config.verbose {
            println!("Removing default home-manager config...");
        }
        let status = Command::new("rm")
            .args(&["-rf", &hm_config_path])
            .status()
            .expect("Failed to remove home-manager config");
        
        if !status.success() {
            eprintln!("Failed to remove default home-manager config");
            std::process::exit(1);
        }
    }
    
    // Remove default nix config
    if Path::new(&nix_config_path).exists() {
        if config.verbose {
            println!("Removing default nix config...");
        }
        let status = Command::new("rm")
            .args(&["-rf", &nix_config_path])
            .status()
            .expect("Failed to remove nix config");
        
        if !status.success() {
            eprintln!("Failed to remove default nix config");
            std::process::exit(1);
        }
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
    
    println!("✓ Custom dotfiles deployed successfully!");
}

// Install Nix package manager
fn install_nix(config: &Config) {
    println!("Installing Nix package manager...");
    
    if config.dry_run {
        println!("[DRY RUN] Would execute:");
        println!("  1. Check if nix is already installed");
        println!("  2. cd ~");
        println!("  3. curl --proto '=https' --tlsv1.2 -sSfL https://nixos.org/nix/install -o nix-install.sh");
        println!("  4. chmod +x nix-install.sh");
        println!("  5. sh ./nix-install.sh --daemon");
        return;
    }
    
    // Check if nix is already installed
    let output = Command::new("which")
        .arg("nix")
        .output()
        .expect("Failed to execute which command");
    
    if !output.stdout.is_empty() {
        if config.verbose {
            println!("✓ Nix is already installed: {}", String::from_utf8_lossy(&output.stdout).trim());
        } else {
            println!("✓ Nix already installed, skipping installation");
        }
        return;
    }
    
    let home = env::var("HOME").expect("HOME environment variable not set");
    
    // Download Nix installer
    if config.verbose {
        println!("Downloading Nix installer to {}...", home);
    }
    let status = Command::new("curl")
        .args(&[
            "--proto", "=https",
            "--tlsv1.2",
            "-sSfL",
            "https://nixos.org/nix/install",
            "-o", "nix-install.sh"
        ])
        .current_dir(&home)
        .status()
        .expect("Failed to execute curl");
    
    if !status.success() {
        eprintln!("Failed to download Nix installer");
        std::process::exit(1);
    }
    
    // Make installer executable
    if config.verbose {
        println!("Making installer executable...");
    }
    let nix_installer_path = format!("{}/nix-install.sh", home);
    let status = Command::new("chmod")
        .args(&["+x", &nix_installer_path])
        .status()
        .expect("Failed to execute chmod");
    
    if !status.success() {
        eprintln!("Failed to make Nix installer executable");
        std::process::exit(1);
    }
    
    // Run Nix installer with daemon mode
    if config.verbose {
        println!("Running Nix installer (daemon mode)...");
    }
    let status = Command::new("sh")
        .args(&["./nix-install.sh", "--daemon"])
        .current_dir(&home)
        .status()
        .expect("Failed to execute Nix installer");
    
    if !status.success() {
        eprintln!("Failed to install Nix");
        std::process::exit(1);
    }
    
    println!("✓ Nix installed successfully!");
}
// Enable Nix daemon and setup home-manager
fn setup_home_manager(config: &Config) {
    println!("Setting up Home Manager...");
    
    if config.dry_run {
        println!("[DRY RUN] Would execute:");
        println!("  1. sudo systemctl enable --now nix-daemon.service");
        println!("  2. nix-channel --add https://github.com/nix-community/home-manager/archive/master.tar.gz home-manager");
        println!("  3. nix-channel --update");
        println!("  4. nix-shell '<home-manager>' -A install");
        return;
    }
    
    // Enable and start Nix daemon service
    if config.verbose {
        println!("Enabling Nix daemon service...");
    }
    let status = Command::new("sudo")
        .args(&["systemctl", "enable", "--now", "nix-daemon.service"])
        .status()
        .expect("Failed to execute systemctl");
    
    if !status.success() {
        eprintln!("Failed to enable Nix daemon service");
        std::process::exit(1);
    }
    
    // Add home-manager channel
    if config.verbose {
        println!("Adding home-manager channel...");
    }
    let status = Command::new("nix-channel")
        .args(&[
            "--add",
            "https://github.com/nix-community/home-manager/archive/master.tar.gz",
            "home-manager"
        ])
        .status()
        .expect("Failed to execute nix-channel add");
    
    if !status.success() {
        eprintln!("Failed to add home-manager channel");
        std::process::exit(1);
    }
    
    // Update channels
    if config.verbose {
        println!("Updating nix channels...");
    }
    let status = Command::new("nix-channel")
        .arg("--update")
        .status()
        .expect("Failed to execute nix-channel update");
    
    if !status.success() {
        eprintln!("Failed to update nix channels");
        std::process::exit(1);
    }
    
    // Install home-manager
    if config.verbose {
        println!("Installing home-manager...");
    }
    let status = Command::new("nix-shell")
        .args(&["<home-manager>", "-A", "install"])
        .status()
        .expect("Failed to execute nix-shell");
    
    if !status.success() {
        eprintln!("Failed to install home-manager");
        std::process::exit(1);
    }
    
    println!("✓ Home Manager setup complete!");
}

// Clone wallpaper repositories
fn clone_wallpapers(config: &Config) {
    println!("Cloning wallpaper repositories...");
    
    let wallpaper_repos = vec![
        "https://github.com/rann01/IRIX-tiles",
        "https://github.com/dharmx/walls",
        "https://github.com/wallace-aph/tiles-and-such",
        "https://github.com/tile-anon/tiles",
        "https://github.com/whoisYoges/lwalpapers",
        "https://github.com/D3Ext/aesthetic-wallpapers",
        "https://github.com/peteroupc/classic-wallpaper",
        "https://github.com/dixiedream/wallpapers",
        "https://github.com/mylinuxforwork/wallpaper",
        "https://github.com/makccr/wallpapers",
        "https://github.com/Axenide/Wallpapers",
        "https://github.com/l3ct3r/wallpapers",
        "https://github.com/dmighty007/WallPapers",
        "https://github.com/DenverCoder1/minimalistic-wallpaper-collection",
        "https://github.com/BitterSweetcandyshop/wallpapers",
        "https://github.com/linuxdotexe/nordic-wallpapers",
    ];
    
    if config.dry_run {
        println!("[DRY RUN] Would clone {} wallpaper repositories to ~/ with --depth=1", wallpaper_repos.len());
        for repo in &wallpaper_repos {
            println!("  - {}", repo);
        }
        return;
    }
    
    let home = env::var("HOME").expect("HOME environment variable not set");
    
    for repo in &wallpaper_repos {
        // Extract repo name from URL
        let repo_name = repo.split('/').last().unwrap_or("");
        let repo_path = format!("{}/{}", home, repo_name);
        
        // Check if repo already exists
        if Path::new(&repo_path).exists() {
            if config.verbose {
                println!("✓ {} already exists, skipping", repo_name);
            }
            continue;
        }
        
        if config.verbose {
            println!("Cloning {}...", repo);
        }
        
        let status = Command::new("git")
            .args(&["clone", "--depth=1", repo])
            .current_dir(&home)
            .status()
            .expect("Failed to execute git clone");
        
        if !status.success() {
            eprintln!("⚠ Warning: Failed to clone {}", repo);
            // Continue with other repos instead of exiting
        } else if config.verbose {
            println!("✓ Cloned {}", repo);
        }
    }
    
    println!("✓ Wallpaper repositories cloned!");
}

// Rebuild home-manager configuration
fn rebuild_home_manager(config: &Config) {
    println!("Rebuilding Home Manager configuration...");
    
    if config.dry_run {
        println!("[DRY RUN] Would execute:");
        println!("  home-manager switch -b backup");
        return;
    }
    
    if config.verbose {
        println!("Running home-manager switch...");
    }
    
    let status = Command::new("home-manager")
        .args(&["switch", "-b", "backup"])
        .status()
        .expect("Failed to execute home-manager");
    
    if !status.success() {
        eprintln!("Failed to rebuild home-manager configuration");
        std::process::exit(1);
    }
    
    println!("✓ Home Manager configuration rebuilt successfully!");
}

// Setup Chaotic AUR repository
fn setup_chaotic_aur(config: &Config) {
    println!("Setting up Chaotic AUR...");
    
    if config.dry_run {
        println!("[DRY RUN] Would execute:");
        println!("  1. Check if Chaotic AUR is already configured");
        println!("  2. sudo pacman-key --recv-key 3056513887B78AEB --keyserver keyserver.ubuntu.com");
        println!("  3. sudo pacman-key --lsign-key 3056513887B78AEB");
        println!("  4. sudo pacman -U --noconfirm 'https://cdn-mirror.chaotic.cx/chaotic-aur/chaotic-keyring.pkg.tar.zst'");
        println!("  5. sudo pacman -U --noconfirm 'https://cdn-mirror.chaotic.cx/chaotic-aur/chaotic-mirrorlist.pkg.tar.zst'");
        println!("  6. Append chaotic-aur config to /etc/pacman.conf");
        println!("  7. sudo pacman -Syu --noconfirm");
        return;
    }
    
    // Check if Chaotic AUR is already configured
    let pacman_conf = std::fs::read_to_string("/etc/pacman.conf")
        .unwrap_or_default();
    
    if pacman_conf.contains("[chaotic-aur]") {
        if config.verbose {
            println!("✓ Chaotic AUR already configured");
        } else {
            println!("✓ Chaotic AUR already configured, skipping setup");
        }
        return;
    }
    
    // Receive GPG key
    if config.verbose {
        println!("Receiving Chaotic AUR GPG key...");
    }
    let status = Command::new("sudo")
        .args(&["pacman-key", "--recv-key", "3056513887B78AEB", "--keyserver", "keyserver.ubuntu.com"])
        .status()
        .expect("Failed to execute pacman-key recv");
    
    if !status.success() {
        eprintln!("Failed to receive Chaotic AUR GPG key");
        std::process::exit(1);
    }
    
    // Locally sign the key
    if config.verbose {
        println!("Signing Chaotic AUR GPG key...");
    }
    let status = Command::new("sudo")
        .args(&["pacman-key", "--lsign-key", "3056513887B78AEB"])
        .status()
        .expect("Failed to execute pacman-key lsign");
    
    if !status.success() {
        eprintln!("Failed to sign Chaotic AUR GPG key");
        std::process::exit(1);
    }
    
    // Install chaotic-keyring
    if config.verbose {
        println!("Installing chaotic-keyring...");
    }
    let status = Command::new("sudo")
        .args(&["pacman", "-U", "--noconfirm", "https://cdn-mirror.chaotic.cx/chaotic-aur/chaotic-keyring.pkg.tar.zst"])
        .status()
        .expect("Failed to execute pacman");
    
    if !status.success() {
        eprintln!("Failed to install chaotic-keyring");
        std::process::exit(1);
    }
    
    // Install chaotic-mirrorlist
    if config.verbose {
        println!("Installing chaotic-mirrorlist...");
    }
    let status = Command::new("sudo")
        .args(&["pacman", "-U", "--noconfirm", "https://cdn-mirror.chaotic.cx/chaotic-aur/chaotic-mirrorlist.pkg.tar.zst"])
        .status()
        .expect("Failed to execute pacman");
    
    if !status.success() {
        eprintln!("Failed to install chaotic-mirrorlist");
        std::process::exit(1);
    }
    
    // Append to /etc/pacman.conf
    if config.verbose {
        println!("Adding Chaotic AUR to pacman.conf...");
    }
    
    // Remove temp file if it exists
    let _ = std::fs::remove_file("/tmp/chaotic-aur.conf");
    
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("/tmp/chaotic-aur.conf")
        .expect("Failed to create temp file");
    
    writeln!(file, "\n[chaotic-aur]").expect("Failed to write");
    writeln!(file, "Include = /etc/pacman.d/chaotic-mirrorlist").expect("Failed to write");
    
    let status = Command::new("sudo")
        .args(&["tee", "-a", "/etc/pacman.conf"])
        .stdin(std::fs::File::open("/tmp/chaotic-aur.conf").expect("Failed to open temp file"))
        .stdout(std::process::Stdio::null())
        .status()
        .expect("Failed to append to pacman.conf");
    
    if !status.success() {
        eprintln!("Failed to update pacman.conf");
        std::process::exit(1);
    }
    
    // Clean up temp file
    let _ = std::fs::remove_file("/tmp/chaotic-aur.conf");
    
    // Update system
    if config.verbose {
        println!("Updating system with Chaotic AUR...");
    }
    let status = Command::new("sudo")
        .args(&["pacman", "-Syu", "--noconfirm"])
        .status()
        .expect("Failed to execute pacman");
    
    if !status.success() {
        eprintln!("Failed to update system");
        std::process::exit(1);
    }
    
    println!("✓ Chaotic AUR setup complete!");
}

fn main() {
    let config = parse_args();
    
    if config.dry_run {
        println!("=== DRY RUN MODE ===");
        println!("No actual changes will be made\n");
    }
    
    println!("A.S.S. - Arch Setup Script");
    check_deps(&config);
    install_paru(&config);
    setup_chaotic_aur(&config);
    setup_dotfiles(&config);
    deploy_dotfiles(&config);           // Just install stow and create dirs
    install_nix(&config);
    setup_home_manager(&config);        // This builds the initial default generation
    stow_custom_configs(&config);       // NOW we replace with your custom configs
    clone_wallpapers(&config);
    rebuild_home_manager(&config);      // Rebuild with your custom configs
    
    if config.dry_run {
        println!("\n=== DRY RUN COMPLETE ===");
    } else {
        println!("\n✓ Setup complete! Your system is ready to use!");
    }
}