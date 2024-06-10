# Log Util

Log Util is a Rust-based tool designed to analyze Nginx access logs. It reads log files, extracts and counts requests by IP addresses and URLs, and displays the top IP addresses and URLs. The tool can handle large log files by reading only new data as it is appended, or by processing the entire file from the beginning.

## Features

- Reads Nginx access logs and extracts IP addresses and URLs.
- Counts and displays the top IP addresses and URLs.
- Can operate in two modes: `new` (read only new data) and `all` (read the entire file).
- Supports specifying the number of top entries to display.
- Allows customizing the regular expression used for log parsing.

## Usage

### Command-line Options

- `--file`: Path to the log file (default: `access.log`).
- `--mode`: Mode of operation (`new` to read new data from the end, `all` to read the entire file; default: `new`).
- `--regex`: Regular expression to parse the log entries (default: `^(\S+) - "-\|-" \[.*?\] \d+\.\d+ "\S+" "\S+ (\S+?)(?:\?.*?)? HTTP/.*"`).
- `--top`: Number of top entries to display (default: `10`).

### Example

```sh
cargo run -- --file "/path/to/access.log" --mode all --top 20
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
    cargo run -- --file "/path/to/access.log" --mode all --top 20
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
