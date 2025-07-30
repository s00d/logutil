use std::collections::{HashMap};
use std::sync::{Arc, RwLock, atomic::{AtomicU64, Ordering}};
use std::time::{SystemTime};
use dashmap::DashMap;

/// Структура для хранения индексированных данных (оптимизированная)
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
    pub response_time: Option<f32>, // f32 вместо f64 для экономии памяти
    pub user_agent: Option<String>,
    pub log_line: String,
    #[allow(dead_code)]
    pub created_at: SystemTime,
}



/// Быстрая in-memory база данных с индексацией (оптимизированная версия)
#[derive(Debug)]
pub struct MemoryDB {
    // Основные данные (без блокировок)
    records: DashMap<u64, LogRecord>,
    next_id: AtomicU64,
    

    
    // Эффективные индексы - только ID, без дублирования данных
    ip_index: DashMap<String, Vec<u64>>,
    url_index: DashMap<String, Vec<u64>>,
    status_code_index: DashMap<u16, Vec<u64>>,
    
    // Специализированные кэши безопасности (только для ошибок)
    error_records_cache: DashMap<u64, bool>,
    

    
    // Статистика (с блокировкой только для статистики)
    stats: Arc<RwLock<DBStats>>,
    
    // Настройки оптимизации
    max_records: usize,
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
            status_code_index: DashMap::new(),
            error_records_cache: DashMap::new(),

            stats: Arc::new(RwLock::new(DBStats::new())),
            max_records: 10_000_000, // Максимум 10M записей
        }
    }

    /// Добавляет новую запись в базу данных (оптимизированная версия)
    pub fn insert(&self, record: LogRecord) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);

        // Проверяем лимит записей
        if self.records.len() >= self.max_records {
            // Удаляем старые записи (FIFO)
            self.evict_old_records();
        }

        // Добавляем запись в основную таблицу
        self.records.insert(id, record.clone());

        // Обновляем только критически важные индексы
        self.update_critical_indexes(id, &record);
        self.update_stats(&record);

        id
    }

    /// Удаляет старые записи для контроля памяти
    fn evict_old_records(&self) {
        let target_size = self.max_records / 2; // Удаляем половину
        let mut to_remove = Vec::new();
        
        // Собираем ID старых записей
        for entry in self.records.iter() {
            if to_remove.len() >= target_size {
                break;
            }
            to_remove.push(*entry.key());
        }
        
        // Удаляем записи и их индексы
        for id in to_remove {
            if let Some(record) = self.records.remove(&id) {
                self.remove_from_indexes(id, &record.1);
            }
        }
    }

    /// Обновляет только критически важные индексы
    fn update_critical_indexes(&self, id: u64, record: &LogRecord) {
        // Обновляем только основные индексы
        self.ip_index.entry(record.ip.clone()).or_insert_with(|| Vec::with_capacity(4)).push(id);
        self.url_index.entry(record.url.clone()).or_insert_with(|| Vec::with_capacity(4)).push(id);
        
        if let Some(status_code) = record.status_code {
            self.status_code_index.entry(status_code).or_insert_with(|| Vec::with_capacity(4)).push(id);
            
            // Обновляем кэш ошибок только для ошибок
            if status_code >= 400 {
                self.error_records_cache.insert(id, true);
            }
        }
        

    }

    /// Удаляет запись из всех индексов
    fn remove_from_indexes(&self, id: u64, record: &LogRecord) {
        // Удаляем из основных индексов
        if let Some(mut ids) = self.ip_index.get_mut(&record.ip) {
            ids.retain(|&x| x != id);
        }
        if let Some(mut ids) = self.url_index.get_mut(&record.url) {
            ids.retain(|&x| x != id);
        }
        if let Some(status_code) = record.status_code {
            if let Some(mut ids) = self.status_code_index.get_mut(&status_code) {
                ids.retain(|&x| x != id);
            }
        }
        
        // Удаляем из кэша ошибок
        self.error_records_cache.remove(&id);
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
            let n = stats.total_records as f64;
            stats.avg_response_time = (stats.avg_response_time * (n - 1.0) + response_time as f64) / n;
        }
    }

    /// Поиск записей по IP (оптимизированная версия)
    pub fn find_by_ip(&self, ip: &str) -> Vec<LogRecord> {
        if let Some(ids) = self.ip_index.get(ip) {
            ids.iter()
                .take(1000) // Ограничиваем результаты для скорости
                .filter_map(|id| self.records.get(id).map(|r| r.clone()))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Поиск записей по URL (оптимизированная версия)
    pub fn find_by_url(&self, url: &str) -> Vec<LogRecord> {
        if let Some(ids) = self.url_index.get(url) {
            ids.iter()
                .take(1000) // Ограничиваем результаты для скорости
                .filter_map(|id| self.records.get(id).map(|r| r.clone()))
                .collect()
        } else {
            Vec::new()
        }
    }
    /// Получение топ IP адресов (оптимизированная версия)
    pub fn get_top_ips(&self, limit: usize) -> Vec<(String, usize)> {
        // Вычисляем топ IP
        let mut ip_counts: Vec<(String, usize)> = Vec::new();
        let max_items = std::cmp::min(self.ip_index.len(), 1000);
        
        for (i, entry) in self.ip_index.iter().enumerate() {
            if i >= max_items {
                break;
            }
            ip_counts.push((entry.key().clone(), entry.value().len()));
        }
        
        ip_counts.sort_by(|a, b| b.1.cmp(&a.1));
        ip_counts.truncate(limit);
        ip_counts
    }

    /// Получение топ URL (оптимизированная версия)
    pub fn get_top_urls(&self, limit: usize) -> Vec<(String, usize)> {
        // Вычисляем топ URL
        let mut url_counts: Vec<(String, usize)> = Vec::new();
        let max_items = std::cmp::min(self.url_index.len(), 1000);
        
        for (i, entry) in self.url_index.iter().enumerate() {
            if i >= max_items {
                break;
            }
            url_counts.push((entry.key().clone(), entry.value().len()));
        }
        
        url_counts.sort_by(|a, b| b.1.cmp(&a.1));
        url_counts.truncate(limit);
        url_counts
    }

    /// Получение статистики
    pub fn get_stats(&self) -> DBStats {
        let mut stats = self.stats.write().unwrap();
        
        stats.unique_ips = self.ip_index.len();
        stats.unique_urls = self.url_index.len();
        stats.unique_domains = 0; // Ленивый подсчет
        
        stats.clone()
    }

    /// Получение статистики ошибок (оптимизированная версия)
    pub fn get_error_stats(&self) -> (usize, usize, usize) {
        let error_codes_count = self.status_code_index.iter()
            .filter(|entry| {
                let status_code = *entry.key();
                status_code >= 400
            })
            .map(|entry| entry.value().len())
            .sum();

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

    /// Получение топ статус кодов (оптимизированная версия)
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

    /// Получение статистики времени ответа (оптимизированная версия)
    pub fn get_response_time_stats(&self) -> (f64, f64, f64) {
        let mut times: Vec<f64> = Vec::new();
        let max_samples = 5000; // Уменьшенный лимит
        
        for entry in self.records.iter() {
            if times.len() >= max_samples {
                break;
            }
            if let Some(response_time) = entry.response_time {
                times.push(response_time as f64);
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

    /// Получение медленных запросов (оптимизированная версия)
    pub fn get_slow_requests_with_limit(&self, threshold: f64, limit: usize) -> Vec<(String, f64)> {
        let mut slow_requests: Vec<(String, f64)> = Vec::new();
        let max_scan = 25000; // Уменьшенный лимит
        let mut scanned = 0;
        
        for entry in self.records.iter() {
            if scanned >= max_scan {
                break;
            }
            
            if let Some(response_time) = entry.response_time {
                if response_time > threshold as f32 {
                    slow_requests.push((entry.ip.clone(), response_time as f64));
                }
            }
            scanned += 1;
        }
        
        slow_requests.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        slow_requests.truncate(limit);
        slow_requests
    }

    /// Получение запросов в секунду
    pub fn get_requests_per_second(&self) -> f64 {
        let stats = self.get_stats();
        if stats.total_requests == 0 {
            return 0.0;
        }
        
        stats.total_requests as f64 / 60.0
    }

    /// Получение статистики ботов (оптимизированная версия)
    pub fn get_bot_stats(&self) -> (usize, usize, usize) {
        let mut bot_ips = std::collections::HashSet::new();
        let mut bot_urls = std::collections::HashSet::new();
        let mut bot_types_count = 0;
        
        // Простой подсчет через User-Agent в записях
        for entry in self.records.iter() {
            if let Some(ref user_agent) = entry.user_agent {
                let user_agent_lower = user_agent.to_lowercase();
                if user_agent_lower.contains("bot") || user_agent_lower.contains("crawler") || user_agent_lower.contains("spider") {
                    bot_types_count += 1;
                    bot_ips.insert(entry.ip.clone());
                    bot_urls.insert(entry.url.clone());
                }
            }
        }
        
        (bot_ips.len(), bot_types_count, bot_urls.len())
    }

    /// Получение топ User-Agent (оптимизированная версия)
    pub fn get_top_user_agents(&self, limit: usize) -> Vec<(String, usize)> {
        let mut user_agent_counts: HashMap<String, usize> = HashMap::new();
        
        for entry in self.records.iter() {
            if let Some(ref user_agent) = entry.user_agent {
                *user_agent_counts.entry(user_agent.clone()).or_insert(0) += 1;
            }
        }
        
        let mut result: Vec<(String, usize)> = user_agent_counts.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1));
        result.truncate(limit);
        result
    }

    /// Получение статистики по временным интервалам (оптимизированная версия)
    pub fn get_time_series_data(&self, interval_seconds: i64) -> Vec<(i64, usize)> {
        let mut interval_counts: HashMap<i64, usize> = HashMap::new();
        
        let max_entries = 5000; // Уменьшенный лимит
        let mut processed = 0;
        
        for entry in self.records.iter() {
            if processed >= max_entries {
                break;
            }
            
            let timestamp = entry.timestamp;
            let interval = timestamp / interval_seconds;
            *interval_counts.entry(interval).or_insert(0) += 1;
            processed += 1;
        }
        
        let mut result: Vec<(i64, usize)> = interval_counts.into_iter().collect();
        result.sort_by(|a, b| a.0.cmp(&b.0));
        result
    }

    /// Очистка базы данных
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.records.clear();
        self.next_id.store(1, Ordering::Relaxed);
        self.ip_index.clear();
        self.url_index.clear();
        self.status_code_index.clear();
        self.error_records_cache.clear();

        {
            let mut stats = self.stats.write().unwrap();
            *stats = DBStats::new();
        }
    }

    /// Устанавливает максимальное количество записей
    pub fn set_max_records(&mut self, max_records: usize) {
        self.max_records = max_records;
    }



    /// Получает текущее использование памяти (приблизительно)
    pub fn get_memory_usage(&self) -> usize {
        let records_size = self.records.len() * std::mem::size_of::<LogRecord>();
        let ip_index_size = self.ip_index.len() * 64; // Приблизительно
        let url_index_size = self.url_index.len() * 128; // Приблизительно
        let status_index_size = self.status_code_index.len() * 32; // Приблизительно
        let error_cache_size = self.error_records_cache.len() * 16; // Приблизительно
        
        records_size + ip_index_size + url_index_size + status_index_size + error_cache_size
    }

    /// Получает количество записей в базе данных
    pub fn get_records_count(&self) -> usize {
        self.records.len()
    }
    /// Получение всех записей (оптимизированная версия)
    pub fn get_all_records(&self) -> Vec<LogRecord> {
        self.records.iter().map(|entry| entry.value().clone()).collect()
    }

    /// Анализ безопасности - подозрительные IP (оптимизированная версия)
    pub fn get_suspicious_ips(&self) -> Vec<(String, usize)> {
        // Простая реализация через поиск по IP с ошибками
        let mut suspicious_ips: HashMap<String, usize> = HashMap::new();
        
        for entry in self.error_records_cache.iter() {
            if let Some(record) = self.records.get(entry.key()) {
                *suspicious_ips.entry(record.ip.clone()).or_insert(0) += 1;
            }
        }
        
        let mut result: Vec<(String, usize)> = suspicious_ips.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1));
        result.truncate(10);
        result
    }

    /// Анализ безопасности - паттерны атак (оптимизированная версия)
    pub fn get_attack_patterns(&self) -> Vec<(String, usize)> {
        // Простая реализация через поиск паттернов в записях с ошибками
        let mut patterns: HashMap<String, usize> = HashMap::new();
        
        for entry in self.error_records_cache.iter() {
            if let Some(record) = self.records.get(entry.key()) {
                let log_line = record.log_line.to_lowercase();
                let attack_patterns = [
                    "admin", "wp-admin", "phpmyadmin", "config", "backup",
                    "union select", "drop table", "insert into", "delete from",
                    "script", "javascript", "eval(", "document.cookie",
                    "..", "~", "etc/passwd", "/proc/", "/sys/",
                ];
                
                for pattern in &attack_patterns {
                    if log_line.contains(pattern) {
                        *patterns.entry(pattern.to_string()).or_insert(0) += 1;
                    }
                }
            }
        }
        
        let mut result: Vec<(String, usize)> = patterns.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1));
        result.truncate(10);
        result
    }

    /// Получение подозрительных паттернов для IP (оптимизированная версия)
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
}

impl Default for MemoryDB {
    fn default() -> Self {
        Self::new()
    }
}

/// Глобальный экземпляр синглтона (оптимизированная версия)
pub static GLOBAL_DB: std::sync::LazyLock<Arc<MemoryDB>> = std::sync::LazyLock::new(|| Arc::new(MemoryDB::new())); 