use anyhow::{Context, Result};
use clap::Parser;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::{
    net::TcpStream,
    sync::Semaphore,
    time::{timeout, Duration},
};

#[derive(Parser)]
#[command(
    name = "BulletScanner",
    version = "1.0",
    about = "Fast TCP Port Scanner written in Rust"
)]
struct Args {
    /// Target IPv4/IPv6 address
    target: String,

    /// Starting port
    #[arg(long, default_value_t = 1)]
    start: u16,

    /// Ending port
    #[arg(long, default_value_t = 10000)]
    end: u16,

    /// Maximum concurrent connections
    #[arg(long, default_value_t = 500)]
    concurrency: usize,

    /// Connection timeout in milliseconds
    #[arg(long, default_value_t = 700)]
    timeout_ms: u64,

    /// Number of additional retries
    #[arg(long, default_value_t = 1)]
    retries: u8,
}

// ============================================================
// Banner
// ============================================================

fn print_banner() {
    println!();

    println!("\x1b[32m");
    println!(r"██████╗ ██╗   ██╗██╗     ██╗     ███████╗████████╗");
    println!(r"██╔══██╗██║   ██║██║     ██║     ██╔════╝╚══██╔══╝");
    println!(r"██████╔╝██║   ██║██║     ██║     █████╗     ██║   ");
    println!(r"██╔══██╗██║   ██║██║     ██║     ██╔══╝     ██║   ");
    println!(r"██████╔╝╚██████╔╝███████╗███████╗███████╗   ██║   ");
    println!(r"╚═════╝  ╚═════╝ ╚══════╝╚══════╝╚══════╝   ╚═╝   ");
    println!("\x1b[0m");

    println!("\x1b[1;32m              BulletScanner_v1\x1b[0m");
    println!("\x1b[90m          Fast TCP Port Scanner\x1b[0m");
    println!();

    println!(
        "\x1b[32m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m"
    );

    println!(
        "\x1b[36m  GitHub  :\x1b[0m https://github.com/0xChanithu/BulletScanner_v1.git"
    );
    println!("\x1b[36m  Engine  :\x1b[0m Rust Async TCP Scanner");
    println!("\x1b[36m  Purpose :\x1b[0m Fast reconnaissance & port discovery");

    println!(
        "\x1b[32m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m"
    );

    println!();
}

// ============================================================
// TCP Probe
// ============================================================

async fn probe(ip: IpAddr, port: u16, timeout_ms: u64) -> bool {
    matches!(
        timeout(
            Duration::from_millis(timeout_ms),
            TcpStream::connect(SocketAddr::new(ip, port))
        )
        .await,
        Ok(Ok(_))
    )
}

// ============================================================
// Main
// ============================================================

#[tokio::main]
async fn main() -> Result<()> {
    // --------------------------------------------------------
    // IMPORTANT: Banner is displayed FIRST
    // --------------------------------------------------------
    print_banner();

    // Parse command-line arguments
    let args = Args::parse();

    // --------------------------------------------------------
    // Validate port range
    // --------------------------------------------------------

    if args.start == 0 || args.end == 0 || args.start > args.end {
        anyhow::bail!("Invalid port range");
    }

    // --------------------------------------------------------
    // Parse target
    // --------------------------------------------------------

    let ip: IpAddr = args
        .target
        .parse()
        .with_context(|| "Target must be an IP address")?;

    // --------------------------------------------------------
    // Scan information
    // --------------------------------------------------------

    println!("\x1b[36m[*] Target      : {}\x1b[0m", ip);
    println!(
        "\x1b[36m[*] Port range  : {}-{}\x1b[0m",
        args.start, args.end
    );
    println!(
        "\x1b[36m[*] Concurrency : {}\x1b[0m",
        args.concurrency
    );
    println!(
        "\x1b[36m[*] Timeout     : {} ms\x1b[0m",
        args.timeout_ms
    );
    println!(
        "\x1b[36m[*] Retries     : {}\x1b[0m",
        args.retries
    );

    println!();

    println!(
        "\x1b[33m[*] Scanning ports {}-{}...\x1b[0m",
        args.start, args.end
    );

    println!();

    // --------------------------------------------------------
    // Semaphore controls concurrent connections
    // --------------------------------------------------------

    let semaphore = Arc::new(Semaphore::new(args.concurrency.max(1)));

    let mut jobs = Vec::new();

    // --------------------------------------------------------
    // Create scanning jobs
    // --------------------------------------------------------

    for port in args.start..=args.end {
        let semaphore = semaphore.clone();
        let timeout_ms = args.timeout_ms;
        let retries = args.retries;

        jobs.push(tokio::spawn(async move {
            let _permit = semaphore.acquire_owned().await.ok()?;

            // Initial probe
            if probe(ip, port, timeout_ms).await {
                return Some(port);
            }

            // Retry failed connection
            for _ in 0..retries {
                if probe(ip, port, timeout_ms).await {
                    return Some(port);
                }
            }

            None
        }));
    }

    // --------------------------------------------------------
    // Collect results
    // --------------------------------------------------------

    let mut open_ports = Vec::new();

    for job in jobs {
        if let Ok(Some(port)) = job.await {
            open_ports.push(port);
        }
    }

    // --------------------------------------------------------
    // Sort and remove duplicates
    // --------------------------------------------------------

    open_ports.sort_unstable();
    open_ports.dedup();

    // --------------------------------------------------------
    // Results
    // --------------------------------------------------------

    println!(
        "\x1b[32m[+] Scan completed successfully.\x1b[0m"
    );

    println!();

    if open_ports.is_empty() {
        println!("\x1b[31m[-] No open TCP ports found.\x1b[0m");
    } else {
        println!("\x1b[32m[+] Open TCP ports:\x1b[0m");

        for port in open_ports {
            println!("\x1b[32m    {}\x1b[0m", port);
        }
    }

    println!();

    Ok(())
}