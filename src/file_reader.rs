use std::fs::{OpenOptions};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use regex_lite::Regex;
use log::error;
use rayon::prelude::*;
use crate::memory_db::{LogRecord, GLOBAL_DB};
use crate::progress_bar::ProgressBar;

/// Параметры для добавления записи в лог
#[derive(Debug)]
pub struct LogEntryParams {
    /// IP-адрес источника запроса
    pub ip: String,
    /// URL-путь запроса
    pub url: String,
    /// Полная строка лога
    pub log_line: String,
    /// Временная метка запроса в формате Unix timestamp
    pub timestamp: i64,
    /// Тип HTTP-запроса (GET, POST, etc.)
    pub request_type: String,
    /// Домен запроса
    pub request_domain: String,
    /// HTTP-статус код ответа
    pub status_code: Option<u16>,
    /// Размер ответа в байтах
    pub response_size: Option<u64>,
    /// Время ответа в секундах
    pub response_time: Option<f64>,
    /// User-Agent клиента
    pub user_agent: Option<String>,
}

pub struct FileReader {
    file_path: PathBuf,
    regex_pattern: String,
    date_format: String,
    last_processed_line: usize,
}

impl FileReader {
    pub fn new(file_path: PathBuf, regex_pattern: String, date_format: String) -> Self {
        Self {
            file_path,
            regex_pattern,
            date_format,
            last_processed_line: 0,
        }
    }

    /// Инициализация: устанавливает позицию в зависимости от count
    pub fn initialize(&mut self, count: isize) -> std::io::Result<()> {
        let mut progress_bar = ProgressBar::new();
        
        match count {
            -1 => {
                // Обрабатываем весь файл с начала
                self.process_all_lines(&mut progress_bar)?;
            }
            0 => {
                // Просто устанавливаем позицию на последнюю строку файла
                self.last_processed_line = self.count_lines()?;
                self.log_to_file(&format!("Set last_processed_line to {} for count=0", self.last_processed_line));
            }
            n if n > 0 => {
                // Обрабатываем последние N строк
                self.process_last_n_lines(n as usize, &mut progress_bar)?;
            }
            _ => {}
        }
        Ok(())
    }



    /// Мониторинг новых строк без подсчета количества строк
    pub fn monitor_new_lines_without_count(&mut self) -> std::io::Result<()> {
        // Просто проверяем, есть ли новые строки, не подсчитывая общее количество
        let file = OpenOptions::new().read(true).open(&self.file_path)?;
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines().filter_map(|l| l.ok()).collect();
        
        let current_line_count = lines.len();
        
        self.log_to_file(&format!("Current line count: {}, last processed: {}, thread: {:?}", 
            current_line_count, self.last_processed_line, std::thread::current().id()));
        
        if current_line_count > self.last_processed_line {
            // Есть новые строки, обрабатываем их
            let new_lines_count = current_line_count - self.last_processed_line;
            self.log_to_file(&format!("Found {} new lines, processing...", new_lines_count));
            self.process_lines_from(self.last_processed_line)?;
            self.last_processed_line = current_line_count;
            self.log_to_file(&format!("Processed {} new lines", new_lines_count));
        }
        
        Ok(())
    }

    /// Устанавливает позицию последней обработанной строки
    pub fn set_last_processed_line(&mut self, line: usize) {
        self.last_processed_line = line;
    }

    /// Записывает сообщение в лог файл
    fn log_to_file(&self, message: &str) {
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("logutil.log")
        {
            use std::io::Write;
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            let _ = writeln!(file, "[{}] {}", timestamp, message);
        }
    }

    /// Подсчитывает количество строк в файле
    pub fn count_lines(&self) -> std::io::Result<usize> {
        // Делаем несколько попыток для стабильного результата
        let mut last_count = 0;
        let mut stable_count = 0;
        
        for attempt in 0..3 {
            let file = OpenOptions::new().read(true).open(&self.file_path)?;
            let reader = BufReader::new(file);
            let count = reader.lines().count();
            
            if attempt == 0 {
                last_count = count;
                stable_count = count;
            } else if count == last_count {
                stable_count = count;
                break;
            } else {
                last_count = count;
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
        }
        
        Ok(stable_count)
    }

    /// Обрабатывает все строки файла
    fn process_all_lines(&mut self, progress_bar: &mut ProgressBar) -> std::io::Result<()> {
        let file = OpenOptions::new().read(true).open(&self.file_path)?;
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines().filter_map(|l| l.ok()).collect();
        
        progress_bar.set_total_lines(lines.len());
        
        // Обрабатываем строки параллельно блоками по 1000
        let chunk_size = 1000;
        let mut processed = 0;
        
        for chunk in lines.chunks(chunk_size) {
            // Параллельная обработка блока строк
            let results: Vec<LogRecord> = chunk
                .par_iter()
                .filter_map(|line| self.process_line_to_record(line))
                .collect();
            
            // Добавляем результаты в базу данных
            for record in results {
                let db = &*GLOBAL_DB;
                db.insert(record);
            }
            
            processed += chunk.len();
            progress_bar.update_processed_lines(processed);
            progress_bar.update(100.0 * processed as f64 / lines.len() as f64);
        }
        
        self.last_processed_line = self.count_lines()?;
        Ok(())
    }

    /// Обрабатывает последние N строк файла
    fn process_last_n_lines(&mut self, n: usize, progress_bar: &mut ProgressBar) -> std::io::Result<()> {
        let file = OpenOptions::new().read(true).open(&self.file_path)?;
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines().filter_map(|l| l.ok()).collect();
        
        let start_index = if lines.len() > n {
            lines.len() - n
        } else {
            0
        };
        
        let lines_to_process = lines.len() - start_index;
        progress_bar.set_total_lines(lines_to_process);
        
        // Обрабатываем строки параллельно блоками по 1000
        let chunk_size = 1000;
        let mut processed = 0;
        
        for chunk in lines.iter().skip(start_index).collect::<Vec<_>>().chunks(chunk_size) {
            // Параллельная обработка блока строк
            let results: Vec<LogRecord> = chunk
                .par_iter()
                .filter_map(|line| self.process_line_to_record(line))
                .collect();
            
            // Добавляем результаты в базу данных
            for record in results {
                let db = &*GLOBAL_DB;
                db.insert(record);
            }
            
            processed += chunk.len();
            progress_bar.update_processed_lines(processed);
            progress_bar.update(100.0 * processed as f64 / lines_to_process as f64);
        }
        
        self.last_processed_line = self.count_lines()?;
        Ok(())
    }

    /// Обрабатывает строки начиная с указанной позиции
    fn process_lines_from(&mut self, from_line: usize) -> std::io::Result<()> {
        let file = OpenOptions::new().read(true).open(&self.file_path)?;
        let reader = BufReader::new(file);
        let all_lines: Vec<String> = reader.lines().filter_map(|l| l.ok()).collect();
        
        // Берем только новые строки
        let new_lines: Vec<String> = all_lines.into_iter().skip(from_line).collect();
        
        if new_lines.is_empty() {
            return Ok(());
        }
        
        // Обрабатываем новые строки параллельно блоками по 1000
        let chunk_size = 1000;
        
        for chunk in new_lines.chunks(chunk_size) {
            // Параллельная обработка блока строк
            let results: Vec<LogRecord> = chunk
                .par_iter()
                .filter_map(|line| self.process_line_to_record(line))
                .collect();
            
            // Добавляем результаты в базу данных
            for record in results {
                let db = &*GLOBAL_DB;
                db.insert(record);
            }
        }
        
        Ok(())
    }



    /// Обрабатывает строку и возвращает LogRecord
    fn process_line_to_record(&self, line: &str) -> Option<LogRecord> {
        let re = match Regex::new(&self.regex_pattern) {
            Ok(re) => re,
            Err(e) => {
                error!("Regex compilation error: {}", e);
                return None;
            }
        };

        if let Ok(Some(params)) = self.parse_line(line, &re) {
            Some(LogRecord {
                id: 0,
                ip: params.ip.clone(),
                url: params.url.clone(),
                timestamp: params.timestamp,
                request_type: params.request_type.clone(),
                request_domain: params.request_domain.clone(),
                status_code: params.status_code,
                response_size: params.response_size,
                response_time: params.response_time,
                user_agent: params.user_agent.clone(),
                log_line: params.log_line.clone(),
                created_at: std::time::SystemTime::now(),
            })
        } else {
            None
        }
    }

    /// Парсит строку лога
    fn parse_line(&self, line: &str, re: &Regex) -> Result<Option<LogEntryParams>, String> {
        let captures = match re.captures(line) {
            Some(caps) => caps,
            None => return Ok(None), // Строка не совпала с regex
        };

        // Извлекаем данные из групп для оригинального формата
        let ip = captures.get(1).map(|m| m.as_str().to_string())
            .ok_or("IP group not found")?;
        
        let timestamp_str = captures.get(2).map(|m| m.as_str())
            .ok_or("Timestamp group not found")?;
        
        let http_method = captures.get(4).map(|m| m.as_str())
            .ok_or("HTTP method group not found")?;
        let url_path = captures.get(5).map(|m| m.as_str())
            .ok_or("URL path group not found")?;
        
        // Собираем полную строку запроса
        let request_line = format!("{} {}", http_method, url_path);
        
        let status_code = captures.get(6).and_then(|m| m.as_str().parse::<u16>().ok());
        let response_size = captures.get(7).and_then(|m| m.as_str().parse::<u64>().ok());
        let response_time = captures.get(8).and_then(|m| m.as_str().parse::<f64>().ok());
        let user_agent = captures.get(9).map(|m| m.as_str().to_string());

        // Парсим timestamp
        let timestamp = self.parse_timestamp(timestamp_str)?;

        // Парсим request line
        let (request_type, url, domain) = self.parse_request_line(&request_line)?;

        Ok(Some(LogEntryParams {
            ip,
            url,
            log_line: line.to_string(),
            timestamp,
            request_type,
            request_domain: domain,
            status_code,
            response_size,
            response_time,
            user_agent,
        }))
    }

    /// Парсит timestamp
    fn parse_timestamp(&self, timestamp_str: &str) -> Result<i64, String> {
        let datetime = chrono::NaiveDateTime::parse_from_str(timestamp_str, &self.date_format)
            .map_err(|e| format!("Failed to parse timestamp: {}", e))?;
        
        Ok(datetime.and_utc().timestamp())
    }

    /// Парсит request line
    fn parse_request_line(&self, request_line: &str) -> Result<(String, String, String), String> {
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 2 {
            return Err("Invalid request line format".to_string());
        }

        let method = parts[0].to_string();
        let url = parts[1].to_string();
        
        // Извлекаем домен из URL
        let domain = if url.starts_with("http://") || url.starts_with("https://") {
            if let Some(domain_start) = url.find("://") {
                let after_protocol = &url[domain_start + 3..];
                if let Some(slash_pos) = after_protocol.find('/') {
                    after_protocol[..slash_pos].to_string()
                } else {
                    after_protocol.to_string()
                }
            } else {
                "unknown".to_string()
            }
        } else {
            if let Some(slash_pos) = url.find('/') {
                url[..slash_pos].to_string()
            } else {
                "unknown".to_string()
            }
        };

        Ok((method, url, domain))
    }
}

 