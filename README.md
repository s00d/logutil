![Logo](https://github.com/s00d/logutil/blob/main/assets/logo.png?raw=true)

# LogUtil - Real-time Nginx Log Analyzer

<div align="center">
    <a href="https://crates.io/crates/logutil" target="_blank">
        <img src="https://img.shields.io/crates/v/logutil?style=for-the-badge" alt="crates.io version">
    </a>
    <a href="https://crates.io/crates/logutil" target="_blank">
        <img src="https://img.shields.io/crates/d/logutil?style=for-the-badge" alt="crates.io downloads">
    </a>
    <a href="https://github.com/s00d/logutil/blob/master/LICENSE" target="_blank">
        <img src="https://img.shields.io/crates/l/logutil?style=for-the-badge" alt="crates.io license">
    </a>
    <a href="https://github.com/s00d/logutil" target="_blank">
        <img src="https://img.shields.io/github/stars/s00d/logutil?style=for-the-badge" alt="GitHub stars">
    </a>
</div>

## Overview

**LogUtil** is a powerful, real-time log analysis tool written in Rust that provides an interactive terminal interface for monitoring and analyzing Nginx access logs. It offers comprehensive log parsing, real-time monitoring, and detailed analytics with a beautiful TUI (Terminal User Interface).

![LogUtil in action](https://github.com/s00d/logutil/blob/main/assets/img.gif?raw=true)

## Key Features

### 🔍 **Real-time Log Monitoring**
- Live tail functionality that monitors log files as they grow
- Automatic detection and processing of new log entries
- Real-time updates without manual refresh

### 📊 **Comprehensive Analytics**
- **IP Address Analysis**: Track requests by IP addresses with detailed statistics
- **URL Analysis**: Monitor most accessed URLs and their patterns
- **Request Type Tracking**: Categorize requests by HTTP methods (GET, POST, etc.)
- **Domain Analysis**: Track requests by domain names
- **Time-based Analytics**: Analyze request patterns over time

### 🎨 **Interactive TUI Interface**
- **9 Main Tabs**: Overview, Requests, Detailed, Sparkline, Heatmap, Security, Performance, Errors, and Bots
- **Overview Tab**: Shows top IPs and URLs with real-time statistics
- **Requests Tab**: Searchable log entries with pagination
- **Detailed Tab**: Drill-down view for specific IP addresses
- **Sparkline Tab**: Real-time request timeline visualization
- **Heatmap Tab**: Hourly request patterns across multiple days
- **Security Tab**: Detection of suspicious IPs and attack patterns
- **Performance Tab**: Response time analysis and slow request identification
- **Errors Tab**: HTTP error code analysis and categorization
- **Bots Tab**: Bot and crawler detection and classification

### ⚡ **Performance Optimizations**
- Efficient memory management with automatic cleanup
- Optimized for large log files (handles 10,000+ entries)
- Minimal CPU usage during real-time monitoring
- Configurable cleanup policies

### 🔧 **Flexible Configuration**
- Customizable regex patterns for different log formats
- Support for various date formats
- Configurable top N entries display
- Optional automatic cleanup of outdated entries

### 📁 **Interactive File Selection Mode**
When you run LogUtil without specifying a log file, it launches an interactive file selection mode:

**File Selector Features:**
- **📂 Directory Navigation**: Browse through folders with intuitive navigation
- **📄 File Browser**: View all files with clear icons (📁 for folders, 📄 for files, ⬆️ for parent directory)
- **🔍 Visual Selection**: Highlighted current selection with clear visual feedback
- **⚡ Quick Access**: Navigate with arrow keys and select with Enter

**Settings Configuration:**
After selecting a log file, you'll see an interactive settings screen where you can configure:
- **📊 Analysis Parameters**: Enable/disable specific analysis tabs (Security, Performance, Errors, Bots, Sparkline, Heatmap)
- **🔧 Processing Options**: Set count limits, regex patterns, date formats
- **⚙️ Real-time Settings**: Configure cleanup policies and monitoring options
- **▶️ Start Analysis**: Launch the analysis with your custom configuration

**Usage:**
```bash
# Launch interactive mode
logutil

# Or specify a file directly
logutil /path/to/access.log
```

### 🎮 **Keyboard Shortcuts**

**Navigation:**
- **Tab** / **T**: Switch to next tab
- **Shift+Tab** / **Shift+T**: Switch to previous tab
- **↑/↓**: Navigate through lists and options
- **←/→**: Switch between panels (in tabs with multiple panels)
- **Enter**: Select item or toggle boolean values
- **Esc**: Go back or cancel

**File Selector:**
- **↑/↓**: Navigate through files and folders
- **Enter**: Select file or enter folder
- **Esc**: Go back to parent directory or exit

**Settings:**
- **↑/↓**: Navigate through settings
- **Enter**: Edit setting or toggle boolean values
- **Esc**: Cancel editing or go back

**TUI Controls:**
- **Q** / **Ctrl+C**: Quit application
- **Enter**: Copy selected item to clipboard (Overview tab)

### 📋 **Command Line Examples**

**Interactive mode with pre-configured parameters:**
```bash
# Launch file selector with all analysis tabs enabled
cargo run "" --enable-security --enable-performance --enable-errors --enable-bots --enable-sparkline --enable-heatmap --count=1000

# Launch with custom settings
cargo run "" --enable-security --enable-performance --top=20 --count=500

# Launch with specific analysis tabs only
cargo run "" --enable-security --enable-errors --count=2000
```

**Direct file analysis with all features:**
```bash
# Analyze with all tabs enabled
logutil access.log --enable-security --enable-performance --enable-errors --enable-bots --enable-sparkline --enable-heatmap --count=1000

# Security-focused analysis
logutil access.log --enable-security --enable-errors --top=50

# Performance monitoring
logutil access.log --enable-performance --enable-sparkline --enable-heatmap
```

## Installation

### Quick Install (Recommended)

Download the latest release for your platform:

**Linux (x86_64):**
```bash
curl -L -o /usr/local/bin/logutil https://github.com/s00d/logutil/releases/latest/download/logutil-x86_64-unknown-linux-gnu
chmod +x /usr/local/bin/logutil
```

**Linux (ARM64):**
```bash
curl -L -o /usr/local/bin/logutil https://github.com/s00d/logutil/releases/latest/download/logutil-aarch64-unknown-linux-gnu
chmod +x /usr/local/bin/logutil
```

**macOS:**
```bash
curl -L -o /usr/local/bin/logutil https://github.com/s00d/logutil/releases/latest/download/logutil-x86_64-apple-darwin
chmod +x /usr/local/bin/logutil
```

### Build from Source

1. **Install Rust:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rust-lang.org | sh
```

2. **Clone and Build:**
```bash
git clone https://github.com/s00d/logutil.git
cd logutil
cargo build --release
```

3. **Install:**
```bash
sudo cp target/release/logutil /usr/local/bin/
```

## Usage

### Basic Usage

**Monitor a log file in real-time:**
```bash
logutil /var/log/nginx/access.log
```

**Analyze the entire log file:**
```bash
logutil /var/log/nginx/access.log --count=-1
```

**Show only the last 1000 lines:**
```bash
logutil /var/log/nginx/access.log --count=1000
```

### Advanced Usage

**Custom regex pattern:**
```bash
logutil /var/log/nginx/access.log --regex='^(\S+) - - \[(.*?)\] "(\S+) (\S+) HTTP/\d+\.\d+" (\d+) (\d+) "([^"]*)" "([^"]*)"$'
```

**Show top 20 entries:**
```bash
logutil /var/log/nginx/access.log --top=20
```

**Disable automatic cleanup:**
```bash
logutil /var/log/nginx/access.log --no-clear
```

**Custom date format:**
```bash
logutil /var/log/nginx/access.log --date-format="%d/%b/%Y:%H:%M:%S %z"
```

**Load regex from file:**
```bash
logutil /var/log/nginx/access.log --regex=/path/to/regex.txt
```

### Tab Management

By default, only the core tabs (Overview, Requests, Detailed) are enabled. Additional tabs can be enabled using command-line flags:

**Enable Security tab (detect suspicious activity, attacks, etc.):**
```bash
logutil /var/log/nginx/access.log --enable-security
```

**Enable Performance tab (monitor response times, slow requests):**
```bash
logutil /var/log/nginx/access.log --enable-performance
```

**Enable Errors tab (track error codes and failed requests):**
```bash
logutil /var/log/nginx/access.log --enable-errors
```

**Enable Bots tab (detect bot traffic and crawlers):**
```bash
logutil /var/log/nginx/access.log --enable-bots
```

**Enable Sparkline tab (real-time request rate visualization):**
```bash
logutil /var/log/nginx/access.log --enable-sparkline
```

**Enable Heatmap tab (hourly traffic patterns visualization):**
```bash
logutil /var/log/nginx/access.log --enable-heatmap
```

**Enable all tabs:**
```bash
logutil /var/log/nginx/access.log --enable-security --enable-performance --enable-errors --enable-bots --enable-sparkline --enable-heatmap
```

### Console Output Mode

**Show top URLs in console:**
```bash
logutil /var/log/nginx/access.log --show-urls --top=10
```

**Show top IPs in console:**
```bash
logutil /var/log/nginx/access.log --show-ips --top=10
```

## Interactive Interface

### Navigation

- **`Tab` or `t`**: Switch between tabs (Overview → Requests → Detailed → Sparkline → Heatmap)
- **`↑/↓`**: Navigate through lists
- **`←/→`**: Switch between panels or pages
- **`Enter`**: Copy selected item to clipboard (in Overview tab)
- **`q` or `Ctrl+C`**: Quit the application

### Tabs Overview

#### 1. **Overview Tab**
- **Left Panel**: Top IP addresses with request counts and last update times
- **Right Panel**: Top URLs with request types, domains, and statistics
- **Bottom Panel**: Full URL display for selected URL
- **Navigation**: Use arrow keys to switch between panels

#### 2. **Requests Tab**
- **Search Functionality**: Type to filter log entries
- **Pagination**: Navigate through large result sets
- **Real-time Updates**: New requests appear automatically

#### 3. **Detailed Tab**
- **IP List**: Select an IP address to view its details
- **Request Details**: View all requests from the selected IP
- **Drill-down Analysis**: Understand traffic patterns per IP

#### 4. **Sparkline Tab**
- **Real-time Timeline**: Visual representation of request patterns
- **Statistics**: Min, max, average, and current request counts
- **Time Range**: Shows the time span of the data

#### 5. **Heatmap Tab**
- **Hourly Patterns**: Color-coded request intensity by hour
- **Multi-day View**: Track patterns across multiple days
- **Legend**: Blue (low) → Green → Red (high) intensity

#### 6. **Security Tab**
- **Advanced Threat Detection**: SQL Injection, XSS, Path Traversal, Command Injection, Brute Force
- **Log Analysis**: View highlighted suspicious patterns in log entries with visual indicators
- **IP Blocking**: Block/unblock suspicious IP addresses with Enter key
- **Threat Levels**: Visual indicators (🔴🟡🟢) for threat severity assessment
- **Pattern Highlighting**: Suspicious patterns are highlighted with icons in log details
- **Security Summary**: Comprehensive overview of all security threats and violations
- **Log Detail View**: Press Enter to view detailed logs with highlighted suspicious patterns
- **Real-time Monitoring**: Continuous monitoring and detection of security threats

#### 7. **Performance Tab**
- **Response Time Analysis**: Tracks average, min, and max response times
- **Slow Request Identification**: Highlights requests taking longer than 1 second
- **Requests Per Second**: Real-time RPS calculation and monitoring
- **Performance Tracking**: Detailed tracking of slow requests with timestamps
- **Performance Metrics**: Real-time performance statistics
- **Resource Usage**: Total response size and throughput analysis

#### 8. **Errors Tab**
- **HTTP Error Analysis**: Categorizes and counts error codes (4xx, 5xx)
- **Error Pattern Detection**: Identifies common error sources
- **Error Distribution**: Shows which URLs and IPs generate most errors
- **Error Trends**: Tracks error patterns over time

#### 9. **Bots Tab**
- **Bot Detection**: Identifies crawlers, scrapers, and automated traffic
- **Bot Classification**: Categorizes different types of bots (Googlebot, Bingbot, etc.)
- **Bot Activity Analysis**: Tracks bot behavior patterns
- **Bot Traffic Statistics**: Quantifies bot vs human traffic

## Configuration

### Command Line Options

| Option | Description | Default |
|--------|-------------|---------|
| `file` | Path to the log file | Required |
| `--count` | Lines to read from end (0=tail, -1=all) | `0` |
| `--regex` | Regex pattern or file path | Nginx default |
| `--date-format` | Date parsing format | `%d/%b/%Y:%H:%M:%S %z` |
| `--top` | Number of top entries to show | `10` |
| `--no-clear` | Disable automatic cleanup | `false` |
| `--show-urls` | Output top URLs to console | `false` |
| `--show-ips` | Output top IPs to console | `false` |
| `--log-to-file` | Enable logging to app.log | `false` |
| `--enable-security` | Enable Security tab | `false` |
| `--enable-performance` | Enable Performance tab | `false` |
| `--enable-errors` | Enable Errors tab | `false` |
| `--enable-bots` | Enable Bots tab | `false` |
| `--enable-sparkline` | Enable Sparkline tab | `false` |
| `--enable-heatmap` | Enable Heatmap tab | `false` |

### Supported Log Formats

#### 1. **Nginx Access Log (Default)**
```regex
^(\S+) - ".+" \[(.*?)\] \d+\.\d+ "(\S+)" "(\S+) (\S+?)(?:\?.*?)? "
```
**Date Format:** `%d/%b/%Y:%H:%M:%S %z`

#### 2. **Apache Access Log**
```regex
^(\S+) \S+ \S+ \[.*?\] "\S+ (\S+?)(?:\?.*?)? \S+" \d+ \d+
```
**Date Format:** `%d/%b/%Y:%H:%M:%S %z`

#### 3. **Custom Format Example**
```regex
^(\S+) - - \[(.*?)\] "(\S+) (\S+) HTTP/\d+\.\d+" (\d+) (\d+) "([^"]*)" "([^"]*)"$
```
**Date Format:** `%d/%b/%Y:%H:%M:%S %z`

## Examples

### Basic Monitoring
```bash
# Monitor nginx access logs in real-time
logutil /var/log/nginx/access.log

# Analyze entire log file
logutil /var/log/nginx/access.log --count=-1 --top=20
```

### Custom Log Formats
```bash
# Apache access logs
logutil /var/log/apache2/access.log --regex='^(\S+) \S+ \S+ \[.*?\] "\S+ (\S+?)(?:\?.*?)? \S+" \d+ \d+'

# Custom application logs
logutil /var/log/app/access.log --regex='^(\S+) \[(.*?)\] (\S+) (\S+)'
```

### Console Output
```bash
# Get top URLs for reporting
logutil /var/log/nginx/access.log --show-urls --top=10

# Get top IPs for security analysis
logutil /var/log/nginx/access.log --show-ips --top=20
```

### Advanced Configuration
```bash
# Custom regex from file
logutil /var/log/nginx/access.log --regex=/etc/logutil/patterns.txt

# Disable cleanup for long-term analysis
logutil /var/log/nginx/access.log --no-clear --count=-1

# Custom date format
logutil /var/log/nginx/access.log --date-format="%Y-%m-%d %H:%M:%S"
```

## Performance Considerations

### Memory Management
- **Automatic Cleanup**: Removes entries older than 20 minutes when over 10,000 entries
- **Configurable**: Use `--no-clear` to disable automatic cleanup
- **Efficient**: Minimal memory footprint even with large log files

### Processing Speed
- **Real-time**: Processes new lines as they appear
- **Optimized**: Efficient regex matching and data structures
- **Scalable**: Handles high-traffic logs without performance degradation

### File Handling
- **Smart Reading**: Only processes new lines when tailing
- **Error Recovery**: Gracefully handles file rotation and truncation
- **Progress Tracking**: Shows loading progress for large files

## Troubleshooting

### Common Issues

**1. "No match for line" errors**
- Check your regex pattern with `--regex` option
- Verify log format matches the expected pattern
- Use `--log-to-file` to debug parsing issues

**2. High memory usage**
- Enable automatic cleanup (default behavior)
- Use `--count` to limit initial processing
- Consider using `--no-clear` only for short-term analysis

**3. Slow performance with large files**
- Use `--count=1000` to limit initial processing
- Ensure regex pattern is optimized
- Check system resources (CPU, memory)

### Debug Mode
```bash
# Enable debug logging
logutil /var/log/nginx/access.log --log-to-file

# Check the generated app.log file for errors
tail -f app.log
```

## Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for details.

### Development Setup
```bash
git clone https://github.com/s00d/logutil.git
cd logutil
cargo build
cargo test
```