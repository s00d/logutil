use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use prettytable::{format, row, Attr, Cell, Row, Table};
use regex::Regex;
use structopt::StructOpt;
use tokio::time::sleep;

#[derive(StructOpt)]
#[structopt(name = "Log Analyzer", about = "A tool to analyze Nginx access logs.")]
struct Cli {
    /// Path to the log file
    #[structopt(short, long, default_value = "access.log")]
    file: String,

    /// Mode of operation: 'new' to read new data from the end, 'all' to read the entire file
    #[structopt(short, long, default_value = "new")]
    mode: String,

    /// Regular expression to parse the log entries or path to a file containing the regex
    #[structopt(
        short,
        long,
        default_value = r#"^(\S+) - ".+" \[.*?\] \d+\.\d+ "\S+" "\S+ (\S+?)(?:\?.*?)? HTTP/.*""#
    )]
    regex: String,

    /// Number of top entries to display
    #[structopt(short, long, default_value = "10")]
    top: usize,

    /// Disable clearing of outdated entries
    #[structopt(long)]
    no_clear: bool,
}

#[tokio::main]
async fn main() {
    let args = Cli::from_args();
    let file_path = args.file.clone();
    let mode = args.mode.clone();
    let regex_pattern = if Path::new(&args.regex).exists() {
        fs::read_to_string(&args.regex).expect("Could not read regex file")
    } else {
        args.regex.clone()
    };
    let top_n = args.top;
    let no_clear = args.no_clear;

    let log_data = Arc::new(Mutex::new(LogData::new()));
    let log_data_clone = Arc::clone(&log_data);

    tokio::spawn(async move {
        loop {
            if let Err(e) =
                tail_file(&file_path, &mode, &regex_pattern, &log_data_clone, no_clear).await
            {
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

struct LogEntry {
    count: usize,
    last_update: SystemTime,
}

struct LogData {
    by_ip: HashMap<String, LogEntry>,
    by_url: HashMap<String, LogEntry>,
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

    fn add_entry(&mut self, ip: String, url: String, no_clear: bool) {
        let now = SystemTime::now();

        {
            let entry = self.by_ip.entry(ip).or_insert(LogEntry {
                count: 0,
                last_update: now,
            });
            entry.count += 1;
            entry.last_update = now;
        }

        {
            let entry = self.by_url.entry(url).or_insert(LogEntry {
                count: 0,
                last_update: now,
            });
            entry.count += 1;
            entry.last_update = now;
        }

        self.total_requests += 1;

        if self.by_ip.len() > 10000 && !no_clear {
            self.clear_outdated_entries();
        }
    }

    fn clear_outdated_entries(&mut self) {
        let threshold = SystemTime::now() - Duration::from_secs(1200); // 20 минут
        self.by_ip.retain(|_, entry| entry.last_update >= threshold);
        self.by_url
            .retain(|_, entry| entry.last_update >= threshold);
    }

    fn get_top_n(&self, n: usize) -> (Vec<(String, usize)>, Vec<(String, usize)>) {
        let mut top_ip = self.by_ip.iter().collect::<Vec<_>>();
        let mut top_url = self.by_url.iter().collect::<Vec<_>>();

        top_ip.sort_by(|a, b| b.1.count.cmp(&a.1.count));
        top_url.sort_by(|a, b| b.1.count.cmp(&a.1.count));

        (
            top_ip
                .into_iter()
                .take(n)
                .map(|(k, v)| (k.clone(), v.count))
                .collect(),
            top_url
                .into_iter()
                .take(n)
                .map(|(k, v)| (k.clone(), v.count))
                .collect(),
        )
    }

    fn get_unique_counts(&self) -> (usize, usize) {
        (self.by_ip.len(), self.by_url.len())
    }
}

async fn tail_file(
    file_path: &str,
    mode: &str,
    regex_pattern: &str,
    log_data: &Arc<Mutex<LogData>>,
    no_clear: bool,
) -> std::io::Result<()> {
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

        if let Some(caps) = re.captures(&line) {
            let ip = caps.get(1).map_or("", |m| m.as_str()).to_string();
            let url = caps.get(2).map_or("", |m| m.as_str()).to_string();

            let mut log_data = log_data.lock().unwrap();
            log_data.add_entry(ip, url, no_clear);
        } else {
            println!("No match for line: {}", line);
        }
    }

    Ok(())
}

async fn print_stats(log_data: &Arc<Mutex<LogData>>, top_n: usize) {
    if cfg!(target_os = "windows") {
        Command::new("cls").status().unwrap();
    } else {
        Command::new("clear").status().unwrap();
    }

    let log_data = log_data.lock().unwrap();
    let (top_ips, top_urls) = log_data.get_top_n(top_n);
    let (unique_ips, unique_urls) = log_data.get_unique_counts();

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    table.add_row(Row::new(vec![
        Cell::new("Total requests").with_style(Attr::Bold),
        Cell::new(&log_data.total_requests.to_string()),
        Cell::new(""),
        Cell::new("Unique IPs").with_style(Attr::Bold),
        Cell::new(&unique_ips.to_string()),
        Cell::new(""),
        Cell::new("Unique URLs").with_style(Attr::Bold),
        Cell::new(&unique_urls.to_string()),
        Cell::new(""),
    ]));

    table.printstd();
    println!("");

    let mut ip_table = Table::new();
    ip_table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    ip_table.set_titles(Row::new(vec![
        Cell::new("Top IPs").with_style(Attr::Bold),
        Cell::new("Requests").with_style(Attr::Bold),
    ]));
    for (ip, count) in top_ips {
        ip_table.add_row(row![ip, count.to_string()]);
    }
    ip_table.printstd();
    println!("");

    let mut url_table = Table::new();
    url_table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    url_table.set_titles(Row::new(vec![
        Cell::new("Top URLs").with_style(Attr::Bold),
        Cell::new("Requests").with_style(Attr::Bold),
    ]));
    for (url, count) in top_urls {
        url_table.add_row(row![url, count.to_string()]);
    }

    url_table.printstd();
}
