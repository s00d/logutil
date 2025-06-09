use logutil::log_data::{LogData};
use std::time::{SystemTime, UNIX_EPOCH, Duration};

#[test]
fn test_add_entry_and_get_top_n() {
    let mut log_data = LogData::new();
    let ip1 = "192.168.0.1".to_string();
    let ip2 = "192.168.0.2".to_string();
    let url1 = "http://example.com/page1".to_string();
    let url2 = "http://example.com/page2".to_string();
    let log_line1 = "GET /page1 HTTP/1.1".to_string();
    let log_line2 = "GET /page2 HTTP/1.1".to_string();
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;

    log_data.add_entry(ip1.clone(), url1.clone(), log_line1.clone(), timestamp, "GET".to_string(), "example.com".to_string(), false);
    log_data.add_entry(ip1.clone(), url1.clone(), log_line1.clone(), timestamp, "GET".to_string(), "example.com".to_string(), false);
    log_data.add_entry(ip2.clone(), url2.clone(), log_line2.clone(), timestamp, "GET".to_string(), "example.com".to_string(), false);

    let (top_ips, top_urls) = log_data.get_top_n(2);

    assert_eq!(top_ips.len(), 2);
    assert_eq!(top_urls.len(), 2);

    assert_eq!(top_ips[0].0, ip1);
    assert_eq!(top_ips[0].1.count, 2);

    assert_eq!(top_ips[1].0, ip2);
    assert_eq!(top_ips[1].1.count, 1);

    assert_eq!(top_urls[0].0, url1);
    assert_eq!(top_urls[0].1.count, 2);

    assert_eq!(top_urls[1].0, url2);
    assert_eq!(top_urls[1].1.count, 1);
}

#[test]
fn test_get_unique_counts() {
    let mut log_data = LogData::new();
    let ip = "192.168.0.1".to_string();
    let url = "http://example.com/page1".to_string();
    let log_line = "GET /page1 HTTP/1.1".to_string();
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;

    log_data.add_entry(ip.clone(), url.clone(), log_line.clone(), timestamp, "GET".to_string(), "example.com".to_string(), false);
    log_data.add_entry(ip.clone(), url.clone(), log_line.clone(), timestamp, "GET".to_string(), "example.com".to_string(), false);

    let (unique_ips, unique_urls) = log_data.get_unique_counts();

    assert_eq!(unique_ips, 1);
    assert_eq!(unique_urls, 1);
}

#[test]
fn test_get_last_requests() {
    let mut log_data = LogData::new();
    let ip = "192.168.0.1".to_string();
    let url = "http://example.com/page1".to_string();
    let log_line1 = "GET /page1 HTTP/1.1".to_string();
    let log_line2 = "POST /page1 HTTP/1.1".to_string();
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;

    log_data.add_entry(ip.clone(), url.clone(), log_line1.clone(), timestamp, "GET".to_string(), "example.com".to_string(), false);
    log_data.add_entry(ip.clone(), url.clone(), log_line2.clone(), timestamp, "POST".to_string(), "example.com".to_string(), false);

    let last_requests = log_data.get_last_requests(&ip);

    assert_eq!(last_requests.len(), 2);
    assert_eq!(last_requests[0], log_line2);
    assert_eq!(last_requests[1], log_line1);
}

#[test]
fn test_clear_outdated_entries() {
    let mut log_data = LogData::new();
    let ip = "192.168.0.1".to_string();
    let url = "http://example.com/page1".to_string();
    let log_line = "GET /page1 HTTP/1.1".to_string();
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;

    // Добавляем запись
    log_data.add_entry(ip.clone(), url.clone(), log_line.clone(), timestamp, "GET".to_string(), "example.com".to_string(), false);
    
    // Проверяем, что запись существует
    assert_eq!(log_data.by_ip.len(), 1);
    assert_eq!(log_data.by_url.len(), 1);

    // Имитируем устаревшую запись, изменяя last_update
    if let Some(entry) = log_data.by_ip.get_mut(&ip) {
        entry.last_update = SystemTime::now() - Duration::from_secs(1300); // Больше 20 минут
    }
    if let Some(entry) = log_data.by_url.get_mut(&url) {
        entry.last_update = SystemTime::now() - Duration::from_secs(1300); // Больше 20 минут
    }

    // Вызываем очистку
    log_data.clear_outdated_entries();

    // Проверяем, что устаревшая запись удалена
    assert_eq!(log_data.by_ip.len(), 0);
    assert_eq!(log_data.by_url.len(), 0);
}

#[test]
fn test_last_requests_limit() {
    let mut log_data = LogData::new();
    let ip = "192.168.0.1".to_string();
    let url = "http://example.com/page1".to_string();
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;

    // Добавляем 15 запросов
    for i in 0..15 {
        let log_line = format!("GET /page1?req={} HTTP/1.1", i);
        log_data.add_entry(
            ip.clone(),
            url.clone(),
            log_line,
            timestamp,
            "GET".to_string(),
            "example.com".to_string(),
            false
        );
    }

    // Проверяем, что сохранились только последние 10 запросов
    let last_requests = log_data.get_last_requests(&ip);
    assert_eq!(last_requests.len(), 10);
    
    // Проверяем, что сохранились именно последние запросы (теперь они в начале списка)
    assert!(last_requests[0].contains("req=14"));
    assert!(last_requests[9].contains("req=5"));
}

#[test]
fn test_nonexistent_ip() {
    let log_data = LogData::new();
    let nonexistent_ip = "192.168.0.999";

    // Проверяем, что для несуществующего IP возвращается пустой вектор
    let last_requests = log_data.get_last_requests(nonexistent_ip);
    assert!(last_requests.is_empty());
}

#[test]
fn test_different_request_types() {
    let mut log_data = LogData::new();
    let ip = "192.168.0.1".to_string();
    let url = "http://example.com/api".to_string();
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;

    // Добавляем запросы разных типов
    let requests = vec![
        ("GET", "/api/users"),
        ("POST", "/api/users"),
        ("PUT", "/api/users/1"),
        ("DELETE", "/api/users/1"),
    ];

    for (method, path) in requests {
        let log_line = format!("{} {} HTTP/1.1", method, path);
        log_data.add_entry(
            ip.clone(),
            url.clone(),
            log_line,
            timestamp,
            method.to_string(),
            "example.com".to_string(),
            false
        );
    }

    // Проверяем, что все запросы сохранились
    let last_requests = log_data.get_last_requests(&ip);
    assert_eq!(last_requests.len(), 4);

    // Проверяем, что типы запросов сохранились корректно
    assert!(last_requests.iter().any(|r| r.starts_with("GET")));
    assert!(last_requests.iter().any(|r| r.starts_with("POST")));
    assert!(last_requests.iter().any(|r| r.starts_with("PUT")));
    assert!(last_requests.iter().any(|r| r.starts_with("DELETE")));
}

#[test]
fn test_last_requests_order() {
    let mut log_data = LogData::new();
    let ip = "192.168.0.1".to_string();
    let url = "http://example.com/page1".to_string();
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;

    // Добавляем 5 запросов в хронологическом порядке
    let requests = vec![
        "GET /page1 HTTP/1.1",
        "GET /page2 HTTP/1.1",
        "GET /page3 HTTP/1.1",
        "GET /page4 HTTP/1.1",
        "GET /page5 HTTP/1.1",
    ];

    for request in requests.iter() {
        log_data.add_entry(
            ip.clone(),
            url.clone(),
            request.to_string(),
            timestamp,
            "GET".to_string(),
            "example.com".to_string(),
            false
        );
    }

    // Получаем последние запросы
    let last_requests = log_data.get_last_requests(&ip);

    // Проверяем, что запросы отображаются в правильном порядке (от новых к старым)
    assert_eq!(last_requests.len(), 5);
    assert_eq!(last_requests[0], "GET /page5 HTTP/1.1");
    assert_eq!(last_requests[1], "GET /page4 HTTP/1.1");
    assert_eq!(last_requests[2], "GET /page3 HTTP/1.1");
    assert_eq!(last_requests[3], "GET /page2 HTTP/1.1");
    assert_eq!(last_requests[4], "GET /page1 HTTP/1.1");

    // Добавляем еще 6 запросов, чтобы проверить ограничение в 10 записей
    let additional_requests = vec![
        "GET /page6 HTTP/1.1",
        "GET /page7 HTTP/1.1",
        "GET /page8 HTTP/1.1",
        "GET /page9 HTTP/1.1",
        "GET /page10 HTTP/1.1",
        "GET /page11 HTTP/1.1",
    ];

    for request in additional_requests.iter() {
        log_data.add_entry(
            ip.clone(),
            url.clone(),
            request.to_string(),
            timestamp,
            "GET".to_string(),
            "example.com".to_string(),
            false
        );
    }

    // Получаем обновленный список запросов
    let last_requests = log_data.get_last_requests(&ip);

    // Проверяем, что сохранились только последние 10 запросов
    assert_eq!(last_requests.len(), 10);
    
    // Проверяем, что запросы отображаются в правильном порядке (от новых к старым)
    // и что самые старые запросы (page1, page2) были удалены
    assert_eq!(last_requests[0], "GET /page11 HTTP/1.1");
    assert_eq!(last_requests[1], "GET /page10 HTTP/1.1");
    assert_eq!(last_requests[2], "GET /page9 HTTP/1.1");
    assert_eq!(last_requests[3], "GET /page8 HTTP/1.1");
    assert_eq!(last_requests[4], "GET /page7 HTTP/1.1");
    assert_eq!(last_requests[5], "GET /page6 HTTP/1.1");
    assert_eq!(last_requests[6], "GET /page5 HTTP/1.1");
    assert_eq!(last_requests[7], "GET /page4 HTTP/1.1");
    assert_eq!(last_requests[8], "GET /page3 HTTP/1.1");
    assert_eq!(last_requests[9], "GET /page2 HTTP/1.1");
} 