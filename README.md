# CodeStat

A fast code statistics tool written in Rust. Counts lines of code and shows language distribution for Git repositories or directories.

[中文文档](./README.zh.md)

## Features

- Fast parallel processing using Rayon
- Memory-safe Rust implementation
- Counts code, comments, and blank lines separately
- Detects 50+ programming languages
- Respects .gitignore rules
- Output in table, JSON, or CSV format
- Incremental caching for faster re-runs

## Installation

```bash
cargo install codestat
```

Or build from source:

```bash
git clone https://github.com/Cactusinhand/codestat
cd codestat
cargo build --release
```

The binary will be at `target/release/codestat`.

## Usage

Basic usage:

```bash
# Analyze current directory
codestat

# Analyze specific directory
codestat /path/to/project

# Analyze single file
codestat src/main.rs
```

Output formats:

```bash
# Table (default)
codestat --format table .

# JSON
codestat --format json .

# CSV
codestat --format csv .
```

Options:

```bash
# Show per-file statistics
codestat -F .

# Show progress
codestat --progress .

# Include hidden files
codestat --hidden .

# Filter by language
codestat --languages rust,python,go .

# Exclude patterns
codestat --exclude "*.test.js,*.min.css" .

# Disable parallel processing (less memory)
codestat --no-parallel .

# Follow symlinks
codestat --follow-links .
```

## Example Output

Table format:

```
================================================================================
 Language                  Files      Lines       Code   Comments     Blanks
--------------------------------------------------------------------------------
 Rust                          9       2536       1993        215        328
 Markdown                      2        356        260          0         96
 TOML                          1         35         32          0          3
--------------------------------------------------------------------------------
 Total                        12       2927       2285        215        427
================================================================================

Statistics completed in 0.014s (857 files/s, 209071 lines/s)
```

JSON format:

```json
{
  "summary": {
    "elapsed_ms": 6,
    "error_count": 0,
    "total_blanks": 405,
    "total_bytes": 84917,
    "total_code": 2235,
    "total_comments": 215,
    "total_files": 12,
    "total_lines": 2855
  },
  "languages": [
    {
      "blanks": 328,
      "bytes": 78300,
      "code": 1993,
      "comments": 215,
      "files": 9,
      "lines": 2536,
      "name": "Rust"
    },
    {
      "blanks": 74,
      "bytes": 5779,
      "code": 210,
      "comments": 0,
      "files": 2,
      "lines": 284,
      "name": "Markdown"
    },
    {
      "blanks": 3,
      "bytes": 838,
      "code": 32,
      "comments": 0,
      "files": 1,
      "lines": 35,
      "name": "TOML"
    }
  ],
  "files": null
}
```

## Implementation Notes

Techniques used for performance:

- **Memory mapping**: Large files use mmap to avoid copy overhead
- **Byte-level parsing**: Works on `&[u8]` directly, no UTF-8 validation
- **Parallel batching**: Files sorted by size for better load balancing
- **Cached detection**: Language detection uses static HashMap with O(1) lookup

## License

MIT
