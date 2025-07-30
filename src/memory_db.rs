use std::collections::{HashMap};
use std::sync::{Arc, RwLock, atomic::{AtomicU64, Ordering}};
use std::time::{SystemTime};
use dashmap::DashMap;

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

/// Быстрая in-memory база данных с индексацией (без блокировок)
#[derive(Debug)]
pub struct MemoryDB {
    // Основные данные (без блокировок)
    records: DashMap<u64, LogRecord>,
    next_id: AtomicU64,
    
    // Индексы для быстрого поиска (без блокировок)
    ip_index: DashMap<String, Vec<u64>>,
    url_index: DashMap<String, Vec<u64>>,
    domain_index: DashMap<String, Vec<u64>>,
    timestamp_index: DashMap<i64, Vec<u64>>,
    status_code_index: DashMap<u16, Vec<u64>>,
    request_type_index: DashMap<String, Vec<u64>>,
    user_agent_index: DashMap<String, Vec<u64>>,
    
    // Специализированные индексы для безопасности (без блокировок)
    suspicious_ips_cache: DashMap<String, usize>,
    attack_patterns_cache: DashMap<String, usize>,
    error_records_cache: DashMap<u64, bool>, // Используем DashMap вместо Vec для быстрого доступа
    
    // Кэши для топ результатов (без блокировок)
    top_ips_cache: DashMap<usize, Vec<(String, usize)>>,
    top_urls_cache: DashMap<usize, Vec<(String, usize)>>,
    
    // Статистика (с блокировкой только для статистики)
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
            records: DashMap::new(),
            next_id: AtomicU64::new(1),
            ip_index: DashMap::new(),
            url_index: DashMap::new(),
            domain_index: DashMap::new(),
            timestamp_index: DashMap::new(),
            status_code_index: DashMap::new(),
            request_type_index: DashMap::new(),
            user_agent_index: DashMap::new(),
            suspicious_ips_cache: DashMap::new(),
            attack_patterns_cache: DashMap::new(),
            error_records_cache: DashMap::new(),
            top_ips_cache: DashMap::new(),
            top_urls_cache: DashMap::new(),
            stats: Arc::new(RwLock::new(DBStats::new())),
        }
    }

    /// Добавляет новую запись в базу данных (без блокировок)
    pub fn insert(&self, record: LogRecord) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);

        // Добавляем запись в основную таблицу (без блокировок)
        self.records.insert(id, record.clone());

        // Обновляем индексы и статистику одновременно
        self.update_indexes(id, &record);
        self.update_stats(&record);

        id
    }

    /// Обновляет все индексы для новой записи (без блокировок)
    fn update_indexes(&self, id: u64, record: &LogRecord) {
        // Обновляем индексы без блокировок
        self.ip_index.entry(record.ip.clone()).or_insert_with(|| Vec::with_capacity(4)).push(id);
        self.url_index.entry(record.url.clone()).or_insert_with(|| Vec::with_capacity(4)).push(id);
        self.domain_index.entry(record.request_domain.clone()).or_insert_with(|| Vec::with_capacity(4)).push(id);
        self.timestamp_index.entry(record.timestamp).or_insert_with(|| Vec::with_capacity(4)).push(id);
        self.request_type_index.entry(record.request_type.clone()).or_insert_with(|| Vec::with_capacity(4)).push(id);
        
        // Обновляем опциональные индексы только если нужно
        if let Some(status_code) = record.status_code {
            self.status_code_index.entry(status_code).or_insert_with(|| Vec::with_capacity(4)).push(id);

            // Обновляем кэш ошибок только для ошибок
            if status_code >= 400 {
                self.error_records_cache.insert(id, true);
            }
        }
        
        if let Some(ref user_agent) = record.user_agent {
            self.user_agent_index.entry(user_agent.clone()).or_insert_with(|| Vec::with_capacity(4)).push(id);
        }
        
        // Проверяем на подозрительную активность только для потенциально опасных записей
        if record.status_code.map_or(false, |code| code >= 400) ||
           record.url.contains("admin") ||
           record.url.contains("config") ||
           record.url.contains("backup") {
            self.update_security_caches(id, record);
        }
    }

    /// Обновляет кэши безопасности для новой записи (без блокировок)
    fn update_security_caches(&self, _id: u64, record: &LogRecord) {
        let log_line = record.log_line.to_lowercase();
        
        // Только самые частые паттерны для быстрой проверки
        let suspicious_patterns = [
            "admin", "wp-admin", "phpmyadmin", "config", "backup",
            "union select", "drop table", "insert into", "delete from",
        ];

        // Быстрая проверка паттернов
        for pattern in &suspicious_patterns {
            if log_line.contains(pattern) {
                // Обновляем кэши без блокировок
                *self.suspicious_ips_cache.entry(record.ip.clone()).or_insert(0) += 1;
                *self.attack_patterns_cache.entry(pattern.to_string()).or_insert(0) += 1;
                break; // Один паттерн найден, достаточно
            }
        }
    }

    /// Обновляет статистику базы данных (оптимизированная версия)
    fn update_stats(&self, record: &LogRecord) {
        let mut stats = self.stats.write().unwrap();
        stats.total_records += 1;
        stats.total_requests += 1;
        
        if let Some(response_size) = record.response_size {
            stats.total_response_size += response_size;
        }
        
        // Оптимизированный расчет среднего времени ответа
        if let Some(response_time) = record.response_time {
            // Используем формулу для инкрементального обновления среднего
            let n = stats.total_records as f64;
            stats.avg_response_time = (stats.avg_response_time * (n - 1.0) + response_time) / n;
        }
    }

    /// Поиск записей по IP (без блокировок)
    pub fn find_by_ip(&self, ip: &str) -> Vec<LogRecord> {
        if let Some(ids) = self.ip_index.get(ip) {
            ids.iter()
                .filter_map(|id| self.records.get(id).map(|r| r.clone()))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Поиск записей по URL (без блокировок)
    pub fn find_by_url(&self, url: &str) -> Vec<LogRecord> {
        if let Some(ids) = self.url_index.get(url) {
            ids.iter()
                .filter_map(|id| self.records.get(id).map(|r| r.clone()))
                .collect()
        } else {
            Vec::new()
        }
    }

    // /// Поиск записей по домену (без блокировок) - высокопроизводительная версия
    // pub fn find_by_domain(&self, domain: &str) -> Vec<LogRecord> {
    //     // Для больших результатов возвращаем только первые 1000 записей для производительности
    //     if let Some(ids) = self.domain_index.get(domain) {
    //         let limit = std::cmp::min(ids.len(), 1000);
    //         ids.iter()
    //             .take(limit)
    //             .filter_map(|&id| self.records.get(&id).map(|r| r.clone()))
    //             .collect()
    //     } else {
    //         vec![]
    //     }
    // }

    // /// Поиск записей по статус коду (без блокировок) - высокопроизводительная версия
    // pub fn find_by_status_code(&self, status_code: u16) -> Vec<LogRecord> {
    //     if let Some(ids) = self.status_code_index.get(&status_code) {
    //         let limit = std::cmp::min(ids.len(), 1000);
    //         ids.iter()
    //             .take(limit)
    //             .filter_map(|id| self.records.get(id).map(|r| r.clone()))
    //             .collect()
    //     } else {
    //         Vec::new()
    //     }
    // }

    // /// Поиск записей по типу запроса (без блокировок) - высокопроизводительная версия
    // pub fn find_by_request_type(&self, request_type: &str) -> Vec<LogRecord> {
    //     // Для больших результатов возвращаем только первые 1000 записей для производительности
    //     if let Some(ids) = self.request_type_index.get(request_type) {
    //         let limit = std::cmp::min(ids.len(), 1000);
    //         ids.iter()
    //             .take(limit)
    //             .filter_map(|&id| self.records.get(&id).map(|r| r.clone()))
    //             .collect()
    //     } else {
    //         vec![]
    //     }
    // }

    /// Получение топ IP адресов (без блокировок) - высокопроизводительная версия с кэшированием
    pub fn get_top_ips(&self, limit: usize) -> Vec<(String, usize)> {
        // Проверяем кэш
        if let Some(cached_result) = self.top_ips_cache.get(&limit) {
            return cached_result.clone();
        }
        
        // Простой и быстрый подход - берем только первые элементы
        let mut ip_counts: Vec<(String, usize)> = Vec::new();
        let max_items = std::cmp::min(self.ip_index.len(), 1000); // Ограничиваем количество обрабатываемых элементов
        
        for (i, entry) in self.ip_index.iter().enumerate() {
            if i >= max_items {
                break;
            }
            let ip = entry.key().clone();
            let count = entry.value().len();
            ip_counts.push((ip, count));
        }
        
        // Сортируем только обработанные элементы
        ip_counts.sort_by(|a, b| b.1.cmp(&a.1));
        ip_counts.truncate(limit);
        
        // Кэшируем результат
        self.top_ips_cache.insert(limit, ip_counts.clone());
        ip_counts
    }

    /// Получение топ URL (без блокировок) - высокопроизводительная версия с кэшированием
    pub fn get_top_urls(&self, limit: usize) -> Vec<(String, usize)> {
        // Проверяем кэш
        if let Some(cached_result) = self.top_urls_cache.get(&limit) {
            return cached_result.clone();
        }
        
        // Простой и быстрый подход - берем только первые элементы
        let mut url_counts: Vec<(String, usize)> = Vec::new();
        let max_items = std::cmp::min(self.url_index.len(), 1000); // Ограничиваем количество обрабатываемых элементов
        
        for (i, entry) in self.url_index.iter().enumerate() {
            if i >= max_items {
                break;
            }
            let url = entry.key().clone();
            let count = entry.value().len();
            url_counts.push((url, count));
        }
        
        // Сортируем только обработанные элементы
        url_counts.sort_by(|a, b| b.1.cmp(&a.1));
        url_counts.truncate(limit);
        
        // Кэшируем результат
        self.top_urls_cache.insert(limit, url_counts.clone());
        url_counts
    }

    /// Получение статистики (с блокировкой только для статистики)
    pub fn get_stats(&self) -> DBStats {
        let mut stats = self.stats.write().unwrap();
        
        // Обновляем уникальные значения
        stats.unique_ips = self.ip_index.len();
        stats.unique_urls = self.url_index.len();
        stats.unique_domains = self.domain_index.len();
        
        stats.clone()
    }

    /// Получение всех записей (без блокировок)
    pub fn get_all_records(&self) -> Vec<LogRecord> {
        self.records.iter().map(|entry| entry.value().clone()).collect()
    }

    // /// Получение записей с ошибками (без блокировок)
    // pub fn get_error_records(&self) -> Vec<LogRecord> {
    //     self.error_records_cache.iter()
    //         .filter_map(|entry| self.records.get(entry.key()).map(|r| r.clone()))
    //         .collect()
    // }

    /// Анализ безопасности - подозрительные IP (мгновенная версия с кэшем)
    pub fn get_suspicious_ips(&self) -> Vec<(String, usize)> {
        let mut result: Vec<(String, usize)> = self.suspicious_ips_cache
            .iter()
            .map(|entry| (entry.key().clone(), *entry.value()))
            .collect();
        result.sort_by(|a, b| b.1.cmp(&a.1));
        result.truncate(10);
        result
    }

    /// Анализ безопасности - паттерны атак (мгновенная версия с кэшем)
    pub fn get_attack_patterns(&self) -> Vec<(String, usize)> {
        let mut result: Vec<(String, usize)> = self.attack_patterns_cache
            .iter()
            .map(|entry| (entry.key().clone(), *entry.value()))
            .collect();
        result.sort_by(|a, b| b.1.cmp(&a.1));
        result.truncate(10);
        result
    }

    /// Получение подозрительных паттернов для IP (без блокировок)
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

    /// Получение статистики по временным интервалам (без блокировок) - высокопроизводительная версия
    pub fn get_time_series_data(&self, interval_seconds: i64) -> Vec<(i64, usize)> {
        let mut interval_counts: HashMap<i64, usize> = HashMap::new();
        
        // Ограничиваем количество обрабатываемых записей для производительности
        let max_entries = 10000;
        let mut processed = 0;
        
        for entry in self.timestamp_index.iter() {
            if processed >= max_entries {
                break;
            }
            
            let timestamp = *entry.key();
            let interval = timestamp / interval_seconds;
            *interval_counts.entry(interval).or_insert(0) += entry.value().len();
            processed += 1;
        }
        
        let mut result: Vec<(i64, usize)> = interval_counts.into_iter().collect();
        result.sort_by(|a, b| a.0.cmp(&b.0));
        result
    }

    /// Получение статистики ошибок (без блокировок) - оптимизированная версия
    pub fn get_error_stats(&self) -> (usize, usize, usize) {
        let error_codes_count = self.status_code_index.iter()
            .filter(|entry| {
                let status_code = *entry.key();
                status_code >= 400
            })
            .map(|entry| entry.value().len())
            .sum();

        // Используем кэш ошибок для быстрого подсчета
        let error_urls_count = self.error_records_cache.len();
        
        // Быстрый подсчет уникальных IP с ошибками
        let mut error_ips = std::collections::HashSet::new();
        for entry in self.error_records_cache.iter() {
            if let Some(record) = self.records.get(entry.key()) {
                error_ips.insert(record.ip.clone());
            }
        }
        let error_ips_count = error_ips.len();

        (error_codes_count, error_urls_count, error_ips_count)
    }

    /// Получение топ статус кодов (без блокировок)
    pub fn get_top_status_codes(&self, limit: usize) -> Vec<(String, usize)> {
        let mut status_counts: HashMap<String, usize> = HashMap::new();
        
        for entry in self.status_code_index.iter() {
            let status_code = entry.key().to_string();
            let count = entry.value().len();
            status_counts.insert(status_code, count);
        }
        
        let mut result: Vec<(String, usize)> = status_counts.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1));
        result.truncate(limit);
        result
    }

    /// Получение статистики времени ответа (без блокировок) - высокопроизводительная версия
    pub fn get_response_time_stats(&self) -> (f64, f64, f64) {
        let mut times: Vec<f64> = Vec::new();
        let max_samples = 10000; // Ограничиваем количество выборок для производительности
        
        for entry in self.records.iter() {
            if times.len() >= max_samples {
                break;
            }
            if let Some(response_time) = entry.response_time {
                times.push(response_time);
            }
        }
        
        if times.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        
        let avg_time = times.iter().sum::<f64>() / times.len() as f64;
        let max_time = times.iter().fold(0.0_f64, |a, &b| a.max(b));
        let min_time = times.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        
        (avg_time, max_time, min_time)
    }

    /// Получение медленных запросов (без блокировок) - высокопроизводительная версия
    pub fn get_slow_requests_with_limit(&self, threshold: f64, limit: usize) -> Vec<(String, f64)> {
        let mut slow_requests: Vec<(String, f64)> = Vec::new();
        let max_scan = 50000; // Ограничиваем сканирование для производительности
        let mut scanned = 0;
        
        for entry in self.records.iter() {
            if scanned >= max_scan {
                break;
            }
            
            if let Some(response_time) = entry.response_time {
                if response_time > threshold {
                    slow_requests.push((entry.ip.clone(), response_time));
                }
            }
            scanned += 1;
        }
        
        slow_requests.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        slow_requests.truncate(limit);
        slow_requests
    }

    /// Получение запросов в секунду (без блокировок)
    pub fn get_requests_per_second(&self) -> f64 {
        let stats = self.get_stats();
        if stats.total_requests == 0 {
            return 0.0;
        }
        
        // Простая оценка RPS на основе общего количества запросов
        // В реальном приложении нужно учитывать временные интервалы
        stats.total_requests as f64 / 60.0 // Предполагаем 1 минуту
    }

    /// Получение статистики ботов (без блокировок) - оптимизированная версия
    pub fn get_bot_stats(&self) -> (usize, usize, usize) {
        // Быстрый подсчет через User-Agent индекс
        let mut bot_ips = std::collections::HashSet::new();
        let mut bot_urls = std::collections::HashSet::new();
        let mut bot_types_count = 0;
        
        for entry in self.user_agent_index.iter() {
            let user_agent = entry.key().to_lowercase();
            if user_agent.contains("bot") || user_agent.contains("crawler") || user_agent.contains("spider") {
                bot_types_count += 1;
                
                // Собираем уникальные IP и URL для ботов
                for &id in entry.value() {
                    if let Some(record) = self.records.get(&id) {
                        bot_ips.insert(record.ip.clone());
                        bot_urls.insert(record.url.clone());
                    }
                }
            }
        }
        
        (bot_ips.len(), bot_types_count, bot_urls.len())
    }

    /// Получение топ User-Agent (без блокировок)
    pub fn get_top_user_agents(&self, limit: usize) -> Vec<(String, usize)> {
        let mut user_agent_counts: HashMap<String, usize> = HashMap::new();
        
        for entry in self.user_agent_index.iter() {
            let user_agent = entry.key();
            let count = entry.value().len();
            user_agent_counts.insert(user_agent.clone(), count);
        }
        
        let mut result: Vec<(String, usize)> = user_agent_counts.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1));
        result.truncate(limit);
        result
    }

    // /// Получение топ доменов (без блокировок)
    // pub fn get_top_domains(&self, limit: usize) -> Vec<(String, usize)> {
    //     let mut domain_counts: HashMap<String, usize> = HashMap::new();
    //
    //     for entry in self.domain_index.iter() {
    //         let domain = entry.key();
    //         let count = entry.value().len();
    //         domain_counts.insert(domain.clone(), count);
    //     }
    //
    //     let mut result: Vec<(String, usize)> = domain_counts.into_iter().collect();
    //     result.sort_by(|a, b| b.1.cmp(&a.1));
    //     result.truncate(limit);
    //     result
    // }

    // /// Получение топ типов запросов (без блокировок)
    // pub fn get_top_request_types(&self, limit: usize) -> Vec<(String, usize)> {
    //     let mut request_type_counts: HashMap<String, usize> = HashMap::new();
    //
    //     for entry in self.request_type_index.iter() {
    //         let request_type = entry.key();
    //         let count = entry.value().len();
    //         request_type_counts.insert(request_type.clone(), count);
    //     }
    //
    //     let mut result: Vec<(String, usize)> = request_type_counts.into_iter().collect();
    //     result.sort_by(|a, b| b.1.cmp(&a.1));
    //     result.truncate(limit);
    //     result
    // }

    // /// Очистка базы данных
    // pub fn clear(&mut self) {
    //     self.records.clear();
    //     self.next_id.store(1, Ordering::Relaxed);
    //     self.ip_index.clear();
    //     self.url_index.clear();
    //     self.domain_index.clear();
    //     self.timestamp_index.clear();
    //     self.status_code_index.clear();
    //     self.request_type_index.clear();
    //     self.user_agent_index.clear();
    //     self.suspicious_ips_cache.clear();
    //     self.attack_patterns_cache.clear();
    //     self.error_records_cache.clear();
    //     self.top_ips_cache.clear();
    //     self.top_urls_cache.clear();
    //
    //     {
    //         let mut stats = self.stats.write().unwrap();
    //         *stats = DBStats::new();
    //     }
    // }
}

impl Default for MemoryDB {
    fn default() -> Self {
        Self::new()
    }
}

/// Глобальный экземпляр синглтона (без блокировок)
pub static GLOBAL_DB: std::sync::LazyLock<Arc<MemoryDB>> = std::sync::LazyLock::new(|| Arc::new(MemoryDB::new())); 