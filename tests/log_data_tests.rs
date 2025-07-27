use logutil::log_data::{LogData, LogEntryParams};

#[test]
fn test_add_entry() {
    let mut log_data = LogData::with_enabled_tabs(true, true, true, true, true, true);

    let params = LogEntryParams {
        ip: "192.168.1.1".to_string(),
        url: "/test".to_string(),
        log_line: "test log line".to_string(),
        timestamp: 1234567890,
        request_type: "GET".to_string(),
        request_domain: "example.com".to_string(),
        status_code: Some(200),
        response_size: Some(1024),
        response_time: Some(0.1),
        user_agent: Some("test-agent".to_string()),
    };

    log_data.add_entry(params);

    assert_eq!(log_data.total_requests, 1);
    assert!(log_data.by_ip.contains_key("192.168.1.1"));
    assert!(log_data.by_url.contains_key("/test"));
}

#[test]
fn test_get_top_n() {
    let mut log_data = LogData::with_enabled_tabs(true, true, true, true, true, true);

    // Добавляем несколько записей
    for i in 1..=5 {
        let params = LogEntryParams {
            ip: format!("192.168.1.{}", i),
            url: format!("/test{}", i),
            log_line: format!("test log line {}", i),
            timestamp: 1234567890 + i,
            request_type: "GET".to_string(),
            request_domain: "example.com".to_string(),
            status_code: Some(200),
            response_size: Some(1024),
            response_time: Some(0.1),
            user_agent: Some("test-agent".to_string()),
        };
        log_data.add_entry(params);
    }

    let (top_ips, top_urls) = log_data.get_top_n(3);
    assert_eq!(top_ips.len(), 3);
    assert_eq!(top_urls.len(), 3);
}

#[test]
fn test_get_unique_counts() {
    let mut log_data = LogData::with_enabled_tabs(true, true, true, true, true, true);

    let params = LogEntryParams {
        ip: "192.168.1.1".to_string(),
        url: "/test".to_string(),
        log_line: "test log line".to_string(),
        timestamp: 1234567890,
        request_type: "GET".to_string(),
        request_domain: "example.com".to_string(),
        status_code: Some(200),
        response_size: Some(1024),
        response_time: Some(0.1),
        user_agent: Some("test-agent".to_string()),
    };

    log_data.add_entry(params);

    let (unique_ips, unique_urls) = log_data.get_unique_counts();
    assert_eq!(unique_ips, 1);
    assert_eq!(unique_urls, 1);
}

#[test]
fn test_get_last_requests() {
    let mut log_data = LogData::with_enabled_tabs(true, true, true, true, true, true);

    // Добавляем несколько записей для одного IP
    for i in 1..=3 {
        let params = LogEntryParams {
            ip: "192.168.1.1".to_string(),
            url: format!("/test{}", i),
            log_line: format!("test log line {}", i),
            timestamp: 1234567890 + i,
            request_type: "GET".to_string(),
            request_domain: "example.com".to_string(),
            status_code: Some(200),
            response_size: Some(1024),
            response_time: Some(0.1),
            user_agent: Some("test-agent".to_string()),
        };
        log_data.add_entry(params);
    }

    let last_requests = log_data.get_last_requests("192.168.1.1");
    assert_eq!(last_requests.len(), 3);
}

#[test]
fn test_security_data() {
    let mut log_data = LogData::with_enabled_tabs(true, false, false, false, false, false);

    let params = LogEntryParams {
        ip: "192.168.1.1".to_string(),
        url: "/admin/login".to_string(),
        log_line: "suspicious request".to_string(),
        timestamp: 1234567890,
        request_type: "POST".to_string(),
        request_domain: "example.com".to_string(),
        status_code: Some(200),
        response_size: Some(1024),
        response_time: Some(0.1),
        user_agent: Some("test-agent".to_string()),
    };

    log_data.add_entry(params);

    let (suspicious_count, attack_count, rate_limit_count) = log_data.get_security_summary();
    assert!(suspicious_count > 0 || attack_count > 0 || rate_limit_count > 0);
}

#[test]
fn test_performance_data() {
    let mut log_data = LogData::with_enabled_tabs(false, true, false, false, false, false);

    let params = LogEntryParams {
        ip: "192.168.1.1".to_string(),
        url: "/test".to_string(),
        log_line: "performance test".to_string(),
        timestamp: 1234567890,
        request_type: "GET".to_string(),
        request_domain: "example.com".to_string(),
        status_code: Some(200),
        response_size: Some(1024),
        response_time: Some(0.5),
        user_agent: Some("test-agent".to_string()),
    };

    log_data.add_entry(params);

    let (avg_time, max_time, min_time, total_size) = log_data.get_performance_summary();
    assert!(avg_time > 0.0);
    assert!(max_time > 0.0);
    assert!(min_time > 0.0);
    assert!(total_size > 0);
}

#[test]
fn test_error_data() {
    let mut log_data = LogData::with_enabled_tabs(false, false, true, false, false, false);

    let params = LogEntryParams {
        ip: "192.168.1.1".to_string(),
        url: "/error".to_string(),
        log_line: "error test".to_string(),
        timestamp: 1234567890,
        request_type: "GET".to_string(),
        request_domain: "example.com".to_string(),
        status_code: Some(404),
        response_size: Some(1024),
        response_time: Some(0.1),
        user_agent: Some("test-agent".to_string()),
    };

    log_data.add_entry(params);

    let (error_count, error_url_count, error_ip_count) = log_data.get_error_summary();
    assert!(error_count > 0 || error_url_count > 0 || error_ip_count > 0);
}

#[test]
fn test_bot_data() {
    let mut log_data = LogData::with_enabled_tabs(false, false, false, true, false, false);

    let params = LogEntryParams {
        ip: "192.168.1.1".to_string(),
        url: "/bot".to_string(),
        log_line: "bot test".to_string(),
        timestamp: 1234567890,
        request_type: "GET".to_string(),
        request_domain: "example.com".to_string(),
        status_code: Some(200),
        response_size: Some(1024),
        response_time: Some(0.1),
        user_agent: Some("bot-agent".to_string()),
    };

    log_data.add_entry(params);

    let (bot_count, bot_type_count, bot_ua_count) = log_data.get_bot_summary();
    assert!(bot_count > 0 || bot_type_count > 0 || bot_ua_count > 0);
}
