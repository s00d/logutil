use std::fs::{OpenOptions};
use std::io::Write;
use std::sync::{Arc, Mutex};
use tempfile::NamedTempFile;
use logutil::helpers::{process_line, tail_file};
use logutil::log_data::LogData;

async fn create_test_log_file(content: &str) -> NamedTempFile {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "{}", content).unwrap();
    temp_file
}

#[tokio::test]
async fn test_process_line() {
    let log_data = Arc::new(Mutex::new(LogData::new()));
    let regex_pattern = r#"^(\S+) - \S+ \[(.*?)\] \d+\.\d+ "(\S+)" "(\S+) (\S+?)(?:\?.*?)? "#;
    let date_format = "%d/%b/%Y:%H:%M:%S %z";
    let test_line = r#"127.0.0.1 - - [10/Oct/2023:13:55:36 +0000] 0.000 "GET" "GET /test HTTP/1.1" "#;

    process_line(test_line, regex_pattern, date_format, &log_data, false).await.unwrap();

    let log_data = log_data.lock().unwrap();
    assert_eq!(log_data.total_requests, 1);
    assert!(log_data.by_ip.contains_key("127.0.0.1"));
    assert!(log_data.by_url.contains_key("/test"));
}

#[tokio::test]
async fn test_process_line_invalid_format() {
    let log_data = Arc::new(Mutex::new(LogData::new()));
    let regex_pattern = r#"^(\S+) - \S+ \[(.*?)\] \d+\.\d+ "(\S+)" "(\S+) (\S+?)(?:\?.*?)? "#;
    let date_format = "%d/%b/%Y:%H:%M:%S %z";
    let invalid_line = "invalid log line format";

    process_line(invalid_line, regex_pattern, date_format, &log_data, false).await.unwrap();

    let log_data = log_data.lock().unwrap();
    assert_eq!(log_data.total_requests, 0);
}

#[tokio::test]
async fn test_tail_file_count_0() {
    let temp_file = create_test_log_file(
        r#"127.0.0.1 - - [10/Oct/2023:13:55:36 +0000] 0.000 "GET" "GET /test1 HTTP/1.1" 
127.0.0.2 - - [10/Oct/2023:13:55:37 +0000] 0.000 "GET" "GET /test2 HTTP/1.1" "#,
    ).await;

    let log_data = Arc::new(Mutex::new(LogData::new()));
    let regex_pattern = r#"^(\S+) - \S+ \[(.*?)\] \d+\.\d+ "(\S+)" "(\S+) (\S+?)(?:\?.*?)? "#;
    let date_format = "%d/%b/%Y:%H:%M:%S %z";
    let progress_callback = |_| {};

    let result = tail_file(
        &temp_file.path().to_path_buf(),
        0,
        regex_pattern,
        date_format,
        &log_data,
        false,
        None,
        progress_callback,
    ).await;

    assert!(result.is_ok());
    let log_data = log_data.lock().unwrap();
    assert_eq!(log_data.total_requests, 0); // count=0 означает только установить last_processed_line
}

#[tokio::test]
async fn test_tail_file_count_minus_1() {
    let temp_file = create_test_log_file(
        r#"127.0.0.1 - - [10/Oct/2023:13:55:36 +0000] 0.000 "GET" "GET /test1 HTTP/1.1" 
127.0.0.2 - - [10/Oct/2023:13:55:37 +0000] 0.000 "GET" "GET /test2 HTTP/1.1" "#,
    ).await;

    let log_data = Arc::new(Mutex::new(LogData::new()));
    let regex_pattern = r#"^(\S+) - \S+ \[(.*?)\] \d+\.\d+ "(\S+)" "(\S+) (\S+?)(?:\?.*?)? "#;
    let date_format = "%d/%b/%Y:%H:%M:%S %z";
    let progress_callback = |_| {};

    let result = tail_file(
        &temp_file.path().to_path_buf(),
        -1,
        regex_pattern,
        date_format,
        &log_data,
        false,
        None,
        progress_callback,
    ).await;

    assert!(result.is_ok());
    let log_data = log_data.lock().unwrap();
    assert_eq!(log_data.total_requests, 2);
    assert!(log_data.by_ip.contains_key("127.0.0.1"));
    assert!(log_data.by_ip.contains_key("127.0.0.2"));
}

#[tokio::test]
async fn test_tail_file_count_1() {
    let temp_file = create_test_log_file(
        r#"127.0.0.1 - - [10/Oct/2023:13:55:36 +0000] 0.000 "GET" "GET /test1 HTTP/1.1" 
127.0.0.2 - - [10/Oct/2023:13:55:37 +0000] 0.000 "GET" "GET /test2 HTTP/1.1" "#,
    ).await;

    let log_data = Arc::new(Mutex::new(LogData::new()));
    let regex_pattern = r#"^(\S+) - \S+ \[(.*?)\] \d+\.\d+ "(\S+)" "(\S+) (\S+?)(?:\?.*?)? "#;
    let date_format = "%d/%b/%Y:%H:%M:%S %z";
    let progress_callback = |_| {};

    let result = tail_file(
        &temp_file.path().to_path_buf(),
        1,
        regex_pattern,
        date_format,
        &log_data,
        false,
        None,
        progress_callback,
    ).await;

    assert!(result.is_ok());
    let log_data = log_data.lock().unwrap();
    assert_eq!(log_data.total_requests, 1);
    assert!(log_data.by_ip.contains_key("127.0.0.2")); // Последняя строка
    assert!(!log_data.by_ip.contains_key("127.0.0.1")); // Первая строка не должна быть обработана
}

#[tokio::test]
async fn test_tail_file_with_last_processed_line() {
    let temp_file = create_test_log_file(
        r#"127.0.0.1 - - [10/Oct/2023:13:55:36 +0000] 0.000 "GET" "GET /test1 HTTP/1.1" 
127.0.0.2 - - [10/Oct/2023:13:55:37 +0000] 0.000 "GET" "GET /test2 HTTP/1.1" "#,
    ).await;

    {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(temp_file.path())
            .unwrap();
        writeln!(file, r#"127.0.0.3 - - [10/Oct/2023:13:55:38 +0000] 0.000 "GET" "GET /test3 HTTP/1.1" "#).unwrap();
    }

    let log_data = Arc::new(Mutex::new(LogData::new()));
    let regex_pattern = r#"^(\S+) - \S+ \[(.*?)\] \d+\.\d+ "(\S+)" "(\S+) (\S+?)(?:\?.*?)? "#;
    let date_format = "%d/%b/%Y:%H:%M:%S %z";
    let progress_callback = |_| {};

    let result = tail_file(
        &temp_file.path().to_path_buf(),
        0,
        regex_pattern,
        date_format,
        &log_data,
        false,
        Some(2),
        progress_callback,
    ).await;

    assert!(result.is_ok());
    let log_data = log_data.lock().unwrap();
    assert_eq!(log_data.total_requests, 1);
    assert!(log_data.by_ip.contains_key("127.0.0.3"));
    assert!(!log_data.by_ip.contains_key("127.0.0.1"));
    assert!(!log_data.by_ip.contains_key("127.0.0.2"));
} 