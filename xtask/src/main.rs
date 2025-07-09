use std::process::Command;
use std::path::Path;
use std::fs;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build documentation
    Docs {
        /// Output directory for documentation
        #[arg(short, long, default_value = "docs/build")]
        output: String,
        /// Build format (html, pdf, all)
        #[arg(short, long, default_value = "html")]
        format: String,
        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Preview documentation with live reload
    PreviewDocs {
        /// Enable browser auto-open
        #[arg(long)]
        open_browser: bool,
        /// Port for preview server
        #[arg(short, long, default_value = "8000")]
        port: u16,
    },
    /// Publish documentation to hosting service
    PublishDocs {
        /// Output directory for documentation
        #[arg(short, long, default_value = "docs_output")]
        output_dir: String,
        /// Skip build and use existing output
        #[arg(long)]
        skip_build: bool,
    },
    /// Validate documentation
    ValidateDocs {
        /// Check for broken links
        #[arg(long)]
        check_links: bool,
        /// Validate requirements coverage
        #[arg(long)]
        check_requirements: bool,
        /// Lint documentation files
        #[arg(long)]
        lint: bool,
    },
    /// Generate API documentation
    ApiDocs {
        /// Include private items
        #[arg(long)]
        include_private: bool,
        /// Generate documentation for dependencies
        #[arg(long)]
        include_deps: bool,
    },
    /// Clean documentation build artifacts
    CleanDocs {
        /// Remove all build artifacts
        #[arg(long)]
        all: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Docs { output, format, verbose } => {
            build_docs(output, format, *verbose);
        }
        Commands::PreviewDocs { open_browser, port } => {
            preview_docs(*open_browser, *port);
        }
        Commands::PublishDocs { output_dir, skip_build } => {
            publish_docs(output_dir, *skip_build);
        }
        Commands::ValidateDocs { check_links, check_requirements, lint } => {
            validate_docs(*check_links, *check_requirements, *lint);
        }
        Commands::ApiDocs { include_private, include_deps } => {
            generate_api_docs(*include_private, *include_deps);
        }
        Commands::CleanDocs { all } => {
            clean_docs(*all);
        }
    }
}

fn build_docs(output: &str, format: &str, verbose: bool) {
    println!("Building documentation...");
    
    let docs_dir = Path::new("docs");
    if !docs_dir.exists() {
        eprintln!("Error: docs directory not found");
        std::process::exit(1);
    }

    // Install dependencies
    install_dependencies();

    // Build based on format
    match format {
        "html" => build_html_docs(output, verbose),
        "pdf" => build_pdf_docs(output, verbose),
        "all" => {
            build_html_docs(output, verbose);
            build_pdf_docs(output, verbose);
        }
        _ => {
            eprintln!("Error: Unsupported format '{}'. Use 'html', 'pdf', or 'all'", format);
            std::process::exit(1);
        }
    }

    println!("Documentation built successfully in {}", output);
}

fn install_dependencies() {
    println!("Installing documentation dependencies...");
    
    let status = Command::new("pip")
        .args(&["install", "-r", "docs/requirements.txt"])
        .status()
        .expect("Failed to install dependencies");

    if !status.success() {
        eprintln!("Error: Failed to install dependencies");
        std::process::exit(1);
    }
}

fn build_html_docs(output: &str, verbose: bool) {
    println!("Building HTML documentation...");
    
    let mut cmd = Command::new("sphinx-build");
    cmd.args(&["-M", "html", "docs/source", output]);
    
    if verbose {
        cmd.arg("-v");
    }
    
    let status = cmd.status().expect("Failed to run sphinx-build");
    
    if !status.success() {
        eprintln!("Error: HTML documentation build failed");
        std::process::exit(1);
    }
}

fn build_pdf_docs(output: &str, verbose: bool) {
    println!("Building PDF documentation...");
    
    let mut cmd = Command::new("sphinx-build");
    cmd.args(&["-M", "latexpdf", "docs/source", output]);
    
    if verbose {
        cmd.arg("-v");
    }
    
    let status = cmd.status().expect("Failed to run sphinx-build");
    
    if !status.success() {
        eprintln!("Error: PDF documentation build failed");
        std::process::exit(1);
    }
}

fn preview_docs(open_browser: bool, port: u16) {
    println!("Starting documentation preview server on port {}...", port);
    
    // Build HTML docs first
    build_html_docs("docs/build", false);
    
    let mut cmd = Command::new("sphinx-autobuild");
    cmd.args(&["docs/source", "docs/build/html"]);
    cmd.args(&["--port", &port.to_string()]);
    
    if open_browser {
        cmd.arg("--open-browser");
    }
    
    let status = cmd.status().expect("Failed to run sphinx-autobuild");
    
    if !status.success() {
        eprintln!("Error: Documentation preview failed");
        std::process::exit(1);
    }
}

fn publish_docs(output_dir: &str, skip_build: bool) {
    println!("Publishing documentation to {}...", output_dir);
    
    if !skip_build {
        // Build all formats
        build_docs(output_dir, "all", false);
    }
    
    // Copy built documentation to output directory
    let build_dir = Path::new("docs/build");
    let output_path = Path::new(output_dir);
    
    if !build_dir.exists() {
        eprintln!("Error: Build directory does not exist. Run build first.");
        std::process::exit(1);
    }
    
    // Create output directory if it doesn't exist
    if !output_path.exists() {
        fs::create_dir_all(output_path).expect("Failed to create output directory");
    }
    
    // Copy HTML documentation
    let html_src = build_dir.join("html");
    let html_dst = output_path.join("html");
    
    if html_src.exists() {
        copy_dir_all(&html_src, &html_dst).expect("Failed to copy HTML documentation");
    }
    
    // Copy PDF documentation
    let pdf_src = build_dir.join("latex");
    let pdf_dst = output_path.join("pdf");
    
    if pdf_src.exists() {
        copy_dir_all(&pdf_src, &pdf_dst).expect("Failed to copy PDF documentation");
    }
    
    println!("Documentation published successfully!");
}

fn validate_docs(check_links: bool, check_requirements: bool, lint: bool) {
    println!("Validating documentation...");
    
    let mut all_passed = true;
    
    if lint {
        println!("Linting documentation files...");
        let status = Command::new("rstcheck")
            .args(&["--recursive", "docs/source"])
            .status()
            .expect("Failed to run rstcheck");
        
        if !status.success() {
            eprintln!("Error: Documentation linting failed");
            all_passed = false;
        }
        
        let status = Command::new("doc8")
            .arg("docs/source")
            .status()
            .expect("Failed to run doc8");
        
        if !status.success() {
            eprintln!("Error: Documentation style check failed");
            all_passed = false;
        }
    }
    
    if check_links {
        println!("Checking for broken links...");
        let status = Command::new("sphinx-build")
            .args(&["-M", "linkcheck", "docs/source", "docs/build"])
            .status()
            .expect("Failed to run linkcheck");
        
        if !status.success() {
            eprintln!("Error: Link check failed");
            all_passed = false;
        }
    }
    
    if check_requirements {
        println!("Checking requirements coverage...");
        let status = Command::new("sphinx-build")
            .args(&["-M", "coverage", "docs/source", "docs/build"])
            .status()
            .expect("Failed to run coverage check");
        
        if !status.success() {
            eprintln!("Error: Requirements coverage check failed");
            all_passed = false;
        }
    }
    
    if all_passed {
        println!("All documentation validation checks passed!");
    } else {
        std::process::exit(1);
    }
}

fn generate_api_docs(include_private: bool, include_deps: bool) {
    println!("Generating API documentation...");
    
    let mut cmd = Command::new("cargo");
    cmd.args(&["doc", "--workspace"]);
    
    if include_private {
        cmd.arg("--document-private-items");
    }
    
    if include_deps {
        cmd.arg("--no-deps");
    } else {
        cmd.arg("--no-deps");
    }
    
    cmd.args(&["--open"]);
    
    let status = cmd.status().expect("Failed to run cargo doc");
    
    if !status.success() {
        eprintln!("Error: API documentation generation failed");
        std::process::exit(1);
    }
    
    println!("API documentation generated successfully!");
}

fn clean_docs(all: bool) {
    println!("Cleaning documentation build artifacts...");
    
    let build_dir = Path::new("docs/build");
    if build_dir.exists() {
        fs::remove_dir_all(build_dir).expect("Failed to remove build directory");
    }
    
    if all {
        // Clean cargo documentation
        let status = Command::new("cargo")
            .args(&["clean", "--doc"])
            .status()
            .expect("Failed to run cargo clean");
        
        if !status.success() {
            eprintln!("Warning: Failed to clean cargo documentation");
        }
        
        // Clean Python cache
        let pycache_dirs = find_pycache_dirs("docs");
        for dir in pycache_dirs {
            if let Err(e) = fs::remove_dir_all(&dir) {
                eprintln!("Warning: Failed to remove {}: {}", dir.display(), e);
            }
        }
    }
    
    println!("Documentation cleanup completed!");
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    
    Ok(())
}

fn find_pycache_dirs(root: &str) -> Vec<std::path::PathBuf> {
    let mut dirs = Vec::new();
    
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_dir() {
                if path.file_name().unwrap_or_default() == "__pycache__" {
                    dirs.push(path);
                } else {
                    dirs.extend(find_pycache_dirs(&path.to_string_lossy()));
                }
            }
        }
    }
    
    dirs
}