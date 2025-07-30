mod app;
mod file_reader;
mod file_settings;
mod progress_bar;
mod memory_db;
mod tab_manager;
mod tabs;
mod tui_manager;

use crate::app::App;
use app::AppConfig;

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    Terminal,
};

use crate::file_settings::{CliArgs, FileSettings, FileSettingsAction};
use crate::file_reader::FileReader;
use crate::tui_manager::{hide_progress_bar};
use anyhow::{Context, Result};
use env_logger::Builder;
use log::{error, LevelFilter};
use std::env;
use std::fs::File;

use std::path::PathBuf;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;
use structopt::StructOpt;
use tokio::sync::mpsc;
use tokio::time::sleep;


#[derive(StructOpt)]
#[structopt(
    name = "log Util",
    author = "s00d",
    about = "A tool to analyze Nginx access logs.\n\n\
    GitHub: https://github.com/s00d/logutil"
)]
struct Cli {
    /// Path to the log file (optional - if not provided, file selector will be shown)
    #[structopt(parse(from_os_str))]
    file: Option<PathBuf>,

    /// Number of lines to read from the end of the file (0 to start from the end, -1 to read the entire file)
    #[structopt(short = "c", long, default_value = "0")]
    count: isize,

    /// Regular expression to parse the log entries or path to a file containing the regex
    /// DO NOT CHANGE DEFAULT VALUES - они настроены для совместимости с существующими логами
    #[structopt(
        short,
        long,
        default_value = r#"^(\S+) - ".+" \[(.*?)\] \d+\.\d+ "(\S+)" "(\S+) (\S+?)(?:\?.*?)? "#
    )]
    regex: String,

    /// Date format to parse the log entries
    /// DO NOT CHANGE DEFAULT VALUES - они настроены для совместимости с существующими логами
    #[structopt(short = "d", long, default_value = "%d/%b/%Y:%H:%M:%S %z")]
    date_format: String,

    /// Number of top entries to display
    #[structopt(short, long, default_value = "10")]
    top: usize,

    /// Show top URLs in console
    #[structopt(long)]
    show_urls: bool,

    /// Show top IPs in console
    #[structopt(long)]
    show_ips: bool,

    /// Enable logging to a file
    #[structopt(long)]
    log_to_file: bool,

    /// Enable Security tab (detect suspicious activity, attacks, etc.)
    #[structopt(long)]
    enable_security: bool,

    /// Enable Performance tab (monitor response times, slow requests)
    #[structopt(long)]
    enable_performance: bool,

    /// Enable Errors tab (track error codes and failed requests)
    #[structopt(long)]
    enable_errors: bool,

    /// Enable Bots tab (detect bot traffic and crawlers)
    #[structopt(long)]
    enable_bots: bool,

    /// Enable Sparkline tab (real-time request rate visualization)
    #[structopt(long)]
    enable_sparkline: bool,

    /// Enable Heatmap tab (hourly traffic patterns visualization)
    #[structopt(long)]
    enable_heatmap: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::from_args();

    // Простая запись в файл для отладки
    if args.log_to_file {
        if let Ok(mut file) = std::fs::File::create("logutil.log") {
            use std::io::Write;
            let _ = writeln!(file, "Logutil started at {}", chrono::Local::now());
        }
    }

    // Если файл не указан или указана пустая строка, запускаем интерактивный режим
    if args.file.is_none()
        || args
            .file
            .as_ref()
            .expect("File path should be Some when checking")
            .to_string_lossy()
            .trim()
            .is_empty()
    {
        return run_interactive_mode(args).await;
    }

    let file_path = args
        .file
        .expect("File path should be Some after validation");

    // Проверяем существование файла
    if !file_path.exists() {
        error!("File does not exist: {}", file_path.display());
        return Err(anyhow::anyhow!(
            "File does not exist: {}",
            file_path.display()
        ));
    }

    // Создаем CliArgs для передачи в run_analysis_with_args
    let cli_args = CliArgs {
        file: Some(file_path),
        regex: args.regex,
        date_format: args.date_format,
        count: args.count,
        top: args.top,
        show_urls: args.show_urls,
        show_ips: args.show_ips,
        log_to_file: args.log_to_file,
        enable_security: args.enable_security,
        enable_performance: args.enable_performance,
        enable_errors: args.enable_errors,
        enable_bots: args.enable_bots,
        enable_sparkline: args.enable_sparkline,
        enable_heatmap: args.enable_heatmap,
    };

    run_analysis_with_args(cli_args).await
}

async fn run_interactive_mode(args: Cli) -> Result<()> {
    // Инициализируем логирование
    if env::args().any(|arg| arg == "-h" || arg == "--help") {
        return Ok(());
    }

    if let Err(e) =
        env::set_current_dir(env::current_dir().expect("Failed to get current directory"))
    {
        error!("Failed to set current directory: {:?}", e);
    }

    // Создаем начальные CLI аргументы из переданных параметров
    let initial_cli_args = CliArgs {
        file: Some(PathBuf::new()),
        regex: args.regex,
        date_format: args.date_format,
        count: args.count,
        top: args.top,
        show_urls: args.show_urls,
        show_ips: args.show_ips,
        log_to_file: args.log_to_file,
        enable_security: args.enable_security,
        enable_performance: args.enable_performance,
        enable_errors: args.enable_errors,
        enable_bots: args.enable_bots,
        enable_sparkline: args.enable_sparkline,
        enable_heatmap: args.enable_heatmap,
    };

    let mut file_settings = FileSettings::new_with_args(&initial_cli_args);

    // Включаем поддержку мыши
    file_settings
        .enable_mouse()
        .context("Failed to enable mouse")?;

    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen).context("Failed to enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;

    terminal.clear().context("Failed to clear terminal")?;

    loop {
        terminal
            .draw(|f| {
                file_settings.draw(f, f.area());
            })
            .context("Failed to draw terminal")?;

        if event::poll(Duration::from_millis(100)).context("Failed to poll events")? {
            match event::read().context("Failed to read event")? {
                Event::Key(key) => {
                    if let Some(action) = file_settings.handle_input(key) {
                        match action {
                            FileSettingsAction::StartAnalysis(cli_args) => {
                                // Выключаем мышь перед запуском анализа
                                file_settings
                                    .disable_mouse()
                                    .context("Failed to disable mouse")?;
                                return run_analysis_with_args(cli_args).await;
                            }
                            FileSettingsAction::Exit => {
                                // Восстанавливаем терминал перед выходом
                                file_settings
                                    .disable_mouse()
                                    .context("Failed to disable mouse")?;
                                disable_raw_mode().context("Failed to disable raw mode")?;
                                execute!(terminal.backend_mut(), LeaveAlternateScreen)
                                    .context("Failed to leave alternate screen")?;
                                terminal.show_cursor().context("Failed to show cursor")?;
                                return Ok(());
                            }
                        }
                    }
                }
                Event::Mouse(mouse) => {
                    let size = terminal.size()?;
                    let total_area = Rect::new(0, 0, size.width, size.height);

                    // Вычисляем области панелей так же, как в draw методе
                    let chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(50), Constraint::Percentage(50)].as_ref(),
                        )
                        .split(total_area);

                    let file_selector_area = chunks[0];
                    let settings_area = chunks[1];

                    if let Some(action) =
                        file_settings.handle_mouse(mouse, file_selector_area, settings_area)
                    {
                        match action {
                            FileSettingsAction::StartAnalysis(cli_args) => {
                                // Выключаем мышь перед запуском анализа
                                file_settings
                                    .disable_mouse()
                                    .context("Failed to disable mouse")?;
                                disable_raw_mode().context("Failed to disable raw mode")?;
                                execute!(
                                    terminal.backend_mut(),
                                    LeaveAlternateScreen,
                                    Clear(ClearType::All)
                                )
                                .context("Failed to leave alternate screen")?;
                                return run_analysis_with_args(cli_args).await;
                            }

                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

async fn run_analysis_with_args(cli_args: CliArgs) -> Result<()> {
    // Инициализируем логирование если нужно
    if cli_args.log_to_file {
        let log_file = File::create("app.log").context("Unable to create log file")?;
        Builder::new()
            .filter(None, LevelFilter::Info)
            .write_style(env_logger::WriteStyle::Always)
            .target(env_logger::Target::Pipe(Box::new(log_file)))
            .init();
    } else {
        env_logger::init();
    }

    let count = cli_args.count;
    let regex_pattern = cli_args.regex.clone();
    let date_format = cli_args.date_format.clone();
    let _top_n = cli_args.top;

    // First read the file

    let file_path = cli_args
        .file
        .as_ref()
        .expect("File path should be Some when checking");
    
    // Инициализируем FileReader и обрабатываем файл
    let mut file_reader = FileReader::new(
        file_path.clone(),
        regex_pattern.clone(),
        date_format.clone(),
    );
    
    if let Err(e) = file_reader.initialize(count) {
        error!("Error initializing file reader: {:?}", e);
        return Err(anyhow::anyhow!("Error initializing file reader: {}", e));
    }
    
    // Получаем позицию после инициализации
    let last_processed_line = file_reader.count_lines().unwrap_or(0);
    
    eprintln!(); // Новая строка после прогресса
    hide_progress_bar(); // Скрываем прогресс-бар
                                 // Output statistics to console if requested
            if cli_args.show_urls || cli_args.show_ips {
                let db = &*crate::memory_db::GLOBAL_DB;
                let top_ips = db.get_top_ips(cli_args.top);
                let top_urls = db.get_top_urls(cli_args.top);
                let stats = db.get_stats();

                if cli_args.show_urls {
                    println!(
                        "\nTop {} URLs (total unique: {}):",
                        cli_args.top, stats.unique_urls
                    );
                    println!("{:<50} | {:<10}", "URL", "Requests");
                    println!("{:-<50}-+-{:-<10}", "", "");
                    for (url, count) in top_urls {
                        println!(
                            "{:<50} | {:<10}",
                            url, count
                        );
                    }
                }

                if cli_args.show_ips {
                    println!("\nTop {} IPs (total unique: {}):", cli_args.top, stats.unique_ips);
                    println!("{:<15} | {:<10}", "IP", "Requests");
                    println!("{:-<15}-+-{:-<10}", "", "");
                    for (ip, count) in top_ips {
                        println!(
                            "{:<15} | {:<10}",
                            ip, count
                        );
                    }
                }
                return Ok(());
            }

            // Запускаем TUI
            enable_raw_mode().context("Failed to enable raw mode")?;
            let mut stdout = std::io::stdout();
            execute!(stdout, EnterAlternateScreen).context("Failed to enter alternate screen")?;
            let backend = CrosstermBackend::new(stdout);
            let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;

            terminal.clear().context("Failed to clear terminal")?;

            let (tx, mut rx) = mpsc::channel(1);

            let app = Arc::new(StdMutex::new(App::new(AppConfig {
                enable_security: cli_args.enable_security,
                enable_performance: cli_args.enable_performance,
                enable_errors: cli_args.enable_errors,
                enable_bots: cli_args.enable_bots,
                enable_sparkline: cli_args.enable_sparkline,
                enable_heatmap: cli_args.enable_heatmap,
            })));

            let _count_clone = count;
            let regex_pattern_clone = regex_pattern.clone();
            let date_format_clone = date_format.clone();
            let cli_args_clone = cli_args.clone();
            let last_processed_line_clone = last_processed_line;

            let handle = tokio::spawn(async move {
                let mut file_reader = FileReader::new(
                    cli_args_clone.file.as_ref().unwrap().clone(),
                    regex_pattern_clone,
                    date_format_clone,
                );

                // Устанавливаем позицию на то место, где остановились
                file_reader.set_last_processed_line(last_processed_line_clone);

                loop {
                    if rx.try_recv().is_ok() {
                        break;
                    }

                    // Мониторинг новых строк (без подсчета количества строк)
                    if let Err(e) = file_reader.monitor_new_lines_without_count() {
                        error!("Error monitoring file: {:?}", e);
                    }

                    sleep(Duration::from_secs(1)).await;
                }
            });

            loop {
                terminal
                    .draw(|f| {
                        let mut app = app.lock().expect("Failed to acquire app lock for drawing");
                        app.draw(f)
                    })
                    .context("Failed to draw terminal")?;

                if event::poll(Duration::from_millis(100)).context("Failed to poll events")? {
                    if let Event::Key(key) = event::read().context("Failed to read event")? {
                        let mut app = app
                            .lock()
                            .expect("Failed to acquire app lock for input handling");
                        app.handle_input(key.code, key.modifiers);
                    }
                }

                if app
                    .lock()
                    .expect("Failed to acquire app lock for quit check")
                    .should_quit
                {
                    break;
                }
            }

            disable_raw_mode().context("Failed to disable raw mode")?;
            execute!(terminal.backend_mut(), LeaveAlternateScreen)
                .context("Failed to leave alternate screen")?;
            terminal.show_cursor().context("Failed to show cursor")?;

            tx.send(()).await.expect("Failed to send shutdown signal");
            handle.await.expect("Failed to wait for background task");

            Ok(())
}
