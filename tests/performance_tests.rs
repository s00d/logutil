use std::time::{Instant, SystemTime};
use logutil::memory_db::{MemoryDB, LogRecord};

#[test]
fn test_performance_with_large_dataset() {
    use std::time::Instant;
    
    let db = MemoryDB::new();
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
            response_time: Some(0.1 + (i % 100) as f32 / 1000.0),
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
    
    // Тестируем получение топ результатов
    let top_start = Instant::now();
    let top_ips = db.get_top_ips(10);
    let top_time = top_start.elapsed();
    println!("Получение топ 10 IP: {:?}", top_time);
    
    let top_start = Instant::now();
    let top_urls = db.get_top_urls(10);
    let top_time = top_start.elapsed();
    println!("Получение топ 10 URL: {:?}", top_time);
    
    // Тестируем статистику
    let stats_start = Instant::now();
    let stats = db.get_stats();
    let stats_time = stats_start.elapsed();
    println!("Получение статистики: {:?}", stats_time);
    println!("Статистика: {:?}", stats);
}

#[test]
fn test_memory_usage_optimization() {
    use std::time::Instant;
    
    let mut db = MemoryDB::new();
    
    // Тестируем потребление памяти на разных объемах данных
    let test_sizes = [10_000, 100_000, 500_000, 1_000_000, 10_000_000];
    
    for &num_records in &test_sizes {
        println!("\n=== Тест с {} записями ===", num_records);
        
        let start_time = Instant::now();
        let initial_memory = db.get_memory_usage();
        
        // Добавляем записи
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
                response_time: Some(0.1 + (i % 100) as f32 / 1000.0),
                user_agent: Some(format!("test-agent-{}", i % 10)),
                log_line: format!("{} - - [10/Oct/2023:13:55:36 +0000] 0.000 \"GET\" \"GET {} HTTP/1.1\"", ip, url),
                created_at: SystemTime::now(),
            };
            
            db.insert(record);
        }
        
        let insert_time = start_time.elapsed();
        let final_memory = db.get_memory_usage();
        let memory_used = final_memory - initial_memory;
        
        println!("Время добавления: {:?}", insert_time);
        println!("Использовано памяти: {} MB", memory_used / 1024 / 1024);
        println!("Записей в секунду: {:.0}", num_records as f64 / insert_time.as_secs_f64());
        
        // Тестируем производительность операций
        let search_start = Instant::now();
        let _results = db.find_by_ip("192.168.1.1");
        let search_time = search_start.elapsed();
        println!("Время поиска по IP: {:?}", search_time);
        
        let top_start = Instant::now();
        let _top_ips = db.get_top_ips(10);
        let top_time = top_start.elapsed();
        println!("Время получения топ IP: {:?}", top_time);
        
        let stats_start = Instant::now();
        let _stats = db.get_stats();
        let stats_time = stats_start.elapsed();
        println!("Время получения статистики: {:?}", stats_time);
        
        // Очищаем для следующего теста
        db.clear();
    }
}

#[test]
fn test_memory_pressure_and_eviction() {
    let mut db = MemoryDB::new();
    
    // Устанавливаем лимит для тестирования эвикции
    db.set_max_records(1_000_000);
    
    println!("\n=== Тест давления памяти и эвикции ===");
    
    let start_time = Instant::now();
    let initial_memory = get_memory_usage();
    
    // Добавляем миллион записей
    for i in 1..=1_000_000 {
        let ip = format!("192.168.{}.{}", (i % 255) + 1, (i % 255) + 1);
        let url = format!("/api/v1/resource/{}", i);
        
        let record = LogRecord {
            id: i as u64,
            ip: ip.clone(),
            url: url.clone(),
            timestamp: 1234567890 + i as i64,
            request_type: "GET".to_string(),
            request_domain: "example.com".to_string(),
            status_code: Some(200),
            response_size: Some(1024),
            response_time: Some(0.1),
            user_agent: Some("test-agent".to_string()),
            log_line: format!("{} - - [10/Oct/2023:13:55:36 +0000] 0.000 \"GET\" \"GET {} HTTP/1.1\"", ip, url),
            created_at: SystemTime::now(),
        };
        
        db.insert(record);
        
            // Проверяем количество записей каждые 100,000
    if i % 100_000 == 0 {
        let current_records = db.get_records_count();
        let current_memory = db.get_memory_usage();
        println!("Записей: {}, Память: {} MB", current_records, current_memory / 1024 / 1024);
    }
    }
    
    let total_time = start_time.elapsed();
    let final_memory = db.get_memory_usage();
    let memory_used = final_memory - initial_memory;
    
    println!("Итоговое время: {:?}", total_time);
    println!("Итоговая память: {} MB", memory_used / 1024 / 1024);
    println!("Итоговое количество записей: {}", db.get_records_count());
    
    // Проверяем что эвикция работает
    assert!(db.get_records_count() <= 1_000_000, "Эвикция не работает");
}

#[test]
fn test_cache_performance() {
    let db = MemoryDB::new();
    
    // Добавляем тестовые данные
    for i in 1..=50_000 {
        let ip = format!("192.168.{}.{}", (i % 255) + 1, (i % 255) + 1);
        let url = format!("/api/v1/resource/{}", i);
        
        let record = LogRecord {
            id: i as u64,
            ip: ip.clone(),
            url: url.clone(),
            timestamp: 1234567890 + i as i64,
            request_type: "GET".to_string(),
            request_domain: "example.com".to_string(),
            status_code: Some(200),
            response_size: Some(1024),
            response_time: Some(0.1),
            user_agent: Some("test-agent".to_string()),
            log_line: format!("{} - - [10/Oct/2023:13:55:36 +0000] 0.000 \"GET\" \"GET {} HTTP/1.1\"", ip, url),
            created_at: SystemTime::now(),
        };
        
        db.insert(record);
    }
    
    println!("\n=== Тест производительности кэшей ===");
    
    // Тестируем кэш топ IP
    let start_time = Instant::now();
    let _top_ips_1 = db.get_top_ips(10);
    let first_call = start_time.elapsed();
    
    let start_time = Instant::now();
    let _top_ips_2 = db.get_top_ips(10);
    let cached_call = start_time.elapsed();
    
    println!("Первый вызов get_top_ips: {:?}", first_call);
    println!("Кэшированный вызов get_top_ips: {:?}", cached_call);
    println!("Ускорение: {:.1}x", first_call.as_nanos() as f64 / cached_call.as_nanos() as f64);
    
    // Тестируем кэш топ URL
    let start_time = Instant::now();
    let _top_urls_1 = db.get_top_urls(10);
    let first_call = start_time.elapsed();
    
    let start_time = Instant::now();
    let _top_urls_2 = db.get_top_urls(10);
    let cached_call = start_time.elapsed();
    
    println!("Первый вызов get_top_urls: {:?}", first_call);
    println!("Кэшированный вызов get_top_urls: {:?}", cached_call);
    println!("Ускорение: {:.1}x", first_call.as_nanos() as f64 / cached_call.as_nanos() as f64);
}

#[test]
fn test_error_handling_performance() {
    let db = MemoryDB::new();
    
    // Добавляем смесь успешных и ошибочных запросов
    for i in 1..=100_000 {
        let ip = format!("192.168.{}.{}", (i % 255) + 1, (i % 255) + 1);
        let url = format!("/api/v1/resource/{}", i);
        let status_code = if i % 20 == 0 { Some(404) } else if i % 50 == 0 { Some(500) } else { Some(200) };
        
        let record = LogRecord {
            id: i as u64,
            ip: ip.clone(),
            url: url.clone(),
            timestamp: 1234567890 + i as i64,
            request_type: "GET".to_string(),
            request_domain: "example.com".to_string(),
            status_code,
            response_size: Some(1024),
            response_time: Some(0.1),
            user_agent: Some("test-agent".to_string()),
            log_line: format!("{} - - [10/Oct/2023:13:55:36 +0000] 0.000 \"GET\" \"GET {} HTTP/1.1\"", ip, url),
            created_at: SystemTime::now(),
        };
        
        db.insert(record);
    }
    
    println!("\n=== Тест производительности обработки ошибок ===");
    
    let start_time = Instant::now();
    let error_stats = db.get_error_stats();
    let error_time = start_time.elapsed();
    
    println!("Время получения статистики ошибок: {:?}", error_time);
    println!("Статистика ошибок: {:?}", error_stats);
    
    let start_time = Instant::now();
    let top_status_codes = db.get_top_status_codes(10);
    let status_time = start_time.elapsed();
    
    println!("Время получения топ статус кодов: {:?}", status_time);
    println!("Топ статус коды: {:?}", top_status_codes);
}

/// Вспомогательная функция для получения текущего использования памяти
fn get_memory_usage() -> usize {
    // Простая реализация для Linux/macOS
    #[cfg(target_os = "linux")]
    {
        if let Ok(contents) = std::fs::read_to_string("/proc/self/status") {
            for line in contents.lines() {
                if line.starts_with("VmRSS:") {
                    if let Some(kb_str) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = kb_str.parse::<usize>() {
                            return kb * 1024; // Конвертируем KB в байты
                        }
                    }
                }
            }
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        // Для macOS используем ps
        if let Ok(output) = std::process::Command::new("ps")
            .args(&["-o", "rss=", "-p", &std::process::id().to_string()])
            .output() {
            if let Ok(memory_str) = String::from_utf8(output.stdout) {
                if let Ok(kb) = memory_str.trim().parse::<usize>() {
                    return kb * 1024; // Конвертируем KB в байты
                }
            }
        }
    }
    
    // Fallback - возвращаем 0 если не можем получить информацию
    0
}

 