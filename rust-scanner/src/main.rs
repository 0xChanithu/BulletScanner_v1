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

// ==============================
// Banner
// ==============================
fn print_banner() {
    println!("\x1b[32m");

    println!(
        r"██████╗ ██╗   ██╗██╗     ██╗     ███████╗████████╗███████╗ ██████╗ █████╗ ███╗   ██╗"
    );
    println!(
        r"██╔══██╗██║   ██║██║     ██║     ██╔════╝╚══██╔══╝██╔════╝██╔════╝██╔══██╗████╗  ██║"
    );
    println!(
        r"██████╔╝██║   ██║██║     ██║     █████╗     ██║   ███████╗██║     ███████║██╔██╗ ██║"
    );
    println!(
        r"██╔══██╗██║   ██║██║     ██║     ██╔══╝     ██║   ╚════██║██║     ██╔══██║██║╚██╗██║"
    );
    println!(
        r"██████╔╝╚██████╔╝███████╗███████╗███████╗   ██║   ███████║╚██████╗██║  ██║██║ ╚████║"
    );
    println!(
        r"╚═════╝  ╚═════╝ ╚══════╝╚══════╝╚══════╝   ╚═╝   ╚══════╝╚═════╝╚═╝  ╚═╝╚═╝  ╚═══╝"
    );

    println!("\x1b[0m");

    println!("\x1b[32mBulletScanner_v1\x1b[0m");
    println!("\x1b[90mFast TCP Port Scanner written in Rust\x1b[0m");
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

// ==============================
// TCP Probe
// ==============================
async fn probe(ip: IpAddr, p: u16, t: u64) -> bool {
    matches!(
        timeout(
            Duration::from_millis(t),
            TcpStream::connect(SocketAddr::new(ip, p))
        )
        .await,
        Ok(Ok(_))
    )
}

// ==============================
// Main
// ==============================
#[tokio::main]
async fn main() -> Result<()> {
    let a = Args::parse();

    // Display banner
    print_banner();

    // Validate port range
    if a.start == 0 || a.end == 0 || a.start > a.end {
        anyhow::bail!("Invalid port range");
    }

    // Parse target IP
    let ip: IpAddr = a
        .target
        .parse()
        .with_context(|| "Target must be an IP address")?;

    println!("\x1b[36m[*] Target      : {}\x1b[0m", ip);
    println!("\x1b[36m[*] Port range  : {}-{}\x1b[0m", a.start, a.end);
    println!("\x1b[36m[*] Concurrency : {}\x1b[0m", a.concurrency);
    println!();

    let sem = Arc::new(Semaphore::new(a.concurrency.max(1)));

    let mut jobs = Vec::new();

    for p in a.start..=a.end {
        let s = sem.clone();
        let t = a.timeout_ms;
        let r = a.retries;

        jobs.push(tokio::spawn(async move {
            let _permit = s.acquire_owned().await.ok()?;

            if probe(ip, p, t).await || (r > 0 && probe(ip, p, t).await) {
                Some(p)
            } else {
                None
            }
        }));
    }

    let mut open = Vec::new();

    for j in jobs {
        if let Ok(Some(p)) = j.await {
            open.push(p);
        }
    }

    open.sort_unstable();
    open.dedup();

    println!("\x1b[32m[+] Open ports:\x1b[0m");

    for p in open {
        println!("{p}");
    }

    println!();

    Ok(())
}