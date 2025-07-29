use std::io::Write;

/// Форматирует длительность в читаемый вид
pub fn format_duration(duration: std::time::Duration) -> String {
    let total_secs = duration.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

/// Структура для отображения прогресса
pub struct ProgressBar {
    start_time: std::time::Instant,
    last_update_time: std::time::Instant,
    last_progress: f64,
    bar_width: usize,
    total_lines: usize,
    processed_lines: usize,
}

impl ProgressBar {
    pub fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
            last_update_time: std::time::Instant::now(),
            last_progress: 0.0,
            bar_width: 50,
            total_lines: 0,
            processed_lines: 0,
        }
    }

    pub fn set_total_lines(&mut self, total: usize) {
        self.total_lines = total;
    }

    pub fn update_processed_lines(&mut self, processed: usize) {
        self.processed_lines = processed;
        let progress = if self.total_lines > 0 {
            (processed as f64 / self.total_lines as f64) * 100.0
        } else {
            0.0
        };
        self.update(progress);
    }

    pub fn update(&mut self, progress: f64) {
        let now = std::time::Instant::now();
        let time_since_last_update = now.duration_since(self.last_update_time);
        
        // Обновляем прогресс только если прошло достаточно времени или прогресс изменился значительно
        if time_since_last_update.as_millis() > 100 || (progress - self.last_progress).abs() > 1.0 {
            self.draw_progress_bar(progress, "Processing");
            self.last_update_time = now;
            self.last_progress = progress;
        }
    }

    fn draw_progress_bar(&self, progress: f64, text: &str) {
        let filled_width = ((progress / 100.0) * self.bar_width as f64) as usize;
        let empty_width = self.bar_width - filled_width;
        
        let filled = "█".repeat(filled_width);
        let empty = "░".repeat(empty_width);
        
        let elapsed = self.start_time.elapsed();
        let _elapsed_str = format_duration(elapsed);
        
        // Рассчитываем примерное время до завершения
        let estimated_total = if progress > 0.0 {
            elapsed.as_secs_f64() * 100.0 / progress
        } else {
            0.0
        };
        let remaining = if estimated_total > elapsed.as_secs_f64() {
            estimated_total - elapsed.as_secs_f64()
        } else {
            0.0
        };
        let remaining_str = format_duration(std::time::Duration::from_secs_f64(remaining));
        
        // Рассчитываем скорость обработки
        let rate = if elapsed.as_secs() > 0 {
            self.processed_lines as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };
        
        // Очищаем строку и перемещаем курсор в начало
        print!("\r");
        
        // Выводим подробный прогресс-бар
        print!(
            "{} [{}{}] {}% ({}/{}) {:.1} lines/s ETA: {}",
            text,
            filled,
            empty,
            progress as i32,
            self.processed_lines,
            self.total_lines,
            rate,
            remaining_str
        );
        
        // Очищаем остаток строки
        print!("{}", " ".repeat(20));
        
        // Принудительно выводим буфер
        std::io::stdout().flush().unwrap();
    }
} 