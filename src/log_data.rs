use std::collections::HashMap;
use std::time::{Duration, SystemTime};

pub struct LogEntry {
    pub(crate) count: usize,
    pub(crate) last_update: SystemTime,
    pub(crate) last_requests: Vec<String>,
    pub(crate) request_type: String,
    pub(crate) request_domain: String,
}

pub struct LogData {
    pub(crate) by_ip: HashMap<String, LogEntry>,
    by_url: HashMap<String, LogEntry>,
    pub(crate) total_requests: usize,
    pub(crate) requests_per_interval: HashMap<i64, usize>,
}

impl LogData {
    pub(crate) fn new() -> Self {
        Self {
            by_ip: HashMap::new(),
            by_url: HashMap::new(),
            total_requests: 0,
            requests_per_interval: HashMap::new(),
        }
    }

    pub(crate) fn add_entry(
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
        entry.last_requests.push(log_line);
        if entry.last_requests.len() > 10 {
            entry.last_requests.remove(0);
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
        entry.last_requests.push(log_line);
        if entry.last_requests.len() > 10 {
            entry.last_requests.remove(0);
        }
    }

    fn clear_outdated_entries(&mut self) {
        let threshold = SystemTime::now() - Duration::from_secs(1200);
        self.by_ip.retain(|_, entry| entry.last_update >= threshold);
        self.by_url.retain(|_, entry| entry.last_update >= threshold);
    }

    fn remove_outdated_intervals(&mut self, _timestamp: i64) {
        // let threshold = timestamp - (20 * 60); // 20 minutes ago
        // self.requests_per_interval.retain(|&k, _| k >= threshold);
    }

    pub(crate) fn get_top_n(&self, n: usize) -> (Vec<(String, &LogEntry)>, Vec<(String, &LogEntry)>) {
        let mut top_ip = self.by_ip.iter().collect::<Vec<_>>();
        let mut top_url = self.by_url.iter().collect::<Vec<_>>();

        top_ip.sort_by(|a, b| b.1.count.cmp(&a.1.count));
        top_url.sort_by(|a, b| b.1.count.cmp(&a.1.count));

        (
            top_ip.into_iter().take(n).map(|(k, v)| (k.clone(), v)).collect(),
            top_url.into_iter().take(n).map(|(k, v)| (k.clone(), v)).collect(),
        )
    }

    pub(crate) fn get_unique_counts(&self) -> (usize, usize) {
        (self.by_ip.len(), self.by_url.len())
    }

    pub(crate) fn get_last_requests(&self, ip: &str) -> Vec<String> {
        self.by_ip.get(ip).map_or(Vec::new(), |entry| entry.last_requests.clone())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
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
        assert_eq!(last_requests[0], log_line1);
        assert_eq!(last_requests[1], log_line2);
    }

    #[test]
    fn test_clear_outdated_entries() {
        let mut log_data = LogData::new();
        let ip = "192.168.0.1".to_string();
        let url = "http://example.com/page1".to_string();
        let log_line = "GET /page1 HTTP/1.1".to_string();

        let old_time = SystemTime::now() - Duration::from_secs(3600); // 1 hour ago
        let new_time = SystemTime::now();

        // Manually set the last_update time for the log entries
        log_data.by_ip.insert(ip.clone(), LogEntry {
            count: 1,
            last_update: old_time,
            last_requests: vec![log_line.clone()],
            request_type: "GET".to_string(),
            request_domain: "example.com".to_string(),
        });
        log_data.by_url.insert(url.clone(), LogEntry {
            count: 1,
            last_update: old_time,
            last_requests: vec![log_line.clone()],
            request_type: "GET".to_string(),
            request_domain: "example.com".to_string(),
        });

        // Clear outdated entries
        log_data.clear_outdated_entries();

        // Check if outdated entries are removed
        assert_eq!(log_data.by_ip.len(), 0);
        assert_eq!(log_data.by_url.len(), 0);

        // Add new entry with current time
        log_data.by_ip.insert(ip.clone(), LogEntry {
            count: 1,
            last_update: new_time,
            last_requests: vec![log_line.clone()],
            request_type: "GET".to_string(),
            request_domain: "example.com".to_string(),
        });
        log_data.by_url.insert(url.clone(), LogEntry {
            count: 1,
            last_update: new_time,
            last_requests: vec![log_line],
            request_type: "GET".to_string(),
            request_domain: "example.com".to_string(),
        });

        // Clear outdated entries again
        log_data.clear_outdated_entries();

        // Check if current entries are kept
        assert_eq!(log_data.by_ip.len(), 1);
        assert_eq!(log_data.by_url.len(), 1);
    }

    // #[test]
    // fn test_remove_outdated_intervals() {
    //     let mut log_data = LogData::new();
    //     let ip = "192.168.0.1".to_string();
    //     let url = "http://example.com/page1".to_string();
    //     let log_line = "GET /page1 HTTP/1.1".to_string();
    //     let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
    //     let old_timestamp = timestamp - (20 * 60 + 1); // 20 minutes and 1 second ago
    //
    //     log_data.add_entry(ip.clone(), url.clone(), log_line.clone(), timestamp, "GET".to_string(), "example.com".to_string(), false);
    //     log_data.add_entry(ip.clone(), url.clone(), log_line.clone(), old_timestamp, "GET".to_string(), "example.com".to_string(), false);
    //
    //     log_data.remove_outdated_intervals(timestamp);
    //
    //     assert_eq!(log_data.requests_per_interval.len(), 1);
    //     assert!(log_data.requests_per_interval.contains_key(&timestamp));
    //     assert!(!log_data.requests_per_interval.contains_key(&old_timestamp));
    // }
}