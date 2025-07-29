use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Тип для возврата топ N записей
pub type TopEntries<'a> = Vec<(String, &'a LogEntry)>;

/// Параметры для обновления записи IP
#[derive(Debug, Clone)]
pub struct IpEntryParams {
    pub ip: String,
    pub log_line: String,
    pub timestamp: SystemTime,
    pub request_type: String,
    pub request_domain: String,
    pub status_code: Option<u16>,
    pub response_size: Option<u64>,
    pub response_time: Option<f64>,
    pub user_agent: Option<String>,
}

/// Параметры для обновления записи URL
#[derive(Debug, Clone)]
pub struct UrlEntryParams {
    pub url: String,
    pub log_line: String,
    pub timestamp: SystemTime,
    pub request_type: String,
    pub request_domain: String,
    pub full_url: String,
    pub status_code: Option<u16>,
    pub response_size: Option<u64>,
    pub response_time: Option<f64>,
    pub user_agent: Option<String>,
}

/// Параметры для добавления записи в лог
///
/// Эта структура инкапсулирует все необходимые данные для создания новой записи в логе.
/// Использование этой структуры вместо отдельных параметров делает код более читаемым
/// и уменьшает количество аргументов в функциях.
#[derive(Debug)]
pub struct LogEntryParams {
    /// IP-адрес источника запроса
    pub ip: String,
    /// URL-путь запроса
    pub url: String,
    /// Полная строка лога
    pub log_line: String,
    /// Временная метка запроса в формате Unix timestamp
    pub timestamp: i64,
    /// Тип HTTP-запроса (GET, POST, etc.)
    pub request_type: String,
    /// Домен запроса
    pub request_domain: String,
    /// HTTP-статус код ответа
    pub status_code: Option<u16>,
    /// Размер ответа в байтах
    pub response_size: Option<u64>,
    /// Время ответа в секундах
    pub response_time: Option<f64>,
    /// User-Agent клиента
    pub user_agent: Option<String>,
}

/// Запись лога для конкретного IP или URL
///
/// Содержит статистическую информацию о запросах, включая количество запросов,
/// последние запросы, типы запросов и дополнительную аналитическую информацию.
#[derive(Debug)]
pub struct LogEntry {
    pub count: usize,
    pub last_update: SystemTime,
    pub last_requests: Vec<String>,
    pub request_type: String,
    pub request_domain: String,
    pub full_url: String,
    // New fields for enhanced analysis
    pub status_codes: HashMap<u16, usize>,
    pub response_sizes: Vec<u64>,
    pub response_times: Vec<f64>,
    pub user_agents: HashMap<String, usize>,
    pub suspicious_patterns: Vec<String>,
    pub is_bot: bool,
    pub bot_type: Option<String>,
}

#[derive(Debug)]
pub struct SecurityData {
    pub suspicious_ips: HashMap<String, usize>,
    pub attack_patterns: HashMap<String, usize>,
    pub rate_limit_violations: HashMap<String, usize>,
}

#[derive(Debug)]
pub struct PerformanceData {
    pub avg_response_time: f64,
    pub max_response_time: f64,
    pub min_response_time: f64,
    pub total_response_size: u64,
    pub requests_per_second: f64,
    pub slow_requests: Vec<String>,
}

#[derive(Debug)]
pub struct ErrorData {
    pub error_codes: HashMap<u16, usize>,
    pub error_urls: HashMap<String, usize>,
    pub error_ips: HashMap<String, usize>,
    pub error_patterns: HashMap<String, usize>,
}

#[derive(Debug)]
pub struct BotData {
    pub bot_ips: HashMap<String, usize>,
    pub bot_types: HashMap<String, usize>,
    pub bot_user_agents: HashMap<String, usize>,
    pub bot_urls: HashMap<String, usize>,
}

#[derive(Debug)]
pub struct LogData {
    pub by_ip: HashMap<String, LogEntry>,
    pub by_url: HashMap<String, LogEntry>,
    pub total_requests: usize,
    pub requests_per_interval: HashMap<i64, usize>,
    // New data structures for enhanced analysis
    pub security: SecurityData,
    pub performance: PerformanceData,
    pub errors: ErrorData,
    pub bots: BotData,
    // Tab enablement flags
    pub enable_security: bool,
    pub enable_performance: bool,
    pub enable_errors: bool,
    pub enable_bots: bool,
    pub enable_sparkline: bool,
    pub enable_heatmap: bool,
}

impl LogData {
    /// Создает новый экземпляр LogData с указанными включенными табами
    ///
    /// # Arguments
    ///
    /// * `enable_security` - Включить анализ безопасности
    /// * `enable_performance` - Включить анализ производительности  
    /// * `enable_errors` - Включить анализ ошибок
    /// * `enable_bots` - Включить анализ ботов
    /// * `enable_sparkline` - Включить sparkline графики
    /// * `enable_heatmap` - Включить heatmap визуализацию
    ///
    /// # Returns
    ///
    /// Новый экземпляр LogData с настроенными флагами включения
    pub fn with_enabled_tabs(
        enable_security: bool,
        enable_performance: bool,
        enable_errors: bool,
        enable_bots: bool,
        enable_sparkline: bool,
        enable_heatmap: bool,
    ) -> Self {
        Self {
            by_ip: HashMap::new(),
            by_url: HashMap::new(),
            total_requests: 0,
            requests_per_interval: HashMap::new(),
            security: SecurityData {
                suspicious_ips: HashMap::new(),
                attack_patterns: HashMap::new(),
                rate_limit_violations: HashMap::new(),
            },
            performance: PerformanceData {
                avg_response_time: 0.0,
                max_response_time: 0.0,
                min_response_time: f64::MAX,
                total_response_size: 0,
                requests_per_second: 0.0,
                slow_requests: Vec::new(),
            },
            errors: ErrorData {
                error_codes: HashMap::new(),
                error_urls: HashMap::new(),
                error_ips: HashMap::new(),
                error_patterns: HashMap::new(),
            },
            bots: BotData {
                bot_ips: HashMap::new(),
                bot_types: HashMap::new(),
                bot_user_agents: HashMap::new(),
                bot_urls: HashMap::new(),
            },
            enable_security,
            enable_performance,
            enable_errors,
            enable_bots,
            enable_sparkline,
            enable_heatmap,
        }
    }

    /// Добавляет новую запись в лог
    ///
    /// Этот метод обрабатывает новую запись лога и обновляет все соответствующие
    /// структуры данных в зависимости от включенных табов. Метод автоматически
    /// обновляет статистику по IP и URL, а также выполняет дополнительный анализ
    /// если соответствующие табы включены.
    ///
    /// # Arguments
    ///
    /// * `params` - Параметры записи лога, содержащие все необходимые данные
    ///
    /// # Performance
    ///
    /// Метод оптимизирован для быстрой работы с большими объемами данных.
    /// Автоматическая очистка старых записей выполняется при достижении лимита.
    pub fn add_entry(&mut self, params: LogEntryParams) {
        self.total_requests += 1;
        let now = SystemTime::now();

        // Обновляем данные по IP
        self.update_ip_entry(IpEntryParams {
            ip: params.ip.clone(),
            log_line: params.log_line.clone(),
            timestamp: now,
            request_type: params.request_type.clone(),
            request_domain: params.request_domain.clone(),
            status_code: params.status_code,
            response_size: params.response_size,
            response_time: params.response_time,
            user_agent: params.user_agent.clone(),
        });

        // Обновляем данные по URL
        self.update_url_entry(UrlEntryParams {
            url: params.url.clone(),
            log_line: params.log_line.clone(),
            timestamp: now,
            request_type: params.request_type.clone(),
            request_domain: params.request_domain.clone(),
            full_url: params.url.clone(),
            status_code: params.status_code,
            response_size: params.response_size,
            response_time: params.response_time,
            user_agent: params.user_agent.clone(),
        });

        // Условное обновление дополнительных данных
        if self.enable_security {
            self.update_security_data(
                &params.ip,
                &params.url,
                &params.log_line,
                params.status_code,
            );
        }
        if self.enable_performance {
            self.update_performance_data(
                &params.ip,
                &params.url,
                params.response_time,
                params.response_size,
            );
        }
        if self.enable_errors {
            self.update_error_data(&params.ip, &params.url, params.status_code);
        }
        if self.enable_bots {
            self.update_bot_data(&params.ip, &params.url, params.user_agent);
        }
        if self.enable_sparkline || self.enable_heatmap {
            self.calculate_requests_per_second(params.timestamp);
            self.requests_per_interval.insert(
                params.timestamp,
                self.requests_per_interval
                    .get(&params.timestamp)
                    .unwrap_or(&0)
                    + 1,
            );
        }
    }

    fn update_ip_entry(&mut self, params: IpEntryParams) {
        let entry = self
            .by_ip
            .entry(params.ip.clone())
            .or_insert_with(|| LogEntry {
                count: 0,
                last_update: params.timestamp,
                last_requests: Vec::new(),
                request_type: params.request_type.clone(),
                request_domain: params.request_domain.clone(),
                full_url: String::new(),
                status_codes: HashMap::new(),
                response_sizes: Vec::new(),
                response_times: Vec::new(),
                user_agents: HashMap::new(),
                suspicious_patterns: Vec::new(),
                is_bot: false,
                bot_type: None,
            });

        entry.count += 1;
        entry.last_update = params.timestamp;

        // Add new request to the beginning of the list
        entry.last_requests.insert(0, params.log_line);

        // If the list exceeds 10 elements, remove the oldest one (last)
        if entry.last_requests.len() > 10 {
            entry.last_requests.pop();
        }

        // Update status codes
        if let Some(code) = params.status_code {
            *entry.status_codes.entry(code).or_insert(0) += 1;
        }

        // Update response sizes
        if let Some(size) = params.response_size {
            entry.response_sizes.push(size);
            if entry.response_sizes.len() > 100 {
                entry.response_sizes.remove(0);
            }
        }

        // Update response times
        if let Some(time) = params.response_time {
            entry.response_times.push(time);
            if entry.response_times.len() > 100 {
                entry.response_times.remove(0);
            }
        }

        // Update user agents
        if let Some(ua) = params.user_agent {
            *entry.user_agents.entry(ua).or_insert(0) += 1;
        }
    }

    fn update_url_entry(&mut self, params: UrlEntryParams) {
        let entry = self
            .by_url
            .entry(params.url.clone())
            .or_insert_with(|| LogEntry {
                count: 0,
                last_update: params.timestamp,
                last_requests: Vec::new(),
                request_type: params.request_type.clone(),
                request_domain: params.request_domain.clone(),
                full_url: params.full_url.clone(),
                status_codes: HashMap::new(),
                response_sizes: Vec::new(),
                response_times: Vec::new(),
                user_agents: HashMap::new(),
                suspicious_patterns: Vec::new(),
                is_bot: false,
                bot_type: None,
            });

        entry.count += 1;
        entry.last_update = params.timestamp;
        entry.full_url = params.full_url;

        // Add new request to the beginning of the list
        entry.last_requests.insert(0, params.log_line);

        // If the list exceeds 10 elements, remove the oldest one (last)
        if entry.last_requests.len() > 10 {
            entry.last_requests.pop();
        }

        // Update status codes
        if let Some(code) = params.status_code {
            *entry.status_codes.entry(code).or_insert(0) += 1;
        }

        // Update response sizes
        if let Some(size) = params.response_size {
            entry.response_sizes.push(size);
            if entry.response_sizes.len() > 100 {
                entry.response_sizes.remove(0);
            }
        }

        // Update response times
        if let Some(time) = params.response_time {
            entry.response_times.push(time);
            if entry.response_times.len() > 100 {
                entry.response_times.remove(0);
            }
        }

        // Update user agents
        if let Some(ua) = params.user_agent {
            *entry.user_agents.entry(ua).or_insert(0) += 1;
        }
    }

    fn update_security_data(
        &mut self,
        ip: &str,
        url: &str,
        log_line: &str,
        _status_code: Option<u16>,
    ) {
        // Расширенный список подозрительных паттернов
        let suspicious_patterns = [
            // SQL Injection
            "'",
            "union",
            "select",
            "drop",
            "insert",
            "update",
            "delete",
            "exec",
            "xp_",
            "sqlmap",
            "information_schema",
            "mysql",
            "oracle",
            "postgresql",
            "sqlite",
            // XSS (Cross-Site Scripting)
            "<script>",
            "javascript:",
            "onload=",
            "onerror=",
            "onclick=",
            "alert(",
            "document.cookie",
            "vbscript:",
            "expression(",
            "eval(",
            "setTimeout(",
            "setInterval(",
            // Path Traversal
            "../",
            "..\\",
            "/etc/",
            "/proc/",
            "c:\\",
            "windows\\",
            "~",
            "..%2f",
            "..%5c",
            "/etc/passwd",
            "/etc/shadow",
            "/proc/version",
            "/proc/cpuinfo",
            // Command Injection
            ";",
            "|",
            "&",
            "`",
            "$(",
            "eval(",
            "system(",
            "exec(",
            "shell_exec(",
            "passthru(",
            "proc_open(",
            "popen(",
            "curl_exec(",
            "file_get_contents(",
            // File Inclusion
            "include(",
            "require(",
            "include_once(",
            "require_once(",
            "fopen(",
            "file(",
            // Authentication Bypass
            "admin",
            "wp-admin",
            "phpmyadmin",
            "config",
            ".env",
            "backup",
            "test",
            "debug",
            "login",
            "auth",
            "administrator",
            "root",
            "user",
            "password",
            // Directory Traversal
            "dirb",
            "gobuster",
            "nikto",
            "nmap",
            "dirbuster",
            "wfuzz",
            // Common Attack Tools
            "sqlmap",
            "burp",
            "wireshark",
            "metasploit",
            "nmap",
            "nikto",
            // Suspicious Headers
            "x-forwarded-for",
            "x-real-ip",
            "x-forwarded-host",
            "x-original-url",
            // File Upload Attacks
            ".php",
            ".asp",
            ".aspx",
            ".jsp",
            ".jspx",
            ".cgi",
            ".pl",
            ".py",
            ".sh",
            "webshell",
            "shell",
            "cmd",
            "command",
            "exec",
            // Information Disclosure
            "robots.txt",
            "sitemap.xml",
            ".git",
            ".svn",
            ".htaccess",
            "web.config",
            "phpinfo",
            "info.php",
            "test.php",
            "debug.php",
            // Brute Force Indicators
            "login",
            "auth",
            "admin",
            "user",
            "password",
            "passwd",
            "wp-login",
            // Other Suspicious Patterns
            "null",
            "undefined",
            "NaN",
            "infinity",
            "true",
            "false",
            "0x",
            "0b",
            "0o",
            "\\x",
            "\\u",
            "\\n",
            "\\r",
            "\\t",
        ];

        let mut detected_patterns = Vec::new();

        // Проверяем URL и log_line на подозрительные паттерны
        for pattern in &suspicious_patterns {
            if url.to_lowercase().contains(pattern) || log_line.to_lowercase().contains(pattern) {
                detected_patterns.push(pattern.to_string());
            }
        }

        // Если найдены подозрительные паттерны
        if !detected_patterns.is_empty() {
            *self
                .security
                .suspicious_ips
                .entry(ip.to_string())
                .or_insert(0) += 1;

            // Добавляем каждый найденный паттерн в статистику
            for pattern in &detected_patterns {
                *self
                    .security
                    .attack_patterns
                    .entry(pattern.clone())
                    .or_insert(0) += 1;
            }

            // Добавляем паттерны в запись IP
            if let Some(entry) = self.by_ip.get_mut(ip) {
                for pattern in &detected_patterns {
                    if !entry.suspicious_patterns.contains(pattern) {
                        entry.suspicious_patterns.push(pattern.clone());
                    }
                }
            }
        }

        // Детект Rate Limiting violations (более 100 запросов в минуту)
        let current_time = SystemTime::now();
        let one_minute_ago = current_time - Duration::from_secs(60);

        if let Some(entry) = self.by_ip.get(ip) {
            if entry.count > 100 && entry.last_update > one_minute_ago {
                *self
                    .security
                    .rate_limit_violations
                    .entry(ip.to_string())
                    .or_insert(0) += 1;
            }
        }

        // Детект Brute Force атак (много запросов к auth endpoints)
        let auth_patterns = [
            "/login",
            "/auth",
            "/admin",
            "/wp-admin",
            "/wp-login",
            "/administrator",
        ];
        let is_auth_request = auth_patterns.iter().any(|pattern| url.contains(pattern));

        if is_auth_request {
            if let Some(entry) = self.by_ip.get(ip) {
                let auth_requests = entry
                    .last_requests
                    .iter()
                    .filter(|req| auth_patterns.iter().any(|pattern| req.contains(pattern)))
                    .count();
                if auth_requests > 10 {
                    *self
                        .security
                        .rate_limit_violations
                        .entry(ip.to_string())
                        .or_insert(0) += 1;
                }
            }
        }
    }

    fn update_performance_data(
        &mut self,
        _ip: &str,
        _url: &str,
        response_time: Option<f64>,
        response_size: Option<u64>,
    ) {
        if let Some(time) = response_time {
            if time > self.performance.max_response_time {
                self.performance.max_response_time = time;
            }
            if time < self.performance.min_response_time {
                self.performance.min_response_time = time;
            }

            // Update average response time
            let total_time = self
                .by_ip
                .values()
                .flat_map(|entry| &entry.response_times)
                .sum::<f64>();
            let total_count = self
                .by_ip
                .values()
                .map(|entry| entry.response_times.len())
                .sum::<usize>();

            if total_count > 0 {
                self.performance.avg_response_time = total_time / total_count as f64;
            }

            // Track slow requests (over 1 second)
            if time > 1.0 {
                // Add to performance slow requests list
                self.performance
                    .slow_requests
                    .push(format!("Response time: {:.2}s", time));
                if self.performance.slow_requests.len() > 100 {
                    self.performance.slow_requests.remove(0);
                }
            }
        }

        if let Some(size) = response_size {
            self.performance.total_response_size += size;
        }
    }

    fn update_error_data(&mut self, ip: &str, url: &str, status_code: Option<u16>) {
        if let Some(code) = status_code {
            if code >= 400 {
                *self.errors.error_codes.entry(code).or_insert(0) += 1;
                *self.errors.error_urls.entry(url.to_string()).or_insert(0) += 1;
                *self.errors.error_ips.entry(ip.to_string()).or_insert(0) += 1;

                // Categorize error patterns
                let error_pattern = match code {
                    400..=499 => "Client Error",
                    500..=599 => "Server Error",
                    _ => "Other Error",
                };
                *self
                    .errors
                    .error_patterns
                    .entry(error_pattern.to_string())
                    .or_insert(0) += 1;
            }
        }
    }

    fn update_bot_data(&mut self, ip: &str, url: &str, user_agent: Option<String>) {
        let bot_indicators = [
            "bot",
            "crawler",
            "spider",
            "scraper",
            "googlebot",
            "bingbot",
            "slurp",
            "teoma",
            "ia_archiver",
            "rogerbot",
            "exabot",
            "mj12bot",
        ];

        if let Some(ua) = user_agent {
            let ua_lower = ua.to_lowercase();
            for indicator in &bot_indicators {
                if ua_lower.contains(indicator) {
                    *self.bots.bot_ips.entry(ip.to_string()).or_insert(0) += 1;
                    *self
                        .bots
                        .bot_types
                        .entry(indicator.to_string())
                        .or_insert(0) += 1;
                    *self.bots.bot_user_agents.entry(ua.clone()).or_insert(0) += 1;
                    *self.bots.bot_urls.entry(url.to_string()).or_insert(0) += 1;

                    // Mark the IP as a bot in the main entry
                    if let Some(entry) = self.by_ip.get_mut(ip) {
                        entry.is_bot = true;
                        entry.bot_type = Some(indicator.to_string());
                    }
                    break;
                }
            }
        }
    }

    pub fn get_top_n(&self, n: usize) -> (TopEntries<'_>, TopEntries<'_>) {
        let mut top_ip = self.by_ip.iter().collect::<Vec<_>>();
        let mut top_url = self.by_url.iter().collect::<Vec<_>>();

        top_ip.sort_by(|a, b| b.1.count.cmp(&a.1.count));
        top_url.sort_by(|a, b| b.1.count.cmp(&a.1.count));

        (
            top_ip
                .into_iter()
                .take(n)
                .map(|(k, v)| (k.clone(), v))
                .collect(),
            top_url
                .into_iter()
                .take(n)
                .map(|(k, v)| (k.clone(), v))
                .collect(),
        )
    }

    pub fn get_unique_counts(&self) -> (usize, usize) {
        (self.by_ip.len(), self.by_url.len())
    }

    /// Возвращает общее количество запросов
    pub fn get_total_requests(&self) -> usize {
        self.total_requests
    }

    /// Возвращает общее количество обработанных строк
    pub fn get_total_lines(&self) -> usize {
        self.by_ip.values().map(|entry| entry.count).sum()
    }

    pub fn get_last_requests(&self, ip: &str) -> Vec<String> {
        self.by_ip
            .get(ip)
            .map_or(Vec::new(), |entry| entry.last_requests.clone())
    }

    // New methods for enhanced analysis
    pub fn get_security_summary(&self) -> (usize, usize, usize) {
        (
            self.security.suspicious_ips.len(),
            self.security.attack_patterns.len(),
            self.security.rate_limit_violations.len(),
        )
    }

    pub fn get_performance_summary(&self) -> (f64, f64, f64, u64) {
        let min_time = if self.performance.min_response_time == f64::MAX {
            0.0
        } else {
            self.performance.min_response_time
        };

        (
            self.performance.avg_response_time,
            self.performance.max_response_time,
            min_time,
            self.performance.total_response_size,
        )
    }

    pub fn get_error_summary(&self) -> (usize, usize, usize) {
        (
            self.errors.error_codes.len(),
            self.errors.error_urls.len(),
            self.errors.error_ips.len(),
        )
    }

    pub fn get_bot_summary(&self) -> (usize, usize, usize) {
        (
            self.bots.bot_ips.len(),
            self.bots.bot_types.len(),
            self.bots.bot_urls.len(),
        )
    }

    pub fn get_top_suspicious_ips(&self) -> Vec<(String, usize)> {
        let mut suspicious: Vec<_> = self.security.suspicious_ips.iter().collect();
        suspicious.sort_by(|a, b| b.1.cmp(a.1));
        suspicious
            .into_iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect()
    }

    pub fn get_top_error_codes(&self) -> Vec<(u16, usize)> {
        let mut errors: Vec<_> = self.errors.error_codes.iter().collect();
        errors.sort_by(|a, b| b.1.cmp(a.1));
        errors.into_iter().map(|(k, v)| (*k, *v)).collect()
    }

    pub fn get_top_bot_types(&self) -> Vec<(String, usize)> {
        let mut bots: Vec<_> = self.bots.bot_types.iter().collect();
        bots.sort_by(|a, b| b.1.cmp(a.1));
        bots.into_iter().map(|(k, v)| (k.clone(), *v)).collect()
    }

    pub fn get_slow_requests(&self) -> Vec<(String, f64)> {
        let mut slow_requests = Vec::new();
        for (ip, entry) in &self.by_ip {
            if let Some(max_time) = entry.response_times.iter().max_by(|a, b| {
                a.partial_cmp(b)
                    .expect("Response times should be comparable")
            }) {
                if *max_time > 1.0 {
                    // Changed threshold to 1.0
                    slow_requests.push((ip.clone(), *max_time));
                }
            }
        }
        slow_requests.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .expect("Response times should be comparable")
        });
        slow_requests
    }

    fn calculate_requests_per_second(&mut self, _timestamp: i64) {
        let now = SystemTime::now();
        let one_second_ago = now - Duration::from_secs(1);

        // Count requests in the last second
        let recent_requests = self
            .requests_per_interval
            .iter()
            .filter(|(&t, _)| {
                let request_time = SystemTime::UNIX_EPOCH + Duration::from_secs(t as u64);
                request_time >= one_second_ago
            })
            .map(|(_, &count)| count)
            .sum::<usize>();

        self.performance.requests_per_second = recent_requests as f64;
    }

    // New methods for enhanced functionality
    pub fn get_requests_per_second(&self) -> f64 {
        self.performance.requests_per_second
    }

    pub fn get_suspicious_patterns_for_ip(&self, ip: &str) -> Vec<String> {
        if let Some(entry) = self.by_ip.get(ip) {
            entry.suspicious_patterns.clone()
        } else {
            Vec::new()
        }
    }
}
