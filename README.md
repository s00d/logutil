Here's the updated README reflecting the changes:

# Log Util

Log Util is a Rust-based tool designed to analyze Nginx access logs. It reads log files, extracts and counts requests by IP addresses and URLs, and displays the top IP addresses and URLs. The tool can handle large log files by reading only new data as it is appended, or by processing the entire file from the beginning.

## Features

- Reads Nginx access logs and extracts IP addresses and URLs.
- Counts and displays the top IP addresses and URLs.
- Supports specifying the number of top entries to display.
- Allows customizing the regular expression used for log parsing.
- Supports loading the regular expression from a file.
- Automatic cleanup of outdated entries if there are more than 10,000 records (can be disabled with `--no-clear`).
- Displays the last 10 requests for top IPs.
- Allows filtering results by IP address.

## Usage

### Command-line Options

- `--file`: Path to the log file (default: `access.log`).
- `--count`: Number of lines to read from the end of the file (`0` to start from the end, `-1` to read the entire file; default: `0`).
- `--regex`: Regular expression to parse the log entries or path to a file containing the regex (default: `^(\S+) - ".+" \[.*?\] \d+\.\d+ "\S+" "\S+ (\S+?)(?:\?.*?)? HTTP/.*`).
- `--top`: Number of top entries to display (default: `10`).
- `--no-clear`: Disable automatic cleanup of outdated entries.
- `--show-last-requests`: Display the last 10 requests for top IPs.
- `--filter-ip`: Filter results by IP address.
- `--refresh`: Refresh interval for console updates in seconds (default: `5`).

### Example

To read the entire log file:

```sh
cargo run -- --file "./access.log" --count=-1
```

To read the last 100 lines from the log file:

```sh
cargo run -- --file "./access.log" --count=100
```

To read new data from the end of the log file as it is appended:

```sh
cargo run -- --file "./access.log" --count=0
```

### Loading Regular Expression from a File

If the `--regex` parameter points to a file, the regular expression will be read from that file.

```sh
cargo run -- --file "/path/to/access.log" --regex "/path/to/regex.txt" --top 20
```

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) installed on your system.

### Building from Source

1. Clone the repository:
    ```sh
    git clone https://github.com/yourusername/log-analyzer.git
    cd log-analyzer
    ```

2. Build the project:
    ```sh
    cargo build --release
    ```

3. Run the project:
    ```sh
    cargo run -- --file "/path/to/access.log" --count=-1 --top 20
    ```

## GitHub Actions

The project includes a GitHub Actions workflow that automatically builds the project for `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu` targets on each push or pull request to the `main` branch.

### Workflow Configuration

The workflow file is located at `.github/workflows/build.yml`. It performs the following steps:

1. Checks out the code.
2. Installs the Rust toolchain and necessary targets.
3. Installs dependencies (e.g., `gcc`).
4. Builds the project in release mode for the specified targets.
5. Uploads the build artifacts.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request for any improvements or bug fixes.

### Pull Request Process

1. Fork the repository.
2. Create a new branch (`git checkout -b feature-branch`).
3. Make your changes.
4. Commit your changes (`git commit -am 'Add some feature'`).
5. Push to the branch (`git push origin feature-branch`).
6. Open a pull request.

### Coding Standards

- Follow Rust's official [style guidelines](https://doc.rust-lang.org/1.0.0/style/).
- Ensure the code passes all tests and lints.
- Write or update tests as necessary.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Prettytable-rs](https://crates.io/crates/prettytable-rs) for table formatting.
- [Regex](https://crates.io/crates/regex) for regular expression parsing.
- [Structopt](https://crates.io/crates/structopt) for command-line argument parsing.
- [Tokio](https://crates.io/crates/tokio) for asynchronous runtime support.
- [Crossterm](https://crates.io/crates/crossterm) for terminal handling.

## Example Regular Expressions

Below are 10 different example regular expressions for parsing logs from various systems:

1. **Nginx Access Log (default)**
    ```regex
    ^(\S+) - ".+" \[.*?\] \d+\.\d+ "\S+" "\S+ (\S+?)(?:\?.*?)? HTTP/.*
    ```

2. **Apache Access Log**
    ```regex
    ^(\S+) \S+ \S+ \[.*?\] "\S+ (\S+?)(?:\?.*?)? \S+" \d+ \d+
    ```

3. **Nginx Error Log**
    ```regex
    ^(\d{4}/\d{2}/\d{2} \d{2}:\d{2}:\d{2}) \[error\] \d+#\d+: \*.*?, client: (\S+), server: \S+, request: "\S+ \S+ \S+", host: "\S+"
    ```

4. **Apache Error Log**
    ```regex
    ^\[\w+ \w+ \d+ \d{2}:\d{2}:\d{2} \d{4}\] \[error\] \[client (\S+)\] \S+:\s(\S+)
    ```

5. **Systemd Journal Log**
    ```regex
    ^<\d+>\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+Z \S+ \S+ \S+ \S+ - - \[ID \d+ \S+\] \S+ (\S+)$
    ```

6. **MySQL General Log**
    ```regex
    ^(\d{6} \d{2}:\d{2}:\d{2})\t(\S+)\t(\S+)\t\S+\t(\S+)$
    ```

7. **PostgreSQL Log**
    ```regex
    ^(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}\.\d+ \S+ \S+) (\S+): \[(\d+)\-(\d+)\] \S+:\s(\S+)$
    ```

8. **Docker Log**
    ```regex
    ^(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}.\d+Z) \S+ \S+ \[error\] (\S+) \S+: (\S+)$
    ```

9. **Kubernetes Log**
    ```regex
    ^(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}.\d+Z) \S+ \S+ (\S+)\[\d+\]: \S+ \S+ (\S+)$
    ```

10. **Redis Log**
    ```regex
    ^(\d{4}:\d{2}:\d{2} \d{2}:\d{2}:\d{2}) \[\d+\] (\S+): \S+ (\S+)$
    ```

You can use any of these regular expressions with Log Util by specifying them directly or by placing them in a file and pointing the `--regex` parameter to that file.