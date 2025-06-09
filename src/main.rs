mod app;
mod log_data;
mod helpers;
mod tui_manager;

use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use std::env;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::{Arc, mpsc, Mutex};
use std::time::{Duration};
use env_logger::Builder;
use log::{error, LevelFilter};
use structopt::StructOpt;
use tokio::time::sleep;
use crate::app::App;
use crate::helpers::tail_file;
use crate::log_data::LogData;
use anyhow::{Result, Context};

#[derive(StructOpt)]
#[structopt(
    name = "log Util",
    author = "s00d",
    about = "A tool to analyze Nginx access logs.\n\n\
    GitHub: https://github.com/s00d/logutil"
)]
struct Cli {
    /// Path to the log file
    #[structopt(parse(from_os_str))]
    file: PathBuf,

    /// Number of lines to read from the end of the file (0 to start from the end, -1 to read the entire file)
    #[structopt(short = "c", long, default_value = "0")]
    count: isize,

    /// Regular expression to parse the log entries or path to a file containing the regex
    #[structopt(
        short,
        long,
        default_value = r#"^(\S+) - ".+" \[(.*?)\] \d+\.\d+ "(\S+)" "(\S+) (\S+?)(?:\?.*?)? "#
    )]
    regex: String,

    /// Date format to parse the log entries
    #[structopt(
        short = "d",
        long,
        default_value = "%d/%b/%Y:%H:%M:%S %z"
    )]
    date_format: String,

    /// Number of top entries to display
    #[structopt(short, long, default_value = "10")]
    top: usize,

    /// Disable clearing of outdated entries
    #[structopt(long)]
    no_clear: bool,

    /// Show top URLs in console
    #[structopt(long)]
    show_urls: bool,

    /// Show top IPs in console
    #[structopt(long)]
    show_ips: bool,

    /// Enable logging to a file
    #[structopt(long)]
    log_to_file: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::from_args();

    if env::args().any(|arg| arg == "-h" || arg == "--help") {
        return Ok(());
    }

    if let Err(e) = env::set_current_dir(env::current_dir().expect("Failed to get current directory")) {
        error!("Failed to set current directory: {:?}", e);
    }

    if args.log_to_file {
        let log_file = File::create("app.log").context("Unable to create log file")?;
        Builder::new()
            .filter(None, LevelFilter::Info)
            .write_style(env_logger::WriteStyle::Always)
            .target(env_logger::Target::Pipe(Box::new(log_file)))
            .init();
    } else {
        env_logger::init();
    }

    let file_path = args.file.clone();
    let count = args.count;
    let regex_pattern = if Path::new(&args.regex).exists() {
        fs::read_to_string(&args.regex).context("Could not read regex file")?
    } else {
        args.regex.clone()
    };
    let date_format = args.date_format.clone();
    let top_n = args.top;
    let no_clear = args.no_clear;

    let log_data = Arc::new(Mutex::new(LogData::new()));
    let log_data_clone = Arc::clone(&log_data);

    // First read the file
    let progress_callback = |progress: f64| {
        eprint!("\rReading file: {:.1}%", progress * 100.0);
    };
    tail_file(&file_path, count, &regex_pattern, &date_format, &log_data_clone, no_clear, None, progress_callback)
        .await
        .context("Error reading file")?;

    // Output statistics to console if requested
    if args.show_urls || args.show_ips {
        let log_data = log_data.lock().unwrap();
        let (top_ips, top_urls) = log_data.get_top_n(args.top);
        let (unique_ips, unique_urls) = log_data.get_unique_counts();

        if args.show_urls {
            println!("\nTop {} URLs (total unique: {}):", args.top, unique_urls);
            println!("{:<50} | {:<10} | {:<10}", "URL", "Requests", "Type");
            println!("{:-<50}-+-{:-<10}-+-{:-<10}", "", "", "");
            for (url, entry) in top_urls {
                println!("{:<50} | {:<10} | {:<10}", 
                    url,
                    entry.count,
                    entry.request_type
                );
            }
        }

        if args.show_ips {
            println!("\nTop {} IPs (total unique: {}):", args.top, unique_ips);
            println!("{:<15} | {:<10} | {:<10}", "IP", "Requests", "Type");
            println!("{:-<15}-+-{:-<10}-+-{:-<10}", "", "", "");
            for (ip, entry) in top_ips {
                println!("{:<15} | {:<10} | {:<10}", 
                    ip,
                    entry.count,
                    entry.request_type
                );
            }
        }
        return Ok(());
    }

    // Если не запрошен вывод статистики, запускаем TUI
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen).context("Failed to enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;

    terminal.clear().context("Failed to clear terminal")?;

    let (tx, rx) = mpsc::channel();

    let app = Arc::new(Mutex::new(App::new(log_data.clone(), top_n)));
    let app_clone = Arc::clone(&app);

    let handle = tokio::spawn(async move {
        let progress_callback = {
            let app = Arc::clone(&app_clone);
            move |progress| {
                let mut app = app.lock().unwrap();
                app.set_progress(progress)
            }
        };

        let mut last_processed_line: Option<usize> = None;
        loop {
            if rx.try_recv().is_ok() {
                break;
            }
            match tail_file(&file_path, 0, &regex_pattern, &date_format, &log_data_clone, no_clear, last_processed_line.clone(), progress_callback.clone()).await {
                Ok(last_line) => {
                    last_processed_line = last_line;
                }
                Err(e) => {
                    error!("Error reading file: {:?}", e);
                }
            }
            sleep(Duration::from_secs(1)).await;
        }
    });

    loop {
        terminal.draw(|f| {
            let mut app = app.lock().unwrap();
            app.draw(f)
        }).context("Failed to draw terminal")?;

        if event::poll(Duration::from_millis(100)).context("Failed to poll events")? {
            if let Event::Key(key) = event::read().context("Failed to read event")? {
                let mut app = app.lock().unwrap();
                app.handle_input(key.code, key.modifiers);
            }
        }

        if app.lock().unwrap().should_quit {
            break;
        }
    }

    disable_raw_mode().context("Failed to disable raw mode")?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen).context("Failed to leave alternate screen")?;
    terminal.show_cursor().context("Failed to show cursor")?;

    tx.send(()).unwrap();
    handle.await.unwrap();

    Ok(())
}


