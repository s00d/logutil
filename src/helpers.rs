use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, FixedOffset, Offset, Utc};
use log::error;
use regex::Regex;
use crate::log_data::LogData;

pub async fn tail_file(
    file_path: &PathBuf,
    count: isize,
    regex_pattern: &str,
    date_format: &str,
    log_data: &Arc<Mutex<LogData>>,
    no_clear: bool,
    last_processed_line: Option<usize>,
    progress_callback: impl Fn(f64) + Send,
) -> std::io::Result<Option<usize>> {
    let file = OpenOptions::new()
        .read(true)
        .open(file_path)?;

    let metadata = file.metadata()?;
    let file_size = metadata.len() as f64;
    let mut reader = BufReader::new(file);
    let mut last_processed = last_processed_line.clone();

    if let Some(ref last_line) = last_processed_line {
        set_reader_to_last_processed_line(&mut reader, last_line.clone(), &progress_callback, file_size).await?;
    } else if count > 0 {
        process_last_n_lines(&mut reader, count, regex_pattern, date_format, log_data, no_clear, &mut last_processed, &progress_callback, file_size).await?;
    } else if count == -1 {
        process_all_lines_from_start(&mut reader, regex_pattern, date_format, log_data, no_clear, &mut last_processed, &progress_callback, file_size).await?;
    } else {
        set_last_processed_to_last_line(&mut reader, &mut last_processed).await?;
    }

    process_new_lines(&mut reader, regex_pattern, date_format, log_data, no_clear, &mut last_processed, &progress_callback, file_size).await?;


    Ok(last_processed)
}

async fn set_last_processed_to_last_line(reader: &mut BufReader<File>, last_processed: &mut Option<usize>) -> std::io::Result<()> {
    let mut buffer = String::new();
    reader.seek(SeekFrom::Start(0))?;
    reader.read_to_string(&mut buffer)?;

    let lines: Vec<&str> = buffer.lines().collect();
    *last_processed = Some(lines.len());

    Ok(())
}

async fn set_reader_to_last_processed_line(reader: &mut BufReader<File>, last_line_number: usize, progress_callback: &impl Fn(f64), file_size: f64) -> std::io::Result<()> {
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
    log_data: &Arc<Mutex<LogData>>,
    no_clear: bool,
    last_processed: &mut Option<usize>,
    progress_callback: &impl Fn(f64),
    _file_size: f64,
) -> std::io::Result<()> {
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;

    let content = String::from_utf8_lossy(&buffer);
    let lines: Vec<&str> = content.lines().collect();
    let start = if lines.len() > count as usize { lines.len() - count as usize } else { 0 };
    let total_lines = lines.len();
    let mut processed_lines = 0;

    for (index, line) in lines[start..].iter().enumerate() {
        process_line(line, regex_pattern, date_format, log_data, no_clear).await?;
        processed_lines += 1;
        progress_callback((processed_lines as f64 / total_lines as f64).min(1.0));
        *last_processed = Some(start + index);
    }

    Ok(())
}

async fn process_all_lines_from_start(
    reader: &mut BufReader<File>,
    regex_pattern: &str,
    date_format: &str,
    log_data: &Arc<Mutex<LogData>>,
    no_clear: bool,
    last_processed: &mut Option<usize>,
    progress_callback: &impl Fn(f64),
    file_size: f64,
) -> std::io::Result<()> {
    reader.seek(SeekFrom::Start(0))?;
    let mut processed_bytes = 0;
    let mut line_number = 0;

    let mut line = String::new();
    while reader.read_line(&mut line)? > 0 {
        process_line(&line, regex_pattern, date_format, log_data, no_clear).await?;
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
    log_data: &Arc<Mutex<LogData>>,
    no_clear: bool,
    last_processed: &mut Option<usize>,
    progress_callback: &impl Fn(f64),
    file_size: f64,
) -> std::io::Result<()> {
    let mut line = String::new();
    let mut processed_bytes = 0;
    let mut line_number = last_processed.unwrap_or(0);

    while reader.read_line(&mut line)? > 0 {
        process_line(&line, regex_pattern, date_format, log_data, no_clear).await?;
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
    log_data: &Arc<Mutex<LogData>>,
    no_clear: bool,
) -> std::io::Result<()> {
    let re = Regex::new(regex_pattern).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
    if let Some(caps) = re.captures(line) {
        let (ip, datetime_str, request_domain, request_type, url) = extract_captures(&caps);

        let datetime = parse_datetime(&datetime_str, date_format);

        let mut log_data = log_data.lock().unwrap();
        log_data.add_entry(ip, url, line.to_string(), datetime.timestamp(), request_type, request_domain, no_clear);
    } else {
        error!("No match for line: {}", line);
    }

    Ok(())
}

fn extract_captures(caps: &regex::Captures) -> (String, String, String, String, String) {
    (
        caps.get(1).map_or("", |m| m.as_str()).to_string(),
        caps.get(2).map_or("", |m| m.as_str()).to_string(),
        caps.get(3).map_or("", |m| m.as_str()).to_string(),
        caps.get(4).map_or("", |m| m.as_str()).to_string(),
        caps.get(5).map_or("", |m| m.as_str()).to_string(),
    )
}

fn parse_datetime(datetime_str: &str, date_format: &str) -> DateTime<FixedOffset> {
    DateTime::parse_from_str(&datetime_str, date_format)
        .or_else(|_| DateTime::parse_from_str(&datetime_str, "%d/%b/%Y:%H:%M %S")
            .map(|dt| dt.with_timezone(&Utc.fix()))
            .map_err(|_: chrono::ParseError| ())
        )
        .unwrap_or_else(|_| Utc::now().with_timezone(&Utc.fix()))
}