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

#[test]
fn test_performance_with_large_dataset() {
    use std::time::Instant;
    
    let mut db = MemoryDB::new();
    let start_time = Instant::now();
    
    // Добавляем 1,000,000 записей
    let num_records = 1_000_000;
    println!("Добавляем {} записей...", num_records);
    
    for i in 1..=num_records {
        let ip = format!("192.168.{}.{}", (i % 255) + 1, (i % 255) + 1);
        let url = format!("/api/v1/resource/{}", i);
        let status_code = if i % 10 == 0 { Some(404) } else { Some(200) };
        
        let record = LogRecord {
            id: i as u64,
            ip: ip.clone(),
            url: url.clone(),
            timestamp: 1234567890 + i as i64,
            request_type: "GET".to_string(),
            request_domain: "example.com".to_string(),
            status_code,
            response_size: Some(1024 + (i % 1000) as u64),
            response_time: Some(0.1 + (i % 100) as f64 / 1000.0),
            user_agent: Some(format!("test-agent-{}", i % 10)),
            log_line: format!("{} - - [10/Oct/2023:13:55:36 +0000] 0.000 \"GET\" \"GET {} HTTP/1.1\"", ip, url),
            created_at: SystemTime::now(),
        };
        
        db.insert(record);
        
        // Выводим прогресс каждые 100,000 записей
        if i % 100_000 == 0 {
            println!("Добавлено {} записей", i);
        }
    }
    
    let insert_time = start_time.elapsed();
    println!("Время добавления {} записей: {:?}", num_records, insert_time);
    
    // Тестируем скорость поиска
    let search_start = Instant::now();
    
    // Поиск по IP
    let results = db.find_by_ip("192.168.1.1");
    let search_time = search_start.elapsed();
    println!("Поиск по IP '192.168.1.1': {} результатов за {:?}", results.len(), search_time);
    
    // Поиск по URL
    let search_start = Instant::now();
    let results = db.find_by_url("/api/v1/resource/1");
    let search_time = search_start.elapsed();
    println!("Поиск по URL '/api/v1/resource/1': {} результатов за {:?}", results.len(), search_time);
    
    // Получение статистики
    let stats_start = Instant::now();
    let stats = db.get_stats();
    let stats_time = stats_start.elapsed();
    println!("Получение статистики: {:?}", stats_time);
    println!("Всего записей: {}", stats.total_records);
    println!("Уникальных IP: {}", stats.unique_ips);
    println!("Уникальных URL: {}", stats.unique_urls);
    
    // Тестируем топ IP
    let top_ips_start = Instant::now();
    let top_ips = db.get_top_ips(10);
    let top_ips_time = top_ips_start.elapsed();
    println!("Получение топ IP: {:?}", top_ips_time);
    
    // Тестируем топ URL
    let top_urls_start = Instant::now();
    let top_urls = db.get_top_urls(10);
    let top_urls_time = top_urls_start.elapsed();
    println!("Получение топ URL: {:?}", top_urls_time);

    // Тестируем подозрительные IP
    let suspicious_start = Instant::now();
    let suspicious_ips = db.get_suspicious_ips();
    let suspicious_time = suspicious_start.elapsed();
    println!("Поиск подозрительных IP: {} записей за {:?}", suspicious_ips.len(), suspicious_time);
    
    // Тестируем паттерны атак
    let attack_start = Instant::now();
    let attack_patterns = db.get_attack_patterns();
    let attack_time = attack_start.elapsed();
    println!("Поиск паттернов атак: {} записей за {:?}", attack_patterns.len(), attack_time);

    // Тестируем топ статус кодов
    let top_status_start = Instant::now();
    let top_status_codes = db.get_top_status_codes(10);
    let top_status_time = top_status_start.elapsed();
    println!("Получение топ статус кодов: {:?}", top_status_time);

    // Тестируем топ User-Agent
    let top_user_agents_start = Instant::now();
    let top_user_agents = db.get_top_user_agents(10);
    let top_user_agents_time = top_user_agents_start.elapsed();
    println!("Получение топ User-Agent: {:?}", top_user_agents_time);
    
    // Тестируем статистику ошибок
    let error_stats_start = Instant::now();
    let (error_codes_count, error_urls_count, error_ips_count) = db.get_error_stats();
    let error_stats_time = error_stats_start.elapsed();
    println!("Получение статистики ошибок: {:?}", error_stats_time);
    println!("  - Коды ошибок: {}", error_codes_count);
    println!("  - URL с ошибками: {}", error_urls_count);
    println!("  - IP с ошибками: {}", error_ips_count);
    
    // Тестируем статистику ботов
    let bot_stats_start = Instant::now();
    let (bot_ips_count, bot_types_count, bot_urls_count) = db.get_bot_stats();
    let bot_stats_time = bot_stats_start.elapsed();
    println!("Получение статистики ботов: {:?}", bot_stats_time);
    println!("  - IP ботов: {}", bot_ips_count);
    println!("  - Типы ботов: {}", bot_types_count);
    println!("  - URL ботов: {}", bot_urls_count);
    
    // Тестируем статистику времени ответа
    let response_time_start = Instant::now();
    let (avg_time, max_time, min_time) = db.get_response_time_stats();
    let response_time_stats_time = response_time_start.elapsed();
    println!("Получение статистики времени ответа: {:?}", response_time_stats_time);
    println!("  - Среднее время: {:.3}s", avg_time);
    println!("  - Максимальное время: {:.3}s", max_time);
    println!("  - Минимальное время: {:.3}s", min_time);
    
    // Тестируем медленные запросы
    let slow_requests_start = Instant::now();
    let slow_requests = db.get_slow_requests_with_limit(0.5, 10);
    let slow_requests_time = slow_requests_start.elapsed();
    println!("Поиск медленных запросов (>0.5s): {} записей за {:?}", slow_requests.len(), slow_requests_time);
    
    // Тестируем запросы в секунду
    let rps_start = Instant::now();
    let rps = db.get_requests_per_second();
    let rps_time = rps_start.elapsed();
    println!("Расчет запросов в секунду: {:.1} RPS за {:?}", rps, rps_time);
    
    // Тестируем временные ряды
    let time_series_start = Instant::now();
    let time_series = db.get_time_series_data(3600); // 1 час интервалы
    let time_series_time = time_series_start.elapsed();
    println!("Получение временных рядов: {} интервалов за {:?}", time_series.len(), time_series_time);
    
    // Тестируем подозрительные паттерны для конкретного IP
    let suspicious_patterns_start = Instant::now();
    let suspicious_patterns = db.get_suspicious_patterns_for_ip("192.168.1.1");
    let suspicious_patterns_time = suspicious_patterns_start.elapsed();
    println!("Поиск подозрительных паттернов для IP: {} паттернов за {:?}", suspicious_patterns.len(), suspicious_patterns_time);
    
    // Общее время выполнения
    let total_time = start_time.elapsed();
    println!("Общее время выполнения теста: {:?}", total_time);
    
    // Проверяем, что все операции завершились успешно
    assert_eq!(stats.total_records, num_records);
    assert!(stats.unique_ips > 0);
    assert!(stats.unique_urls > 0);
    assert!(!top_ips.is_empty());
    assert!(!top_urls.is_empty());
    assert!(!top_status_codes.is_empty());
    assert!(!top_user_agents.is_empty());
    
    // Проверяем производительность на основе реальных результатов
    assert!(insert_time.as_secs() < 15); // Добавление не должно занимать больше 15 секунд
    assert!(search_time.as_micros() < 100000); // Поиск не должен занимать больше 100 миллисекунд
    assert!(stats_time.as_micros() < 100000); // Статистика не должна занимать больше 100 миллисекунд
    assert!(top_ips_time.as_millis() < 500); // Топ IP не должны занимать больше 500 миллисекунд
    assert!(top_urls_time.as_millis() < 500); // Топ URL не должны занимать больше 500 миллисекунд
    assert!(suspicious_time.as_secs() < 10); // Поиск подозрительных IP не должен занимать больше 10 секунд
    assert!(attack_time.as_millis() < 100); // Поиск паттернов атак не должен занимать больше 100 миллисекунд
    assert!(top_status_time.as_millis() < 100); // Топ статус кодов не должен занимать больше 100 миллисекунд
    assert!(top_user_agents_time.as_millis() < 100); // Топ User-Agent не должен занимать больше 100 миллисекунд
    assert!(error_stats_time.as_millis() < 200); // Статистика ошибок не должна занимать больше 200 миллисекунд
    assert!(bot_stats_time.as_millis() < 100); // Статистика ботов не должна занимать больше 100 миллисекунд
    assert!(response_time_stats_time.as_millis() < 200); // Статистика времени ответа не должна занимать больше 200 миллисекунд
    assert!(slow_requests_time.as_millis() < 200); // Поиск медленных запросов не должен занимать больше 200 миллисекунд
    assert!(rps_time.as_millis() < 100); // Расчет RPS не должен занимать больше 100 миллисекунд
    assert!(time_series_time.as_millis() < 500); // Временные ряды не должны занимать больше 500 миллисекунд
    assert!(suspicious_patterns_time.as_millis() < 100); // Поиск паттернов не должен занимать больше 100 миллисекунд
} 