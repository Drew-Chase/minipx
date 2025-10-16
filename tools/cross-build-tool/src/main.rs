use anyhow::{bail, Context, Result};
use clap::Parser;
use colored::*;
use futures::future::join_all;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::Mutex;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

#[derive(Parser, Debug)]
#[command(
    name = "cross-build-tool",
    about = "Cross-compilation tool for minipx",
    long_about = "A tool for cross-compiling minipx to different platforms using Docker and the cross tool.\n\n\
                  Build Variants:\n\
                  - cli: Pure reverse proxy CLI without web interface\n\
                  - cli-webui: CLI with embedded web management interface\n\
                  - web: Standalone web management server\n\
                  - all: Build all three variants (default)\n\n\
                  Supported Targets:\n\
                  - x86_64-unknown-linux-gnu (Linux x64)\n\
                  - aarch64-unknown-linux-gnu (Linux ARM64)\n\
                  - x86_64-apple-darwin (macOS Intel)\n\
                  - aarch64-apple-darwin (macOS Apple Silicon)\n\
                  - x86_64-pc-windows-msvc (Windows x64)\n\
                  - all: Build for all platforms"
)]
struct Args {
    /// Target platform(s) (e.g., aarch64-unknown-linux-gnu, x86_64-unknown-linux-gnu, or "all" for all platforms)
    /// Can specify multiple targets: --target x86_64-unknown-linux-gnu --target aarch64-unknown-linux-gnu
    #[arg(short, long, default_value = "all", num_args = 1..)]
    target: Vec<String>,

    /// Build variant(s): cli, cli-webui, web, or all
    /// Can specify multiple variants: --variant cli --variant web
    #[arg(short = 'v', long, default_value = "all", num_args = 1..)]
    variant: Vec<Variant>,

    /// Clean build artifacts before building
    #[arg(short, long)]
    clean: bool,

    /// Create zip archives in target/dist after building
    #[arg(short, long)]
    archive: bool,

    /// Verbose output
    #[arg(long)]
    verbose: bool,

    /// Run builds in parallel (faster but uses more resources)
    #[arg(long)]
    parallel: bool,
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum Variant {
    Cli,
    CliWebui,
    Web,
    All,
}

impl Variant {
    fn as_str(&self) -> &str {
        match self {
            Variant::Cli => "cli",
            Variant::CliWebui => "cli-webui",
            Variant::Web => "web",
            Variant::All => "all",
        }
    }
}

const ALL_TARGETS: &[&str] =
    &[
        "x86_64-unknown-linux-gnu",
        "aarch64-unknown-linux-gnu",
//        "x86_64-apple-darwin",
//        "aarch64-apple-darwin",
        "x86_64-pc-windows-msvc"
    ];

#[derive(Debug, Clone)]
struct BuildResult {
    #[allow(dead_code)]
    target: String,
    #[allow(dead_code)]
    variant: String,
    success: bool,
    binaries: Vec<BuiltBinary>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Expand "all" in targets
    let mut targets = Vec::new();
    for target in &args.target {
        if target.to_lowercase() == "all" {
            targets.extend(ALL_TARGETS.iter().map(|s| s.to_string()));
        } else {
            targets.push(target.clone());
        }
    }
    // Remove duplicates
    targets.sort();
    targets.dedup();

    println!("{}", "=".repeat(60).cyan());
    println!("{}", "Minipx Cross-Compilation Tool".cyan().bold());
    println!("{}", "=".repeat(60).cyan());
    println!();

    // Pre-flight checks
    check_docker().await?;
    check_cross().await?;

    for target in &targets {
        check_toolchain(target).await?;
        check_target(target).await?;
    }

    if args.clean {
        clean_build().await?;
    }

    println!();
    if args.parallel {
        println!("{} Starting parallel builds...", "[BUILD]".cyan().bold());
    } else {
        println!("{} Starting sequential builds...", "[BUILD]".cyan().bold());
    }
    println!();

    // Setup progress bars
    let multi_progress = Arc::new(MultiProgress::new());
    let results = Arc::new(Mutex::new(Vec::new()));

    // Expand "all" in variants
    let mut variants = Vec::new();
    for variant in &args.variant {
        match variant {
            Variant::All => {
                variants.push(Variant::Cli);
                variants.push(Variant::CliWebui);
                variants.push(Variant::Web);
            }
            v => variants.push(v.clone()),
        }
    }
    // Remove duplicates
    variants.sort_by_key(|v| v.as_str().to_string());
    variants.dedup_by_key(|v| v.as_str().to_string());

    if args.parallel {
        // Parallel execution: spawn all tasks and wait for them
        let mut tasks = Vec::new();

        for target in &targets {
            for variant in &variants {
                let target = target.to_string();
                let variant = variant.clone();
                let mp = Arc::clone(&multi_progress);
                let results = Arc::clone(&results);

                let task = tokio::spawn(async move {
                    let pb = mp.add(ProgressBar::new_spinner());
                    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.cyan} {msg}").unwrap());

                    let variant_str = variant.as_str();
                    let display_name = format!("{} ({})", target, variant_str);
                    pb.set_message(display_name.clone());
                    pb.enable_steady_tick(std::time::Duration::from_millis(100));

                    let build_result = build_target_variant(&target, &variant).await;

                    match &build_result {
                        Ok(binaries) => {
                            pb.finish_with_message(format!("{} {}", "✓".green(), display_name));
                            results.lock().await.push(BuildResult {
                                target: target.clone(),
                                variant: variant_str.to_string(),
                                success: true,
                                binaries: binaries.clone(),
                            });
                        }
                        Err(e) => {
                            pb.finish_with_message(format!("{} {} - {}", "✗".red(), display_name, e));
                            results.lock().await.push(BuildResult {
                                target: target.clone(),
                                variant: variant_str.to_string(),
                                success: false,
                                binaries: Vec::new(),
                            });
                        }
                    }

                    build_result
                });

                tasks.push(task);
            }
        }

        // Wait for all tasks to complete, or handle Ctrl+C
        let build_future = join_all(tasks);

        tokio::select! {
            _ = build_future => {
                // Builds completed normally
            }
            _ = tokio::signal::ctrl_c() => {
                println!();
                println!();
                println!("{} Build cancelled - cleaning up Docker containers...", "[CANCEL]".yellow().bold());

                // Stop all running Docker containers started by cross
                if let Ok(output) = Command::new("docker")
                    .args(["ps", "-a", "--filter", "label=cross", "-q"])
                    .output()
                    .await
                {
                    let container_ids = String::from_utf8_lossy(&output.stdout);
                    for container_id in container_ids.lines().filter(|line| !line.is_empty()) {
                        let _ = Command::new("docker")
                            .args(["stop", container_id])
                            .output()
                            .await;
                        let _ = Command::new("docker")
                            .args(["rm", container_id])
                            .output()
                            .await;
                    }
                }

                println!("{} Cleanup complete", "[DONE]".green().bold());
                std::process::exit(130); // Standard exit code for SIGINT
            }
        }
    } else {
        // Sequential execution: build one at a time
        for target in &targets {
            for variant in &variants {
                let pb = multi_progress.add(ProgressBar::new_spinner());
                pb.set_style(ProgressStyle::default_spinner().template("{spinner:.cyan} {msg}")?);

                let variant_str = variant.as_str();
                let display_name = format!("{} ({})", target, variant_str);
                pb.set_message(display_name.clone());
                pb.enable_steady_tick(std::time::Duration::from_millis(100));

                let build_result = build_target_variant(target, variant).await;

                match &build_result {
                    Ok(binaries) => {
                        pb.finish_with_message(format!("{} {}", "✓".green(), display_name));
                        results.lock().await.push(BuildResult {
                            target: target.to_string(),
                            variant: variant_str.to_string(),
                            success: true,
                            binaries: binaries.clone(),
                        });
                    }
                    Err(e) => {
                        pb.finish_with_message(format!("{} {} - {}", "✗".red(), display_name, e));
                        results.lock().await.push(BuildResult {
                            target: target.to_string(),
                            variant: variant_str.to_string(),
                            success: false,
                            binaries: Vec::new(),
                        });
                    }
                }
            }
        }
    }

    println!();
    println!("{}", "=".repeat(60).bright_black());
    println!();

    // Print summary
    let results = results.lock().await;
    let successful = results.iter().filter(|r| r.success).count();
    let failed = results.iter().filter(|r| !r.success).count();

    if failed == 0 {
        println!("{} All builds completed successfully!", "[COMPLETE]".green().bold());
        println!("{}  Built {} target(s) with {} variant(s)", " ".repeat(11), targets.len(), variants.len());
    } else {
        println!("{} Builds completed with {} successes and {} failures", "[COMPLETE]".yellow().bold(), successful, failed);
    }

    // Handle archiving
    if args.archive && !results.is_empty() {
        println!();
        let all_binaries: Vec<BuildResult> = results.iter().filter(|r| r.success).cloned().collect();

        if !all_binaries.is_empty() {
            archive_all_binaries(&all_binaries).await?;
        }
    }

    println!();
    if args.archive {
        println!("{}  Binaries: target/<target>/release/", " ".repeat(6));
        println!("{}  Archives: {}", " ".repeat(6), "target/dist/".cyan());
    } else {
        println!("{}  Binaries: target/<target>/release/", " ".repeat(6));
    }

    if failed > 0 {
        bail!("{} build(s) failed", failed);
    }

    Ok(())
}

async fn check_docker() -> Result<()> {
    print!("{} Checking Docker... ", "[CHECK]".blue().bold());
    std::io::stdout().flush().ok();

    let output = Command::new("docker")
        .arg("ps")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .context("Failed to run docker command. Is Docker installed?")?;

    if !output.success() {
        println!("{}", "[✗]".red().bold());
        bail!("Docker is not running. Please start Docker Desktop.");
    }

    println!("{}", "[✓]".green().bold());
    Ok(())
}

async fn check_cross() -> Result<()> {
    print!("{} Checking cross... ", "[CHECK]".blue().bold());
    std::io::stdout().flush().ok();

    let output = Command::new("cross").arg("--version").stdout(Stdio::null()).stderr(Stdio::null()).status().await;

    match output {
        Ok(status) if status.success() => {
            println!("{}", "[✓]".green().bold());
            Ok(())
        }
        _ => {
            println!("{}", "[✗]".red().bold());
            bail!(
                "cross is not installed.\n\n\
                 Install it with:\n  \
                 cargo install cross --git https://github.com/cross-rs/cross"
            );
        }
    }
}

async fn check_toolchain(target: &str) -> Result<()> {
    print!("{} Checking rustup toolchain... ", "[CHECK]".blue().bold());
    std::io::stdout().flush().ok();

    // Check if stable toolchain is installed
    let output = Command::new("rustup").args(["toolchain", "list"]).output().await.context("Failed to run rustup command")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stable_installed = stdout.lines().any(|line| line.starts_with("stable") && line.contains(target));

    if !stable_installed {
        println!("{}", "[!]".yellow().bold());
        print!("{} Installing stable toolchain with minimal profile... ", "[INFO]".yellow().bold());
        std::io::stdout().flush().ok();

        let status = Command::new("rustup")
            .args(["toolchain", "install", format!("stable-{}", target).as_str(), "--profile", "minimal", "--force-non-host"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .context("Failed to install toolchain")?;

        if !status.success() {
            println!("{}", "[✗]".red().bold());
            bail!("Failed to install stable toolchain");
        }
        println!("{}", "[✓]".green().bold());
    } else {
        println!("{}", "[✓]".green().bold());
    }

    Ok(())
}

async fn check_target(target: &str) -> Result<()> {
    let output = Command::new("rustup").args(["target", "list"]).output().await.context("Failed to run rustup command")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let target_installed = stdout.lines().any(|line| line.contains(target) && line.contains("(installed)"));

    if !target_installed {
        print!("{} Adding target {}... ", "[INFO]".yellow().bold(), target);
        std::io::stdout().flush().ok();

        let status = Command::new("rustup")
            .args(["target", "add", target])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .context("Failed to add target")?;

        if !status.success() {
            println!("{}", "[✗]".red().bold());
            bail!("Failed to add target {}", target);
        }
        println!("{}", "[✓]".green().bold());
    }

    Ok(())
}

async fn clean_build() -> Result<()> {
    print!("{} Cleaning build artifacts... ", "[CLEAN]".yellow().bold());
    std::io::stdout().flush().ok();

    let status =
        Command::new("cargo").arg("clean").stdout(Stdio::null()).stderr(Stdio::null()).status().await.context("Failed to run cargo clean")?;

    if !status.success() {
        println!("{}", "[✗]".red().bold());
        bail!("Failed to clean build");
    }

    println!("{}", "[✓]".green().bold());
    Ok(())
}

#[derive(Debug, Clone)]
struct BuiltBinary {
    path: PathBuf,
    variant: String,
    target: String,
}

async fn build_target_variant(target: &str, variant: &Variant) -> Result<Vec<BuiltBinary>> {
    let mut binaries = Vec::new();

    // Create logs directory
    let logs_dir = Path::new("target/logs");
    fs::create_dir_all(logs_dir).context("Failed to create logs directory")?;
    let logs_dir_abs = logs_dir.canonicalize().unwrap_or_else(|_| logs_dir.to_path_buf());

    match variant {
        Variant::Cli => {
            let log_file_path = logs_dir.join(format!("minipx-cli-{}.log", target));
            let log_file_path_abs = logs_dir_abs.join(format!("minipx-cli-{}.log", target));
            let log_file = File::create(&log_file_path).context("Failed to create log file")?;
            let log_file_stderr = log_file.try_clone().context("Failed to clone log file handle")?;

            let status = Command::new("cross")
                .args(["build", "--release", "--target", target, "-p", "minipx_cli", "--features", "openssl/vendored"])
                .stdout(Stdio::from(log_file))
                .stderr(Stdio::from(log_file_stderr))
                .status()
                .await
                .context("Failed to run cross build")?;

            if !status.success() {
                bail!("{}", create_log_link(&log_file_path_abs));
            }

            let binary_name = if target.contains("windows") { "minipx.exe" } else { "minipx" };
            let binary_path = PathBuf::from(format!("target/{}/release/{}", target, binary_name));

            binaries.push(BuiltBinary { path: binary_path, variant: "cli".to_string(), target: target.to_string() });
        }
        Variant::CliWebui => {
            let log_file_path = logs_dir.join(format!("minipx-cli-webui-{}.log", target));
            let log_file_path_abs = logs_dir_abs.join(format!("minipx-cli-webui-{}.log", target));
            let log_file = File::create(&log_file_path).context("Failed to create log file")?;
            let log_file_stderr = log_file.try_clone().context("Failed to clone log file handle")?;

            let status = Command::new("cross")
                .args(["build", "--release", "--target", target, "-p", "minipx_cli", "--features", "webui openssl/vendored"])
                .stdout(Stdio::from(log_file))
                .stderr(Stdio::from(log_file_stderr))
                .status()
                .await
                .context("Failed to run cross build")?;

            if !status.success() {
                bail!("{}", create_log_link(&log_file_path_abs));
            }

            let binary_name = if target.contains("windows") { "minipx.exe" } else { "minipx" };
            let binary_path = PathBuf::from(format!("target/{}/release/{}", target, binary_name));

            binaries.push(BuiltBinary { path: binary_path, variant: "cli-webui".to_string(), target: target.to_string() });
        }
        Variant::Web => {
            let log_file_path = logs_dir.join(format!("minipx-web-{}.log", target));
            let log_file_path_abs = logs_dir_abs.join(format!("minipx-web-{}.log", target));
            let log_file = File::create(&log_file_path).context("Failed to create log file")?;
            let log_file_stderr = log_file.try_clone().context("Failed to clone log file handle")?;

            let status = Command::new("cross")
                .args(["build", "--release", "--target", target, "-p", "minipx_web", "--features", "openssl/vendored"])
                .stdout(Stdio::from(log_file))
                .stderr(Stdio::from(log_file_stderr))
                .status()
                .await
                .context("Failed to run cross build")?;

            if !status.success() {
                bail!("{}", create_log_link(&log_file_path_abs));
            }

            let binary_name = if target.contains("windows") { "minipx_web.exe" } else { "minipx_web" };
            let binary_path = PathBuf::from(format!("target/{}/release/{}", target, binary_name));

            binaries.push(BuiltBinary { path: binary_path, variant: "web".to_string(), target: target.to_string() });
        }
        Variant::All => {
            // This shouldn't happen as we split All into individual variants
            bail!("Variant::All should be split before calling build_target_variant");
        }
    }

    Ok(binaries)
}

async fn archive_all_binaries(build_results: &[BuildResult]) -> Result<()> {
    println!("{} Creating archives...", "[ARCHIVE]".cyan().bold());
    println!();

    let dist_dir = Path::new("target/dist");
    fs::create_dir_all(dist_dir).context("Failed to create target/dist directory")?;

    let multi_progress = Arc::new(MultiProgress::new());
    let mut tasks = Vec::new();

    for result in build_results {
        for binary in &result.binaries {
            let binary = binary.clone();
            let mp = Arc::clone(&multi_progress);

            let task = tokio::spawn(async move {
                let (os, arch) = match parse_target(&binary.target) {
                    Ok(parsed) => parsed,
                    Err(e) => return Err(e),
                };

                let archive_name = format!("minipx-{}-{}-{}.zip", binary.variant, os, arch);
                let archive_path = Path::new("target/dist").join(&archive_name);

                let pb = mp.add(ProgressBar::new_spinner());
                pb.set_style(ProgressStyle::default_spinner().template("{spinner:.cyan} {msg}").unwrap());
                pb.set_message(format!("Archiving {}", archive_name));
                pb.enable_steady_tick(std::time::Duration::from_millis(100));

                if !binary.path.exists() {
                    pb.finish_with_message(format!("{} {} - binary not found", "✗".red(), archive_name));
                    return Err(anyhow::anyhow!("Binary not found: {}", binary.path.display()));
                }

                // Create zip archive
                let file = match File::create(&archive_path) {
                    Ok(f) => f,
                    Err(e) => {
                        pb.finish_with_message(format!("{} {} - failed to create", "✗".red(), archive_name));
                        return Err(e.into());
                    }
                };
                let mut zip = ZipWriter::new(file);

                let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated).unix_permissions(0o755);

                let binary_name = binary.path.file_name().context("Failed to get binary filename")?.to_string_lossy();

                zip.start_file(binary_name.as_ref(), options).context("Failed to start file in zip")?;

                let binary_contents = fs::read(&binary.path).context(format!("Failed to read binary: {}", binary.path.display()))?;

                zip.write_all(&binary_contents).context("Failed to write binary to zip")?;

                zip.finish().context("Failed to finalize zip archive")?;

                pb.finish_with_message(format!("{} {}", "✓".green(), archive_name));
                Ok(())
            });

            tasks.push(task);
        }
    }

    let results: Vec<_> = join_all(tasks).await;

    let mut failed = 0;
    for result in results {
        if let Ok(Err(_)) = result {
            failed += 1;
        }
    }

    println!();
    if failed == 0 {
        println!("{} All archives created successfully", "[DONE]".green().bold());
    } else {
        println!("{} {} archive(s) failed", "[WARNING]".yellow().bold(), failed);
    }

    Ok(())
}

fn parse_target(target: &str) -> Result<(String, String)> {
    // Parse target triple like "aarch64-unknown-linux-gnu" or "x86_64-pc-windows-msvc"
    let parts: Vec<&str> = target.split('-').collect();

    let arch = match parts.first() {
        Some(&"x86_64") => "x64",
        Some(&"aarch64") => "arm64",
        Some(&"i686") => "x86",
        Some(&"armv7") => "armv7",
        Some(arch) => arch,
        None => bail!("Invalid target triple: {}", target),
    };

    let os = if target.contains("linux") {
        "linux"
    } else if target.contains("windows") {
        "windows"
    } else if target.contains("darwin") || target.contains("apple") {
        "macos"
    } else {
        bail!("Unsupported OS in target triple: {}", target)
    };

    Ok((os.to_string(), arch.to_string()))
}

/// Creates a clickable terminal hyperlink using OSC 8 escape codes
/// Returns a string like "Build failed - [open log]" where [open log] is clickable
fn create_log_link(log_path: &Path) -> String {
    // Clean up Windows extended path prefix (\\?\) if present
    let path_str = log_path.display().to_string();
    let clean_path = path_str.strip_prefix(r"\\?\").unwrap_or(&path_str);

    // Convert backslashes to forward slashes for file:// URLs
    let url_path = clean_path.replace('\\', "/");

    // Create OSC 8 hyperlink: \x1b]8;;file://path\x1b\\text\x1b]8;;\x1b\\
    let link = format!("\x1b]8;;file://{}\x1b\\{}\x1b]8;;\x1b\\", url_path, "[open log]".cyan().bold());

    format!("Build failed - {}", link)
}
