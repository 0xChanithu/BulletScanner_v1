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
struct Args {
    target: String,

    #[arg(long, default_value_t = 1)]
    start: u16,

    #[arg(long, default_value_t = 10000)]
    end: u16,

    #[arg(long, default_value_t = 500)]
    concurrency: usize,

    #[arg(long, default_value_t = 700)]
    timeout_ms: u64,

    #[arg(long, default_value_t = 1)]
    retries: u8,
}

// ─────────────────────────────────────────────
// Banner
// ─────────────────────────────────────────────

fn print_banner() {
     println!("\x1b[32m");
    println!(r"██████╗ ██╗   ██╗██╗     ██╗     ███████╗████████╗███████╗ ██████╗ █████╗ ███╗   ██╗");
    println!(r"██╔══██╗██║   ██║██║     ██║     ██╔════╝╚══██╔══╝██╔════╝██╔════╝██╔══██╗████╗  ██║");
    println!(r"██████╔╝██║   ██║██║     ██║     █████╗     ██║   ███████╗██║     ███████║██╔██╗ ██║");
    println!(r"██╔══██╗██║   ██║██║     ██║     ██╔══╝     ██║   ╚════██║██║     ██╔══██║██║╚██╗██║");
    println!(r"██████╔╝╚██████╔╝███████╗███████╗███████╗   ██║   ███████║╚██████╗██║  ██║██║ ╚████║");
    println!(r"╚═════╝  ╚═════╝ ╚══════╝╚══════╝╚══════╝   ╚═╝   ╚══════╝ ╚═════╝╚═╝  ╚═╝╚═╝  ╚═══╝");
    println!("\x1b[0m");

    println!("\x1b[32mBulletScanner_v1\x1b[0m");
    println!("\x1b[90mFast TCP Port Scanner written in Rust\x1b[0m");
    println!();
    println!("\x1b[32m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m");
    println!("\x1b[36m  GitHub  :\x1b[0m https://github.com/0xChanithu/BulletScanner_v1.git");
    println!("\x1b[36m  Engine  :\x1b[0m Rust Async TCP Scanner");
    println!("\x1b[36m  Purpose :\x1b[0m Fast reconnaissance & port discovery");
    println!("\x1b[32m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m");
    println!();
}

// ─────────────────────────────────────────────
// Port probe
// ─────────────────────────────────────────────

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

// ─────────────────────────────────────────────
// Main
// ─────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    print_banner();

    let args = Args::parse();

    if args.start == 0 || args.end == 0 || args.start > args.end {
        anyhow::bail!("Invalid port range");
    }

    let ip: IpAddr = args
        .target
        .parse()
        .with_context(|| "Target must be an IP address")?;

    println!("\x1b[36m[+]\x1b[0m Target      : {}", args.target);
    println!(
        "\x1b[36m[+]\x1b[0m Port range  : {}-{}",
        args.start, args.end
    );
    println!("\x1b[36m[+]\x1b[0m Concurrency : {}", args.concurrency);
    println!("\x1b[36m[+]\x1b[0m Timeout     : {} ms", args.timeout_ms);
    println!("\x1b[36m[+]\x1b[0m Retries     : {}", args.retries);
    println!();

    println!("\x1b[33m[*] Starting fast TCP scan...\x1b[0m");
    println!();

    let semaphore = Arc::new(
        Semaphore::new(args.concurrency.max(1))
    );

    let mut jobs = Vec::new();

    for port in args.start..=args.end {
        let semaphore = semaphore.clone();
        let timeout_ms = args.timeout_ms;
        let retries = args.retries;

        jobs.push(tokio::spawn(async move {
            let _permit = semaphore.acquire_owned().await.ok()?;

            if probe(ip, port, timeout_ms).await {
                return Some(port);
            }

            for _ in 0..retries {
                if probe(ip, port, timeout_ms).await {
                    return Some(port);
                }
            }

            None
        }));
    }

    let mut open_ports = Vec::new();

    for job in jobs {
        if let Ok(Some(port)) = job.await {
            open_ports.push(port);
        }
    }

    open_ports.sort_unstable();
    open_ports.dedup();

    println!("\x1b[32m[+] Scan complete!\x1b[0m");
    println!();

    if open_ports.is_empty() {
        println!("\x1b[31m[-] No open ports found.\x1b[0m");
    } else {
        println!("\x1b[32mOPEN PORTS\x1b[0m");
        println!("\x1b[90m────────────────────\x1b[0m");

        for port in open_ports {
            println!("\x1b[32m[OPEN]\x1b[0m {}", port);
        }
    }

    println!();

    Ok(())
}