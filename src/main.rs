use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use regex::Regex;
use structopt::StructOpt;
use tokio::time::sleep;
use crossterm::{ExecutableCommand, terminal::{Clear, ClearType}, cursor::MoveTo};

#[derive(StructOpt)]
#[structopt(name = "Log Analyzer", about = "A tool to analyze Nginx access logs.")]
struct Cli {
    /// Path to the log file
    #[structopt(short, long, default_value = "access.log")]
    file: String,

    /// Mode of operation: 'new' to read new data from the end, 'all' to read the entire file
    #[structopt(short, long, default_value = "new")]
    mode: String,

    /// Regular expression to parse the log entries
    #[structopt(short, long, default_value = r#"^(\S+) - ".+" \[.*?\] \d+\.\d+ "\S+" "\S+ (\S+?)(?:\?.*?)? HTTP/.*""#)]
    regex: String,

    /// Number of top entries to display
    #[structopt(short, long, default_value = "10")]
    top: usize,
}

#[tokio::main]
async fn main() {
    let args = Cli::from_args();
    let file_path = args.file.clone();
    let mode = args.mode.clone();
    let regex_pattern = args.regex.clone();
    let top_n = args.top;

    let log_data = Arc::new(Mutex::new(LogData::new()));
    let log_data_clone = Arc::clone(&log_data);

    tokio::spawn(async move {
        loop {
            if let Err(e) = tail_file(&file_path, &mode, &regex_pattern, &log_data_clone).await {
                eprintln!("Error reading file: {:?}", e);
            }
            sleep(Duration::from_secs(1)).await;
        }
    });

    loop {
        print_stats(&log_data, top_n).await;
        sleep(Duration::from_secs(5)).await;
    }
}

struct LogData {
    by_ip: HashMap<String, usize>,
    by_url: HashMap<String, usize>,
    total_requests: usize,
}

impl LogData {
    fn new() -> Self {
        Self {
            by_ip: HashMap::new(),
            by_url: HashMap::new(),
            total_requests: 0,
        }
    }

    fn add_entry(&mut self, ip: String, url: String) {
        *self.by_ip.entry(ip).or_insert(0) += 1;
        *self.by_url.entry(url).or_insert(0) += 1;
        self.total_requests += 1;
    }

    fn get_top_n(&self, n: usize) -> (Vec<(String, usize)>, Vec<(String, usize)>) {
        let mut top_ip = self.by_ip.iter().collect::<Vec<_>>();
        let mut top_url = self.by_url.iter().collect::<Vec<_>>();

        top_ip.sort_by(|a, b| b.1.cmp(a.1));
        top_url.sort_by(|a, b| b.1.cmp(a.1));

        (
            top_ip.into_iter().take(n).map(|(k, &v)| (k.clone(), v)).collect(),
            top_url.into_iter().take(n).map(|(k, &v)| (k.clone(), v)).collect(),
        )
    }


    fn get_unique_counts(&self) -> (usize, usize) {
        (self.by_ip.len(), self.by_url.len())
    }
}

async fn tail_file(file_path: &str, mode: &str, regex_pattern: &str, log_data: &Arc<Mutex<LogData>>) -> std::io::Result<()> {
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);

    if mode == "new" {
        // Если режим "new", перемещаемся в конец файла
        reader.seek(SeekFrom::End(0))?;
    } else {
        // Если режим "all", перемещаемся в начало файла
        reader.seek(SeekFrom::Start(0))?;
    }

    let re = Regex::new(regex_pattern).unwrap();

    loop {
        let mut line = String::new();
        let len = reader.read_line(&mut line)?;

        if len == 0 {
            break;
        }

        // println!("Read line: {}", line);  // Отладочное сообщение

        if let Some(caps) = re.captures(&line) {
            let ip = caps.get(1).map_or("", |m| m.as_str()).to_string();
            let url = caps.get(2).map_or("", |m| m.as_str()).to_string();

            // println!("Captured IP: {}, URL: {}", ip, url);  // Отладочное сообщение

            let mut log_data = log_data.lock().unwrap();
            log_data.add_entry(ip, url);
        } else {
            println!("No match for line: {}", line);  // Отладочное сообщение
        }
    }

    Ok(())
}

async fn print_stats(log_data: &Arc<Mutex<LogData>>, top_n: usize) {
    let mut stdout = std::io::stdout();
    stdout.execute(Clear(ClearType::All)).unwrap();
    stdout.execute(MoveTo(0, 0)).unwrap();

    let log_data = log_data.lock().unwrap();
    let (top_ips, top_urls) = log_data.get_top_n(top_n);
    let (unique_ips, unique_urls) = log_data.get_unique_counts();

    println!("Total requests: {}", log_data.total_requests);
    println!("Unique IPs: {}", unique_ips);
    println!("Unique URLs: {}", unique_urls);

    println!("\nTop {} IPs:", top_n);
    for (ip, count) in top_ips {
        println!("{}: {}", ip, count);
    }

    println!("\nTop {} URLs:", top_n);
    for (url, count) in top_urls {
        println!("{}: {}", url, count);
    }
}
