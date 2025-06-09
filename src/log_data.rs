use std::collections::HashMap;
use std::time::{Duration, SystemTime};

pub struct LogEntry {
    pub count: usize,
    pub last_update: SystemTime,
    pub last_requests: Vec<String>,
    pub request_type: String,
    pub request_domain: String,
}

pub struct LogData {
    pub by_ip: HashMap<String, LogEntry>,
    pub by_url: HashMap<String, LogEntry>,
    pub total_requests: usize,
    pub requests_per_interval: HashMap<i64, usize>,
}

impl LogData {
    pub fn new() -> Self {
        Self {
            by_ip: HashMap::new(),
            by_url: HashMap::new(),
            total_requests: 0,
            requests_per_interval: HashMap::new(),
        }
    }

    pub fn add_entry(
        &mut self,
        ip: String,
        url: String,
        log_line: String,
        timestamp: i64,
        request_type: String,
        request_domain: String,
        no_clear: bool
    ) {
        let now = SystemTime::now();

        self.update_ip_entry(ip, log_line.clone(), now, request_type.clone(), request_domain.clone());
        self.update_url_entry(url, log_line, now, request_type, request_domain);

        self.total_requests += 1;

        if self.by_ip.len() > 10000 && !no_clear {
            self.clear_outdated_entries();
        }

        *self.requests_per_interval.entry(timestamp).or_insert(0) += 1;

        self.remove_outdated_intervals(timestamp);

    }

    fn update_ip_entry(
        &mut self,
        ip: String,
        log_line: String,
        now: SystemTime,
        request_type: String,
        request_domain: String,
    ) {
        let entry = self.by_ip.entry(ip).or_insert_with(|| LogEntry {
            count: 0,
            request_type: request_type.clone(),
            request_domain: request_domain.clone(),
            last_update: now,
            last_requests: Vec::new(),
        });

        entry.count += 1;
        entry.last_update = now;
        
        // Add new request to the beginning of the list
        entry.last_requests.insert(0, log_line);
        
        // If the list exceeds 10 elements, remove the oldest one (last)
        if entry.last_requests.len() > 10 {
            entry.last_requests.pop();
        }
    }

    fn update_url_entry(
        &mut self,
        url: String,
        log_line: String,
        now: SystemTime,
        request_type: String,
        request_domain: String,
    ) {
        let entry = self.by_url.entry(url).or_insert_with(|| LogEntry {
            count: 0,
            request_type: request_type.clone(),
            request_domain: request_domain.clone(),
            last_update: now,
            last_requests: Vec::new(),
        });

        entry.count += 1;
        entry.last_update = now;
        
        // Add new request to the beginning of the list
        entry.last_requests.insert(0, log_line);
        
        // If the list exceeds 10 elements, remove the oldest one (last)
        if entry.last_requests.len() > 10 {
            entry.last_requests.pop();
        }
    }

    pub fn clear_outdated_entries(&mut self) {
        let threshold = SystemTime::now() - Duration::from_secs(1200);
        self.by_ip.retain(|_, entry| entry.last_update >= threshold);
        self.by_url.retain(|_, entry| entry.last_update >= threshold);
    }

    fn remove_outdated_intervals(&mut self, _timestamp: i64) {
        // let threshold = timestamp - (20 * 60); // 20 minutes ago
        // self.requests_per_interval.retain(|&k, _| k >= threshold);
    }

    pub fn get_top_n(&self, n: usize) -> (Vec<(String, &LogEntry)>, Vec<(String, &LogEntry)>) {
        let mut top_ip = self.by_ip.iter().collect::<Vec<_>>();
        let mut top_url = self.by_url.iter().collect::<Vec<_>>();

        top_ip.sort_by(|a, b| b.1.count.cmp(&a.1.count));
        top_url.sort_by(|a, b| b.1.count.cmp(&a.1.count));

        (
            top_ip.into_iter().take(n).map(|(k, v)| (k.clone(), v)).collect(),
            top_url.into_iter().take(n).map(|(k, v)| (k.clone(), v)).collect(),
        )
    }

    pub fn get_unique_counts(&self) -> (usize, usize) {
        (self.by_ip.len(), self.by_url.len())
    }

    pub fn get_last_requests(&self, ip: &str) -> Vec<String> {
        self.by_ip.get(ip).map_or(Vec::new(), |entry| entry.last_requests.clone())
    }
}