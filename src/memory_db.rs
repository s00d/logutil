use std::collections::{HashMap, BTreeMap};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Структура для хранения индексированных данных
#[derive(Debug, Clone)]
pub struct LogRecord {
    #[allow(dead_code)]
    pub id: u64,
    pub ip: String,
    pub url: String,
    pub timestamp: i64,
    pub request_type: String,
    pub request_domain: String,
    pub status_code: Option<u16>,
    pub response_size: Option<u64>,
    pub response_time: Option<f64>,
    pub user_agent: Option<String>,
    pub log_line: String,
    #[allow(dead_code)]
    pub created_at: SystemTime,
}

/// Быстрая in-memory база данных с индексацией
#[derive(Debug)]
pub struct MemoryDB {
    // Основные данные
    records: Arc<RwLock<HashMap<u64, LogRecord>>>,
    next_id: Arc<RwLock<u64>>,
    
    // Индексы для быстрого поиска
    ip_index: Arc<RwLock<HashMap<String, Vec<u64>>>>,
    url_index: Arc<RwLock<HashMap<String, Vec<u64>>>>,
    domain_index: Arc<RwLock<HashMap<String, Vec<u64>>>>,
    timestamp_index: Arc<RwLock<BTreeMap<i64, Vec<u64>>>>,
    status_code_index: Arc<RwLock<HashMap<u16, Vec<u64>>>>,
    request_type_index: Arc<RwLock<HashMap<String, Vec<u64>>>>,
    user_agent_index: Arc<RwLock<HashMap<String, Vec<u64>>>>,
    
    // Статистика
    stats: Arc<RwLock<DBStats>>,
}

#[derive(Debug, Clone)]
pub struct DBStats {
    pub total_records: usize,
    pub unique_ips: usize,
    pub unique_urls: usize,
    pub unique_domains: usize,
    pub total_requests: usize,
    pub avg_response_time: f64,
    pub total_response_size: u64,
}

impl DBStats {
    pub fn new() -> Self {
        Self {
            total_records: 0,
            unique_ips: 0,
            unique_urls: 0,
            unique_domains: 0,
            total_requests: 0,
            avg_response_time: 0.0,
            total_response_size: 0,
        }
    }
}

impl MemoryDB {
    pub fn new() -> Self {
        Self {
            records: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(RwLock::new(1)),
            ip_index: Arc::new(RwLock::new(HashMap::new())),
            url_index: Arc::new(RwLock::new(HashMap::new())),
            domain_index: Arc::new(RwLock::new(HashMap::new())),
            timestamp_index: Arc::new(RwLock::new(BTreeMap::new())),
            status_code_index: Arc::new(RwLock::new(HashMap::new())),
            request_type_index: Arc::new(RwLock::new(HashMap::new())),
            user_agent_index: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(DBStats::new())),
        }
    }

    /// Добавляет новую запись в базу данных
    pub fn insert(&self, record: LogRecord) -> u64 {
        let id = {
            let mut next_id = self.next_id.write().unwrap();
            let id = *next_id;
            *next_id += 1;
            id
        };

        // Добавляем запись в основную таблицу
        {
            let mut records = self.records.write().unwrap();
            records.insert(id, record.clone());
        }

        // Обновляем индексы
        self.update_indexes(id, &record);

        // Обновляем статистику
        self.update_stats(&record);

        id
    }

    /// Обновляет все индексы для новой записи
    fn update_indexes(&self, id: u64, record: &LogRecord) {
        // IP индекс
        {
            let mut ip_index = self.ip_index.write().unwrap();
            ip_index.entry(record.ip.clone()).or_insert_with(Vec::new).push(id);
        }

        // URL индекс
        {
            let mut url_index = self.url_index.write().unwrap();
            url_index.entry(record.url.clone()).or_insert_with(Vec::new).push(id);
        }

        // Domain индекс
        {
            let mut domain_index = self.domain_index.write().unwrap();
            domain_index.entry(record.request_domain.clone()).or_insert_with(Vec::new).push(id);
        }

        // Timestamp индекс
        {
            let mut timestamp_index = self.timestamp_index.write().unwrap();
            timestamp_index.entry(record.timestamp).or_insert_with(Vec::new).push(id);
        }

        // Status code индекс
        if let Some(status_code) = record.status_code {
            let mut status_code_index = self.status_code_index.write().unwrap();
            status_code_index.entry(status_code).or_insert_with(Vec::new).push(id);
        }

        // Request type индекс
        {
            let mut request_type_index = self.request_type_index.write().unwrap();
            request_type_index.entry(record.request_type.clone()).or_insert_with(Vec::new).push(id);
        }

        // User agent индекс
        if let Some(ref user_agent) = record.user_agent {
            let mut user_agent_index = self.user_agent_index.write().unwrap();
            user_agent_index.entry(user_agent.clone()).or_insert_with(Vec::new).push(id);
        }
    }

    /// Обновляет статистику базы данных
    fn update_stats(&self, record: &LogRecord) {
        let mut stats = self.stats.write().unwrap();
        stats.total_records += 1;
        stats.total_requests += 1;
        
        if let Some(response_size) = record.response_size {
            stats.total_response_size += response_size;
        }
        
        if let Some(response_time) = record.response_time {
            let total_time = stats.avg_response_time * (stats.total_records - 1) as f64 + response_time;
            stats.avg_response_time = total_time / stats.total_records as f64;
        }
    }

    /// Поиск записей по IP
    pub fn find_by_ip(&self, ip: &str) -> Vec<LogRecord> {
        let ip_index = self.ip_index.read().unwrap();
        if let Some(ids) = ip_index.get(ip) {
            let records = self.records.read().unwrap();
            ids.iter()
                .filter_map(|id| records.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Поиск записей по URL
    pub fn find_by_url(&self, url: &str) -> Vec<LogRecord> {
        let url_index = self.url_index.read().unwrap();
        if let Some(ids) = url_index.get(url) {
            let records = self.records.read().unwrap();
            ids.iter()
                .filter_map(|id| records.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Поиск записей по домену
    #[allow(dead_code)]
    pub fn find_by_domain(&self, domain: &str) -> Vec<LogRecord> {
        let domain_index = self.domain_index.read().unwrap();
        if let Some(ids) = domain_index.get(domain) {
            let records = self.records.read().unwrap();
            ids.iter()
                .filter_map(|id| records.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Поиск записей по временному диапазону
    #[allow(dead_code)]
    pub fn find_by_timerange(&self, start_time: i64, end_time: i64) -> Vec<LogRecord> {
        let timestamp_index = self.timestamp_index.read().unwrap();
        let records = self.records.read().unwrap();
        
        let mut result = Vec::new();
        for (_timestamp, ids) in timestamp_index.range(start_time..=end_time) {
            for id in ids {
                if let Some(record) = records.get(id) {
                    result.push(record.clone());
                }
            }
        }
        result
    }

    /// Поиск записей по статус коду
    #[allow(dead_code)]
    pub fn find_by_status_code(&self, status_code: u16) -> Vec<LogRecord> {
        let status_code_index = self.status_code_index.read().unwrap();
        if let Some(ids) = status_code_index.get(&status_code) {
            let records = self.records.read().unwrap();
            ids.iter()
                .filter_map(|id| records.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Поиск записей по типу запроса
    #[allow(dead_code)]
    pub fn find_by_request_type(&self, request_type: &str) -> Vec<LogRecord> {
        let request_type_index = self.request_type_index.read().unwrap();
        if let Some(ids) = request_type_index.get(request_type) {
            let records = self.records.read().unwrap();
            ids.iter()
                .filter_map(|id| records.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Получение топ IP адресов
    pub fn get_top_ips(&self, limit: usize) -> Vec<(String, usize)> {
        let ip_index = self.ip_index.read().unwrap();
        let mut ip_counts: Vec<(String, usize)> = ip_index
            .iter()
            .map(|(ip, ids)| (ip.clone(), ids.len()))
            .collect();
        
        ip_counts.sort_by(|a, b| b.1.cmp(&a.1));
        ip_counts.truncate(limit);
        ip_counts
    }

    /// Получение топ URL
    pub fn get_top_urls(&self, limit: usize) -> Vec<(String, usize)> {
        let url_index = self.url_index.read().unwrap();

        let mut url_counts: Vec<(String, usize)> = url_index
            .iter()
            .map(|(url, ids)| (url.clone(), ids.len()))
            .collect();
        
        url_counts.sort_by(|a, b| b.1.cmp(&a.1));
        url_counts.truncate(limit);
        url_counts
    }

    /// Получение статистики
    pub fn get_stats(&self) -> DBStats {
        let mut stats = self.stats.read().unwrap().clone();
        
        // Обновляем уникальные счетчики
        stats.unique_ips = self.ip_index.read().unwrap().len();
        stats.unique_urls = self.url_index.read().unwrap().len();
        stats.unique_domains = self.domain_index.read().unwrap().len();
        
        stats
    }

    /// Получение всех записей (для экспорта)
    pub fn get_all_records(&self) -> Vec<LogRecord> {
        let records = self.records.read().unwrap();
        records.values().cloned().collect()
    }

    /// Очистка старых записей (по времени)
    #[allow(dead_code)]
    pub fn cleanup_old_records(&self, older_than_seconds: i64) {
        let cutoff_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64 - older_than_seconds;

        let mut records = self.records.write().unwrap();
        let mut ip_index = self.ip_index.write().unwrap();
        let mut url_index = self.url_index.write().unwrap();
        let mut domain_index = self.domain_index.write().unwrap();
        let mut timestamp_index = self.timestamp_index.write().unwrap();
        let mut status_code_index = self.status_code_index.write().unwrap();
        let mut request_type_index = self.request_type_index.write().unwrap();
        let mut user_agent_index = self.user_agent_index.write().unwrap();

        let ids_to_remove: Vec<u64> = records
            .iter()
            .filter(|(_, record)| record.timestamp < cutoff_time)
            .map(|(id, _)| *id)
            .collect();

        for id in ids_to_remove {
            if let Some(record) = records.remove(&id) {
                // Удаляем из всех индексов
                if let Some(ids) = ip_index.get_mut(&record.ip) {
                    ids.retain(|&x| x != id);
                    if ids.is_empty() {
                        ip_index.remove(&record.ip);
                    }
                }

                if let Some(ids) = url_index.get_mut(&record.url) {
                    ids.retain(|&x| x != id);
                    if ids.is_empty() {
                        url_index.remove(&record.url);
                    }
                }

                if let Some(ids) = domain_index.get_mut(&record.request_domain) {
                    ids.retain(|&x| x != id);
                    if ids.is_empty() {
                        domain_index.remove(&record.request_domain);
                    }
                }

                if let Some(ids) = timestamp_index.get_mut(&record.timestamp) {
                    ids.retain(|&x| x != id);
                    if ids.is_empty() {
                        timestamp_index.remove(&record.timestamp);
                    }
                }

                if let Some(status_code) = record.status_code {
                    if let Some(ids) = status_code_index.get_mut(&status_code) {
                        ids.retain(|&x| x != id);
                        if ids.is_empty() {
                            status_code_index.remove(&status_code);
                        }
                    }
                }

                if let Some(ids) = request_type_index.get_mut(&record.request_type) {
                    ids.retain(|&x| x != id);
                    if ids.is_empty() {
                        request_type_index.remove(&record.request_type);
                    }
                }

                if let Some(ref user_agent) = record.user_agent {
                    if let Some(ids) = user_agent_index.get_mut(user_agent) {
                        ids.retain(|&x| x != id);
                        if ids.is_empty() {
                            user_agent_index.remove(user_agent);
                        }
                    }
                }
            }
        }
    }

    /// Получение размера базы данных в памяти
    #[allow(dead_code)]
    pub fn memory_usage(&self) -> usize {
        let records = self.records.read().unwrap();
        let ip_index = self.ip_index.read().unwrap();
        let url_index = self.url_index.read().unwrap();
        
        records.len() * std::mem::size_of::<LogRecord>() +
        ip_index.len() * std::mem::size_of::<Vec<u64>>() +
        url_index.len() * std::mem::size_of::<Vec<u64>>()
    }

    /// Получение записей по статус коду с лимитом
    pub fn get_top_status_codes(&self, limit: usize) -> Vec<(u16, usize)> {
        let status_code_index = self.status_code_index.read().unwrap();
        let mut status_counts: Vec<(u16, usize)> = status_code_index
            .iter()
            .map(|(code, ids)| (*code, ids.len()))
            .collect();
        
        status_counts.sort_by(|a, b| b.1.cmp(&a.1));
        status_counts.truncate(limit);
        status_counts
    }

    /// Получение записей по типу запроса с лимитом
    #[allow(dead_code)]
    pub fn get_top_request_types(&self, limit: usize) -> Vec<(String, usize)> {
        let request_type_index = self.request_type_index.read().unwrap();
        let mut type_counts: Vec<(String, usize)> = request_type_index
            .iter()
            .map(|(req_type, ids)| (req_type.clone(), ids.len()))
            .collect();
        
        type_counts.sort_by(|a, b| b.1.cmp(&a.1));
        type_counts.truncate(limit);
        type_counts
    }

    /// Получение записей по домену с лимитом
    #[allow(dead_code)]
    pub fn get_top_domains(&self, limit: usize) -> Vec<(String, usize)> {
        let domain_index = self.domain_index.read().unwrap();
        let mut domain_counts: Vec<(String, usize)> = domain_index
            .iter()
            .map(|(domain, ids)| (domain.clone(), ids.len()))
            .collect();
        
        domain_counts.sort_by(|a, b| b.1.cmp(&a.1));
        domain_counts.truncate(limit);
        domain_counts
    }

    /// Получение записей по User-Agent с лимитом
    pub fn get_top_user_agents(&self, limit: usize) -> Vec<(String, usize)> {
        let user_agent_index = self.user_agent_index.read().unwrap();
        let mut ua_counts: Vec<(String, usize)> = user_agent_index
            .iter()
            .map(|(ua, ids)| (ua.clone(), ids.len()))
            .collect();
        
        ua_counts.sort_by(|a, b| b.1.cmp(&a.1));
        ua_counts.truncate(limit);
        ua_counts
    }

    /// Получение записей с медленным временем ответа (с лимитом)
    pub fn get_slow_requests_with_limit(&self, threshold: f64, limit: usize) -> Vec<(String, f64)> {
        let records = self.records.read().unwrap();
        let mut slow_requests: Vec<(String, f64)> = records
            .values()
            .filter_map(|r| {
                r.response_time.filter(|&time| time > threshold).map(|time| (r.url.clone(), time))
            })
            .collect();
        
        slow_requests.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        slow_requests.truncate(limit);
        slow_requests
    }

    /// Получение записей с ошибками (статус код >= 400)
    pub fn get_error_records(&self) -> Vec<LogRecord> {
        let status_code_index = self.status_code_index.read().unwrap();
        let records = self.records.read().unwrap();
        
        let mut error_records = Vec::new();
        for (code, ids) in status_code_index.iter() {
            if *code >= 400 {
                for id in ids {
                    if let Some(record) = records.get(id) {
                        error_records.push(record.clone());
                    }
                }
            }
        }
        error_records
    }

    /// Получение записей за последние N секунд
    #[allow(dead_code)]
    pub fn get_recent_records(&self, seconds: i64) -> Vec<LogRecord> {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let start_time = current_time - seconds;
        
        self.find_by_timerange(start_time, current_time)
    }

    /// Получение статистики по временным интервалам
    pub fn get_time_series_data(&self, interval_seconds: i64) -> Vec<(i64, usize)> {
        let timestamp_index = self.timestamp_index.read().unwrap();
        let mut interval_counts: HashMap<i64, usize> = HashMap::new();
        
        for (timestamp, ids) in timestamp_index.iter() {
            let interval = timestamp / interval_seconds;
            *interval_counts.entry(interval).or_insert(0) += ids.len();
        }
        
        let mut result: Vec<(i64, usize)> = interval_counts.into_iter().collect();
        result.sort_by(|a, b| a.0.cmp(&b.0));
        result
    }

    /// Поиск записей по подстроке в логе
    #[allow(dead_code)]
    pub fn search_logs(&self, query: &str) -> Vec<LogRecord> {
        let records = self.records.read().unwrap();
        records
            .values()
            .filter(|r| r.log_line.to_lowercase().contains(&query.to_lowercase()))
            .cloned()
            .collect()
    }

    /// Получение уникальных значений для поля
    #[allow(dead_code)]
    pub fn get_unique_values(&self, field: &str) -> Vec<String> {
        match field {
            "ip" => {
                let ip_index = self.ip_index.read().unwrap();
                ip_index.keys().cloned().collect()
            }
            "url" => {
                let url_index = self.url_index.read().unwrap();
                url_index.keys().cloned().collect()
            }
            "domain" => {
                let domain_index = self.domain_index.read().unwrap();
                domain_index.keys().cloned().collect()
            }
            "request_type" => {
                let request_type_index = self.request_type_index.read().unwrap();
                request_type_index.keys().cloned().collect()
            }
            _ => Vec::new(),
        }
    }

    /// Анализ безопасности - подозрительные IP
    pub fn get_suspicious_ips(&self) -> Vec<(String, usize)> {
        let records = self.get_all_records();
        let mut suspicious_ips: HashMap<String, usize> = HashMap::new();
        
        for record in records {
            let log_line = record.log_line.to_lowercase();
            let suspicious_patterns = [
                "sqlmap", "nikto", "nmap", "dirb", "gobuster", "wfuzz",
                "admin", "wp-admin", "phpmyadmin", "config", "backup",
                "union select", "drop table", "insert into", "delete from",
                "script", "javascript", "eval(", "document.cookie",
                "..", "~", "etc/passwd", "/proc/", "/sys/",
            ];

            for pattern in &suspicious_patterns {
                if log_line.contains(pattern) {
                    *suspicious_ips.entry(record.ip.clone()).or_insert(0) += 1;
                    break;
                }
            }
        }
        
        let mut result: Vec<(String, usize)> = suspicious_ips.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1));
        result.truncate(10);
        result
    }

    /// Анализ безопасности - паттерны атак
    pub fn get_attack_patterns(&self) -> Vec<(String, usize)> {
        let records = self.get_all_records();
        let mut attack_patterns: HashMap<String, usize> = HashMap::new();
        
        for record in records {
            let log_line = record.log_line.to_lowercase();
            let patterns = [
                "sqlmap", "nikto", "nmap", "dirb", "gobuster", "wfuzz",
                "admin", "wp-admin", "phpmyadmin", "config", "backup",
                "union select", "drop table", "insert into", "delete from",
                "script", "javascript", "eval(", "document.cookie",
                "..", "~", "etc/passwd", "/proc/", "/sys/",
            ];

            for pattern in &patterns {
                if log_line.contains(pattern) {
                    *attack_patterns.entry(pattern.to_string()).or_insert(0) += 1;
                }
            }
        }
        
        let mut result: Vec<(String, usize)> = attack_patterns.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1));
        result.truncate(10);
        result
    }

    /// Анализ производительности - медленные запросы
    #[allow(dead_code)]
    pub fn get_slow_requests(&self, threshold: f64) -> Vec<(String, f64)> {
        let records = self.get_all_records();
        let mut slow_requests: Vec<(String, f64)> = records
            .into_iter()
            .filter_map(|r| {
                r.response_time.filter(|&time| time > threshold).map(|time| (r.url, time))
            })
            .collect();
        
        slow_requests.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        slow_requests.truncate(10);
        slow_requests
    }

    /// Анализ производительности - статистика времени ответа
    pub fn get_response_time_stats(&self) -> (f64, f64, f64) {
        let records = self.get_all_records();
        let response_times: Vec<f64> = records
            .into_iter()
            .filter_map(|r| r.response_time)
            .collect();
        
        if response_times.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        
        let avg = response_times.iter().sum::<f64>() / response_times.len() as f64;
        let max = response_times.iter().fold(0.0_f64, |a, &b| a.max(b));
        let min = response_times.iter().fold(f64::MAX, |a, &b| a.min(b));
        
        (avg, max, min)
    }

    /// Анализ ошибок - статистика по статус кодам
    pub fn get_error_stats(&self) -> (usize, usize, usize) {
        let error_records = self.get_error_records();
        let unique_error_urls: std::collections::HashSet<String> = error_records
            .iter()
            .map(|r| r.url.clone())
            .collect();
        let unique_error_ips: std::collections::HashSet<String> = error_records
            .iter()
            .map(|r| r.ip.clone())
            .collect();
        
        (self.get_top_status_codes(10).len(), unique_error_urls.len(), unique_error_ips.len())
    }

    /// Анализ ботов - статистика
    pub fn get_bot_stats(&self) -> (usize, usize, usize) {
        let records = self.get_all_records();
        let mut bot_ips: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut bot_types: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut bot_user_agents: std::collections::HashSet<String> = std::collections::HashSet::new();
        
        let bot_patterns = [
            ("googlebot", "Google"),
            ("bingbot", "Bing"),
            ("slurp", "Yahoo"),
            ("duckduckbot", "DuckDuckGo"),
            ("facebookexternalhit", "Facebook"),
            ("twitterbot", "Twitter"),
            ("linkedinbot", "LinkedIn"),
            ("whatsapp", "WhatsApp"),
            ("telegrambot", "Telegram"),
            ("discord", "Discord"),
            ("curl", "Curl"),
            ("wget", "Wget"),
            ("python", "Python"),
            ("java", "Java"),
            ("php", "PHP"),
        ];

        for record in records {
            if let Some(ref ua) = record.user_agent {
                for (pattern, bot_type) in &bot_patterns {
                    if ua.to_lowercase().contains(pattern) {
                        bot_ips.insert(record.ip.clone());
                        bot_types.insert(bot_type.to_string());
                        bot_user_agents.insert(ua.clone());
                        break;
                    }
                }
            }
        }
        
        (bot_ips.len(), bot_types.len(), bot_user_agents.len())
    }

    /// Получение запросов в секунду
    pub fn get_requests_per_second(&self) -> f64 {
        let stats = self.get_stats();
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as f64;
        stats.total_requests as f64 / (current_time + 1.0)
    }

    /// Получение последних запросов для IP
    #[allow(dead_code)]
    pub fn get_last_requests_for_ip(&self, ip: &str, limit: usize) -> Vec<String> {
        let records = self.find_by_ip(ip);
        records.into_iter()
            .map(|r| r.log_line)
            .take(limit)
            .collect()
    }

    /// Получение подозрительных паттернов для IP
    pub fn get_suspicious_patterns_for_ip(&self, ip: &str) -> Vec<String> {
        let records = self.find_by_ip(ip);
        let mut patterns = Vec::new();
        
        for record in records {
            let log_line = record.log_line.to_lowercase();
            let suspicious_patterns = [
                "sqlmap", "nikto", "nmap", "dirb", "gobuster", "wfuzz",
                "admin", "wp-admin", "phpmyadmin", "config", "backup",
                "union select", "drop table", "insert into", "delete from",
                "script", "javascript", "eval(", "document.cookie",
                "..", "~", "etc/passwd", "/proc/", "/sys/",
            ];

            for pattern in &suspicious_patterns {
                if log_line.contains(pattern) {
                    patterns.push(pattern.to_string());
                }
            }
        }
        
        patterns
    }

    /// Получение общего количества IP
    #[allow(dead_code)]
    pub fn get_total_ips(&self) -> usize {
        let ip_index = self.ip_index.read().unwrap();
        ip_index.len()
    }

    /// Получение общего количества URL
    #[allow(dead_code)]
    pub fn get_total_urls(&self) -> usize {
        let url_index = self.url_index.read().unwrap();
        url_index.len()
    }

    /// Получение общего количества запросов
    #[allow(dead_code)]
    pub fn get_total_requests(&self) -> usize {
        let records = self.records.read().unwrap();
        records.len()
    }

    /// Очистка всех данных в базе
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        {
            let mut records = self.records.write().unwrap();
            records.clear();
        }
        {
            let mut next_id = self.next_id.write().unwrap();
            *next_id = 1;
        }
        {
            let mut ip_index = self.ip_index.write().unwrap();
            ip_index.clear();
        }
        {
            let mut url_index = self.url_index.write().unwrap();
            url_index.clear();
        }
        {
            let mut domain_index = self.domain_index.write().unwrap();
            domain_index.clear();
        }
        {
            let mut timestamp_index = self.timestamp_index.write().unwrap();
            timestamp_index.clear();
        }
        {
            let mut status_code_index = self.status_code_index.write().unwrap();
            status_code_index.clear();
        }
        {
            let mut request_type_index = self.request_type_index.write().unwrap();
            request_type_index.clear();
        }
        {
            let mut user_agent_index = self.user_agent_index.write().unwrap();
            user_agent_index.clear();
        }
        {
            let mut stats = self.stats.write().unwrap();
            *stats = DBStats::new();
        }
    }
}

impl Default for MemoryDB {
    fn default() -> Self {
        Self::new()
    }
}

/// Глобальный экземпляр синглтона
pub static GLOBAL_DB: std::sync::LazyLock<Arc<RwLock<MemoryDB>>> = std::sync::LazyLock::new(|| Arc::new(RwLock::new(MemoryDB::new()))); 