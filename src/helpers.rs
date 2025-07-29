use crate::log_data::{LogData, LogEntryParams};
use chrono::{DateTime, FixedOffset};
use log::error;
use regex_lite::Regex;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::{Arc, Mutex as StdMutex};

// Кэш для скомпилированных регулярных выражений
static REGEX_CACHE: once_cell::sync::Lazy<Arc<StdMutex<HashMap<String, Regex>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(StdMutex::new(HashMap::new())));

/// Получает или компилирует регулярное выражение с кэшированием
fn get_or_compile_regex(pattern: &str) -> Result<Regex, String> {
    // Проверяем кэш
    if let Ok(cache) = REGEX_CACHE.lock() {
        if let Some(regex) = cache.get(pattern) {
            return Ok(regex.clone());
        }
    }

    // Компилируем новое регулярное выражение
    let regex = Regex::new(pattern)
        .map_err(|e| format!("Failed to compile regex pattern '{}': {}", pattern, e))?;

    // Сохраняем в кэш
    if let Ok(mut cache) = REGEX_CACHE.lock() {
        cache.insert(pattern.to_string(), regex.clone());
    }

    Ok(regex)
}

/// Предварительная валидация регулярного выражения
pub fn validate_regex_pattern(pattern: &str) -> Result<(), String> {
    Regex::new(pattern).map_err(|e| format!("Invalid regex pattern '{}': {}", pattern, e))?;
    Ok(())
}

/// Tails a file and processes new lines
pub async fn tail_file(
    file_path: &PathBuf,
    count: isize,
    regex_pattern: &str,
    date_format: &str,
    log_data: &Arc<StdMutex<LogData>>,
    last_processed_line: Option<usize>,
    progress_callback: impl Fn(f64) + Send,
) -> std::io::Result<Option<usize>> {
    // Предварительная валидация regex
    if let Err(e) = validate_regex_pattern(regex_pattern) {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, e));
    }

    let file = OpenOptions::new().read(true).open(file_path)?;

    let metadata = file.metadata()?;
    let file_size = metadata.len() as f64;
    let mut reader = BufReader::new(file);
    let mut last_processed = last_processed_line;

    if let Some(ref last_line) = last_processed_line {
        set_reader_to_last_processed_line(&mut reader, *last_line, &progress_callback, file_size)
            .await?;
    } else if count > 0 {
        process_last_n_lines(
            &mut reader,
            count,
            regex_pattern,
            date_format,
            log_data,
            &mut last_processed,
            &progress_callback,
            file_size,
        )
        .await?;
        // В TUI режиме продолжаем мониторинг даже для count > 0
        // return Ok(last_processed); // Убираем ранний возврат
    } else if count == -1 {
        process_all_lines_from_start(
            &mut reader,
            regex_pattern,
            date_format,
            log_data,
            &mut last_processed,
            &progress_callback,
            file_size,
        )
        .await?;
    } else {
        set_last_processed_to_last_line(&mut reader, &mut last_processed).await?;
    }

    // Продолжаем мониторинг для всех режимов в TUI
    process_new_lines(
        &mut reader,
        regex_pattern,
        date_format,
        log_data,
        &mut last_processed,
        &progress_callback,
        file_size,
    )
    .await?;

    Ok(last_processed)
}

async fn set_last_processed_to_last_line(
    reader: &mut BufReader<File>,
    last_processed: &mut Option<usize>,
) -> std::io::Result<()> {
    let mut buffer = String::new();
    reader.seek(SeekFrom::Start(0))?;
    reader.read_to_string(&mut buffer)?;

    let lines: Vec<&str> = buffer.lines().collect();
    *last_processed = Some(lines.len());

    Ok(())
}

async fn set_reader_to_last_processed_line(
    reader: &mut BufReader<File>,
    last_line_number: usize,
    progress_callback: &impl Fn(f64),
    file_size: f64,
) -> std::io::Result<()> {
    let mut current_line = 0;
    let mut buffer = String::new();
    let mut processed_bytes = 0;

    while current_line < last_line_number {
        let bytes_read = reader.read_line(&mut buffer)?;
        if bytes_read == 0 {
            break; // EOF reached
        }
        processed_bytes += bytes_read;
        current_line += 1;
        buffer.clear();
        progress_callback((processed_bytes as f64 / file_size).min(100.0));
    }

    Ok(())
}

async fn process_last_n_lines(
    reader: &mut BufReader<File>,
    count: isize,
    regex_pattern: &str,
    date_format: &str,
    log_data: &Arc<StdMutex<LogData>>,
    last_processed: &mut Option<usize>,
    progress_callback: &impl Fn(f64),
    _file_size: f64,
) -> std::io::Result<()> {
    let mut lines = Vec::new();
    let mut line = String::new();
    while reader.read_line(&mut line)? > 0 {
        lines.push(line.clone());
        line.clear();
    }

    let start = if lines.len() > count as usize {
        lines.len() - count as usize
    } else {
        0
    };
    let total_lines = count as usize; // Используем count вместо общего количества строк
    let mut processed_lines = 0;

    for (index, line) in lines[start..].iter().enumerate() {
        process_line(line, regex_pattern, date_format, log_data).await?;
        processed_lines += 1;
        progress_callback((processed_lines as f64 / total_lines as f64).min(1.0));
        *last_processed = Some(start + index + 1);
    }

    reader.seek(SeekFrom::Start(0))?;
    for _ in 0..*last_processed.as_ref().unwrap_or(&0) {
        let mut line = String::new();
        reader.read_line(&mut line)?;
    }

    Ok(())
}

async fn process_all_lines_from_start(
    reader: &mut BufReader<File>,
    regex_pattern: &str,
    date_format: &str,
    log_data: &Arc<StdMutex<LogData>>,
    last_processed: &mut Option<usize>,
    progress_callback: &impl Fn(f64),
    file_size: f64,
) -> std::io::Result<()> {
    reader.seek(SeekFrom::Start(0))?;
    let mut processed_bytes = 0;
    let mut line_number = 0;

    let mut line = String::new();
    while reader.read_line(&mut line)? > 0 {
        process_line(&line, regex_pattern, date_format, log_data).await?;
        processed_bytes += line.len();
        line.clear();
        progress_callback((processed_bytes as f64 / file_size).min(1.0));
        line_number += 1;
        *last_processed = Some(line_number);
    }

    Ok(())
}

async fn process_new_lines(
    reader: &mut BufReader<File>,
    regex_pattern: &str,
    date_format: &str,
    log_data: &Arc<StdMutex<LogData>>,
    last_processed: &mut Option<usize>,
    progress_callback: &impl Fn(f64),
    file_size: f64,
) -> std::io::Result<()> {
    let mut line = String::new();
    let mut processed_bytes = 0;
    let mut line_number = last_processed.unwrap_or(0);

    while reader.read_line(&mut line)? > 0 {
        process_line(&line, regex_pattern, date_format, log_data).await?;
        processed_bytes += line.len();
        line.clear();
        progress_callback((processed_bytes as f64 / file_size).min(1.0));
        line_number += 1;
        *last_processed = Some(line_number);
    }

    Ok(())
}

pub async fn process_line(
    line: &str,
    regex_pattern: &str,
    date_format: &str,
    log_data: &Arc<StdMutex<LogData>>,
) -> std::io::Result<()> {
    // Шаг 1: Получение скомпилированного регулярного выражения из кэша
    let re = match get_or_compile_regex(regex_pattern) {
        Ok(re) => re,
        Err(e) => {
            error!("Regex compilation error: {}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, e));
        }
    };

    // Шаг 2: Поиск совпадений в строке
    let caps = match re.captures(line) {
        Some(caps) => caps,
        None => {
            // Убираем логирование для несовпадающих строк - это нормально
            return Ok(());
        }
    };

    // Шаг 3: Извлечение данных из совпадений
    let (ip, datetime_str, request_domain, request_type, url) = match extract_captures_safe(&caps) {
        Ok(data) => data,
        Err(e) => {
            error!(
                "Failed to extract captures from line: {} (Error: {})",
                line.trim(),
                e
            );
            return Ok(()); // Не критическая ошибка, продолжаем обработку
        }
    };

    // Проверяем, что IP не пустой
    if ip.is_empty() {
        error!("Empty IP address in line: {}", line.trim());
        return Ok(());
    }

    // Шаг 4: Парсинг даты
    let datetime = match parse_datetime_safe(&datetime_str, date_format) {
        Ok(dt) => dt,
        Err(e) => {
            error!(
                "Failed to parse datetime '{}' with format '{}': {}",
                datetime_str, date_format, e
            );
            return Ok(()); // Не критическая ошибка, продолжаем обработку
        }
    };

    // Шаг 5: Извлечение дополнительных данных
    let (status_code, response_size, response_time, user_agent) =
        extract_additional_data_safe(line);

    // Шаг 6: Добавление записи в LogData
    let mut log_data = log_data
        .lock()
        .expect("Failed to acquire log data lock for entry addition");

    let params = LogEntryParams {
        ip,
        url,
        log_line: line.to_string(),
        timestamp: datetime.timestamp(),
        request_type,
        request_domain,
        status_code,
        response_size,
        response_time,
        user_agent,
    };

    log_data.add_entry(params);

    Ok(())
}

fn extract_captures_safe(
    caps: &regex_lite::Captures,
) -> Result<(String, String, String, String, String), String> {
    let ip = caps.get(1).map_or("", |m| m.as_str()).to_string();
    let datetime_str = caps.get(2).map_or("", |m| m.as_str()).to_string();
    let request_domain = caps.get(3).map_or("", |m| m.as_str()).to_string();
    let request_type = caps.get(4).map_or("", |m| m.as_str()).to_string();
    let url = caps.get(5).map_or("", |m| m.as_str()).to_string();

    // Проверяем, что все обязательные поля не пустые
    if ip.is_empty() {
        return Err("IP address is empty".to_string());
    }
    if datetime_str.is_empty() {
        return Err("Datetime is empty".to_string());
    }
    if request_type.is_empty() {
        return Err("Request type is empty".to_string());
    }
    if url.is_empty() {
        return Err("URL is empty".to_string());
    }

    Ok((ip, datetime_str, request_domain, request_type, url))
}

fn parse_datetime_safe(
    datetime_str: &str,
    date_format: &str,
) -> Result<DateTime<FixedOffset>, String> {
    // Пробуем основной формат
    match DateTime::parse_from_str(datetime_str, date_format) {
        Ok(dt) => Ok(dt),
        Err(e1) => {
            // Пробуем альтернативный формат
            match DateTime::parse_from_str(datetime_str, "%d/%b/%Y:%H:%M:%S %z") {
                Ok(dt) => Ok(dt),
                Err(e2) => {
                    // Пробуем еще один альтернативный формат
                    match DateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M:%S %z") {
                        Ok(dt) => Ok(dt),
                        Err(e3) => {
                            Err(format!("Failed to parse datetime '{}' with formats '{}', '%d/%b/%Y:%H:%M:%S %z', '%Y-%m-%d %H:%M:%S %z'. Errors: {}, {}, {}", 
                                datetime_str, date_format, e1, e2, e3))
                        }
                    }
                }
            }
        }
    }
}

fn extract_additional_data_safe(
    line: &str,
) -> (Option<u16>, Option<u64>, Option<f64>, Option<String>) {
    let mut status_code = None;
    let mut response_size = None;
    let mut user_agent = None;

    // Проверяем, что строка имеет правильный формат nginx лога
    if !line.contains('"') || !line.contains('[') || !line.contains(']') {
        return (None, None, Some(0.1), None);
    }

    // Парсим nginx лог формат: IP - - [DATE] TIME "METHOD" "REQUEST" STATUS SIZE "REFERER" "USER_AGENT"
    // Ищем статус код и размер ответа после кавычек
    let quote_positions: Vec<usize> = line
        .char_indices()
        .filter(|(_, c)| *c == '"')
        .map(|(i, _)| i)
        .collect();

    if quote_positions.len() >= 4 {
        // После второй кавычки должен быть статус код и размер
        let after_second_quote = &line[quote_positions[1] + 1..];
        let parts: Vec<&str> = after_second_quote.split_whitespace().collect();

        if parts.len() >= 2 {
            // Первый элемент после кавычек - статус код
            if let Ok(code) = parts[0].parse::<u16>() {
                status_code = Some(code);
            }

            // Второй элемент - размер ответа
            if let Ok(size) = parts[1].parse::<u64>() {
                response_size = Some(size);
            }
        }
    }

    // User-Agent обычно находится в последних кавычках
    if quote_positions.len() >= 6 {
        let ua_start = quote_positions[quote_positions.len() - 2] + 1;
        let ua_end = quote_positions[quote_positions.len() - 1];
        let ua = &line[ua_start..ua_end];
        if !ua.trim().is_empty() && ua != "-" {
            user_agent = Some(ua.trim().to_string());
        }
    }

    // Время ответа - заглушка, так как обычно не логируется в nginx
    let response_time = Some(0.1);

    (status_code, response_size, response_time, user_agent)
}
