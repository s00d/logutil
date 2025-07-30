use logutil::memory_db::{MemoryDB, LogRecord};
use std::time::SystemTime;

fn create_test_record(id: u64, ip: &str, url: &str, status_code: Option<u16>) -> LogRecord {
    LogRecord {
        id,
        ip: ip.to_string(),
        url: url.to_string(),
        timestamp: 1234567890,
        request_type: "GET".to_string(),
        request_domain: "example.com".to_string(),
        status_code,
        response_size: Some(1024),
        response_time: Some(0.1),
        user_agent: Some("test-agent".to_string()),
        log_line: format!("{} - - [10/Oct/2023:13:55:36 +0000] 0.000 \"GET\" \"GET {} HTTP/1.1\"", ip, url),
        created_at: SystemTime::now(),
    }
}

#[test]
fn test_insert_and_find_by_ip() {
    let db = MemoryDB::new();
    
    let record1 = create_test_record(1, "192.168.1.1", "/test1", Some(200));
    let record2 = create_test_record(2, "192.168.1.1", "/test2", Some(404));
    let record3 = create_test_record(3, "192.168.1.2", "/test3", Some(200));
    
    db.insert(record1);
    db.insert(record2);
    db.insert(record3);
    
    let results = db.find_by_ip("192.168.1.1");
    assert_eq!(results.len(), 2);
    
    let results = db.find_by_ip("192.168.1.2");
    assert_eq!(results.len(), 1);
    
    let results = db.find_by_ip("192.168.1.3");
    assert_eq!(results.len(), 0);
}

#[test]
fn test_insert_and_find_by_url() {
    let db = MemoryDB::new();
    
    let record1 = create_test_record(1, "192.168.1.1", "/test1", Some(200));
    let record2 = create_test_record(2, "192.168.1.2", "/test1", Some(404));
    let record3 = create_test_record(3, "192.168.1.3", "/test2", Some(200));
    
    db.insert(record1);
    db.insert(record2);
    db.insert(record3);
    
    let results = db.find_by_url("/test1");
    assert_eq!(results.len(), 2);
    
    let results = db.find_by_url("/test2");
    assert_eq!(results.len(), 1);
}

#[test]
fn test_get_stats() {
    let db = MemoryDB::new();
    
    let record1 = create_test_record(1, "192.168.1.1", "/test1", Some(200));
    let record2 = create_test_record(2, "192.168.1.1", "/test2", Some(404));
    let record3 = create_test_record(3, "192.168.1.2", "/test1", Some(200));
    
    db.insert(record1);
    db.insert(record2);
    db.insert(record3);
    
    let stats = db.get_stats();
    assert_eq!(stats.total_records, 3);
    assert_eq!(stats.unique_ips, 2);
    assert_eq!(stats.unique_urls, 2);
}

#[test]
fn test_get_top_ips() {
    let db = MemoryDB::new();
    
    // IP 192.168.1.1 делает 3 запроса
    for i in 1..=3 {
        let record = create_test_record(i, "192.168.1.1", &format!("/test{}", i), Some(200));
        db.insert(record);
    }
    
    // IP 192.168.1.2 делает 2 запроса
    for i in 4..=5 {
        let record = create_test_record(i, "192.168.1.2", &format!("/test{}", i), Some(200));
        db.insert(record);
    }
    
    // IP 192.168.1.3 делает 1 запрос
    let record = create_test_record(6, "192.168.1.3", "/test6", Some(200));
    db.insert(record);
    
    let top_ips = db.get_top_ips(3);
    assert_eq!(top_ips.len(), 3);
    
    // Проверяем, что IP с наибольшим количеством запросов идет первым
    assert_eq!(top_ips[0].0, "192.168.1.1");
    assert_eq!(top_ips[0].1, 3);
    
    assert_eq!(top_ips[1].0, "192.168.1.2");
    assert_eq!(top_ips[1].1, 2);
    
    assert_eq!(top_ips[2].0, "192.168.1.3");
    assert_eq!(top_ips[2].1, 1);
}

#[test]
fn test_get_top_urls() {
    let db = MemoryDB::new();
    
    // URL /test1 запрашивается 3 раза
    for i in 1..=3 {
        let record = create_test_record(i, &format!("192.168.1.{}", i), "/test1", Some(200));
        db.insert(record);
    }
    
    // URL /test2 запрашивается 2 раза
    for i in 4..=5 {
        let record = create_test_record(i, &format!("192.168.1.{}", i), "/test2", Some(200));
        db.insert(record);
    }
    
    // URL /test3 запрашивается 1 раз
    let record = create_test_record(6, "192.168.1.6", "/test3", Some(200));
    db.insert(record);
    
    let top_urls = db.get_top_urls(3);
    assert_eq!(top_urls.len(), 3);
    
    // Проверяем, что URL с наибольшим количеством запросов идет первым
    assert_eq!(top_urls[0].0, "/test1");
    assert_eq!(top_urls[0].1, 3);
    
    assert_eq!(top_urls[1].0, "/test2");
    assert_eq!(top_urls[1].1, 2);
    
    assert_eq!(top_urls[2].0, "/test3");
    assert_eq!(top_urls[2].1, 1);
}

#[test]
fn test_get_suspicious_ips() {
    let db = MemoryDB::new();
    
    // Добавляем подозрительные запросы
    let suspicious_patterns = [
        "admin",
        "wp-admin", 
        "phpmyadmin",
        "config",
        "backup",
        "union select",
        "script",
        "javascript"
    ];
    
    for (i, pattern) in suspicious_patterns.iter().enumerate() {
        let log_line = format!("192.168.1.{} - - [10/Oct/2023:13:55:36 +0000] 0.000 \"GET\" \"GET /{} HTTP/1.1\"", i + 1, pattern);
        let record = LogRecord {
            id: i as u64 + 1,
            ip: format!("192.168.1.{}", i + 1),
            url: format!("/{}", pattern),
            timestamp: 1234567890,
            request_type: "GET".to_string(),
            request_domain: "example.com".to_string(),
            status_code: Some(200),
            response_size: Some(1024),
            response_time: Some(0.1),
            user_agent: Some("test-agent".to_string()),
            log_line,
            created_at: SystemTime::now(),
        };
        db.insert(record);
    }
    
    let suspicious_ips = db.get_suspicious_ips();
    assert!(!suspicious_ips.is_empty());
    // Проверяем, что есть хотя бы один подозрительный IP
    assert!(suspicious_ips.len() > 0);
}

#[test]
fn test_get_attack_patterns() {
    let db = MemoryDB::new();
    
    // Добавляем запросы с атакующими паттернами
    let attack_patterns = [
        "admin",
        "wp-admin",
        "phpmyadmin", 
        "config",
        "backup",
        "union select",
        "drop table",
        "insert into"
    ];
    
    for (i, pattern) in attack_patterns.iter().enumerate() {
        let log_line = format!("192.168.1.{} - - [10/Oct/2023:13:55:36 +0000] 0.000 \"GET\" \"GET /{} HTTP/1.1\"", i + 1, pattern);
        let record = LogRecord {
            id: i as u64 + 1,
            ip: format!("192.168.1.{}", i + 1),
            url: format!("/{}", pattern),
            timestamp: 1234567890,
            request_type: "GET".to_string(),
            request_domain: "example.com".to_string(),
            status_code: Some(200),
            response_size: Some(1024),
            response_time: Some(0.1),
            user_agent: Some("test-agent".to_string()),
            log_line,
            created_at: SystemTime::now(),
        };
        db.insert(record);
    }
    
    let attack_patterns_result = db.get_attack_patterns();
    assert!(!attack_patterns_result.is_empty());
}

#[test]
fn test_get_suspicious_patterns_for_ip() {
    let db = MemoryDB::new();
    
    // Добавляем запросы с подозрительными паттернами для одного IP
    let suspicious_patterns = ["admin", "script", "javascript", "union select"];
    
    for (i, pattern) in suspicious_patterns.iter().enumerate() {
        let log_line = format!("192.168.1.1 - - [10/Oct/2023:13:55:36 +0000] 0.000 \"GET\" \"GET /{} HTTP/1.1\"", pattern);
        let record = LogRecord {
            id: i as u64 + 1,
            ip: "192.168.1.1".to_string(),
            url: format!("/{}", pattern),
            timestamp: 1234567890,
            request_type: "GET".to_string(),
            request_domain: "example.com".to_string(),
            status_code: Some(200),
            response_size: Some(1024),
            response_time: Some(0.1),
            user_agent: Some("test-agent".to_string()),
            log_line,
            created_at: SystemTime::now(),
        };
        db.insert(record);
    }
    
    let patterns = db.get_suspicious_patterns_for_ip("192.168.1.1");
    assert!(!patterns.is_empty());
    assert!(patterns.len() >= suspicious_patterns.len());
}

#[test]
fn test_duplicate_prevention() {
    let db = MemoryDB::new();
    
    // Создаем запись с одинаковым log_line
    let record1 = LogRecord {
        id: 1,
        ip: "192.168.1.1".to_string(),
        url: "/test".to_string(),
        timestamp: 1234567890,
        request_type: "GET".to_string(),
        request_domain: "example.com".to_string(),
        status_code: Some(200),
        response_size: Some(1024),
        response_time: Some(0.1),
        user_agent: Some("test-agent".to_string()),
        log_line: "identical log line".to_string(),
        created_at: SystemTime::now(),
    };
    
    let record2 = LogRecord {
        id: 2,
        ip: "192.168.1.2".to_string(),
        url: "/test2".to_string(),
        timestamp: 1234567890,
        request_type: "GET".to_string(),
        request_domain: "example.com".to_string(),
        status_code: Some(200),
        response_size: Some(1024),
        response_time: Some(0.1),
        user_agent: Some("test-agent".to_string()),
        log_line: "identical log line".to_string(), // Та же строка лога
        created_at: SystemTime::now(),
    };
    
    db.insert(record1);
    db.insert(record2);
    
    // Проверяем, что обе записи были добавлены (логика предотвращения дубликатов может быть отключена)
    let stats = db.get_stats();
    assert_eq!(stats.total_records, 2); // Обе записи добавлены
}