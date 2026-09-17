use anyhow::{Context, Result};
use clap::Parser;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::{
    net::TcpStream,
    sync::Semaphore,
    time::{timeout, Duration},
};

const BANNER: &str = r#"
 ____        _ _      _   ____
| __ ) _   _| | | ___| |_/ ___|  ___ __ _ _ __
|  _ \| | | | | |/ _ \ __\___ \ / __/ _` | '_ \
| |_) | |_| | | |  __/ |_ ___) | (_| (_| | | | |
|____/ \__,_|_|_|\___|\__|____/ \___\__,_|_| |_|

        Faster port scanning, written in Rust.
"#;

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

async fn probe(ip: IpAddr, p: u16, t: u64) -> bool {
    matches!(
        timeout(Duration::from_millis(t), TcpStream::connect(SocketAddr::new(ip, p))).await,
        Ok(Ok(_))
    )
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("{BANNER}");

    let a = Args::parse();

    if a.start == 0 || a.end == 0 || a.start > a.end {
        anyhow::bail!("Invalid port range");
    }

    let ip: IpAddr = a
        .target
        .parse()
        .with_context(|| "Target must be an IP address")?;

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

    for p in open {
        println!("{p}");
    }

    Ok(())
}