# CodeStat - AI Agent Guide

## Project Overview

CodeStat (codestat) is a high-performance code statistics tool written in Rust. It counts lines of code and programming language distribution in Git repositories or specified directories.

**Key Characteristics:**
- **Language**: Rust (2024 Edition)
- **Version**: 0.1.0
- **License**: MIT
- **Repository Structure**: Single Rust crate under `codestat/` directory

**Main Features:**
- ⚡ High performance with parallel processing (Rayon) and SIMD acceleration
- 🛡️ Memory-safe pure Rust implementation
- 📊 Detailed statistics: code lines, comment lines, blank lines
- 🔍 Smart detection for 50+ programming languages
- 🎯 Respects `.gitignore` patterns
- 📁 Multiple output formats: table (default), JSON, CSV
- 💾 Incremental caching for faster subsequent runs

## Project Structure

```
codestat/
├── Cargo.toml          # Project configuration
├── Cargo.lock          # Dependency lock file
├── README.md           # Documentation (in Chinese)
└── src/
    ├── main.rs         # CLI entry point, file collection, result output
    ├── counter.rs      # File counting logic (mmap, buffer, SIMD)
    ├── language.rs     # Language detection, comment syntax definitions
    ├── stats.rs        # Statistics data structures (FileStats, TotalStats)
    ├── cache.rs        # Incremental caching system
    ├── mempool.rs      # Memory pool for buffer reuse
    ├── simd_scanner.rs # SIMD-accelerated byte scanning (NEON/SSE/AVX)
    ├── async_counter.rs# Async I/O implementation (experimental)
    └── benchmark.rs    # Built-in benchmark suite
```

## Technology Stack

**Core Dependencies:**
- `clap` (4.5) - Command-line argument parsing with derive macros
- `rayon` (1.10) - Data parallelism and work-stealing scheduler
- `ignore` (0.4) - File traversal with .gitignore support
- `serde`/`serde_json` (1.0) - JSON serialization
- `memmap2` (0.9) - Memory-mapped file I/O (zero-copy)
- `num_cpus` (1.16) - CPU core detection
- `tempfile` (3.14) - Temporary file creation for benchmarks
- `lazy_static` (1.5) - Static variable initialization

**Build Profiles:**
- Release profile optimized with LTO, single codegen unit, panic=abort, stripped binaries

## Build and Run Commands

```bash
# Build debug version
cargo build

# Build optimized release version
cargo build --release

# Run tests
cargo test

# Run the tool
cargo run --release -- [OPTIONS] [PATH]

# Run with specific options
cargo run --release -- --format json /path/to/project
cargo run --release -- --benchmark
```

**Binary Location:** `codestat/target/release/codestat`

## CLI Usage

```bash
# Basic usage
codestat                          # Analyze current directory
codestat /path/to/project         # Analyze specific directory
codestat src/main.rs              # Analyze single file

# Output formats
codestat --format table .         # Table format (default)
codestat --format json .          # JSON output
codestat --format csv .           # CSV output

# Options
codestat --files .                # Show per-file statistics
codestat --progress .             # Show progress
codestat --hidden .               # Include hidden files
codestat --follow-links .         # Follow symbolic links
codestat --no-parallel .          # Disable parallel processing
codestat --fresh .                # Disable cache, force full scan
codestat --rebuild-cache .        # Clear and rebuild cache

# Filtering
codestat --languages rust,python,go .   # Only specific languages
codestat --exclude "*.test.js,*.min.css" .  # Exclude patterns

# Benchmark
codestat --benchmark              # Run internal benchmark suite
```

## Code Organization

### Module Responsibilities

**`main.rs`**
- CLI argument parsing with clap derive macros
- File collection using `ignore::WalkBuilder`
- Cache management (incremental context)
- Parallel/sequential processing orchestration
- Output formatting (table, JSON, CSV)

**`counter.rs`**
- `count_file()` - Main entry for file counting
- `count_file_mmap()` - Memory-mapped I/O for large files (>1MB)
- `count_file_buffered()` - Buffered I/O for small files with memory pool
- `analyze_bytes()` - Byte-level line analysis without UTF-8 validation
- `analyze_line_bytes()` - Comment/code detection per line
- SIMD-accelerated blank line detection

**`language.rs`**
- `Language` enum - 50+ programming languages
- `CommentSyntax` struct - Line and block comment patterns
- `detect_language()` - Extension and filename-based detection
- Static HashMaps with `OnceLock` for O(1) lookup

**`stats.rs`**
- `FileStats` - Per-file statistics (lines, code, comments, blanks, bytes)
- `LanguageStats` - Aggregated per-language statistics
- `TotalStats` - Overall statistics with sorting methods
- `Summary` - Final result with timing and errors

**`cache.rs`**
- `Cache` - Persistent JSON cache storage
- `CacheEntry` - Individual file cache (mtime, size, stats)
- `IncrementalContext` - Cache hit/miss tracking
- Default cache file: `.codestat-cache.json`

**`mempool.rs`**
- `acquire_buffer()` - Get reusable buffer from pool
- `release_buffer()` - Return buffer to pool
- Thread-local and global pools with size limits
- `open_with_advise()` - Linux preadvise optimization

**`simd_scanner.rs`**
- `count_newlines()` - SIMD-accelerated newline counting
- ARM NEON implementation (128-bit)
- x86 SSE2 (128-bit) and AVX2 (256-bit) implementations
- Fallback for non-SIMD platforms

## Testing Strategy

**Unit Tests:**
- Embedded in each module using `#[cfg(test)]`
- Tests for comment parsing in various languages
- Buffer pool operations
- Cache save/load
- SIMD scanner functions

**Benchmark Tests:**
- `codestat --benchmark` runs comprehensive benchmarks
- Language detection performance
- File size scaling (1KB to 10MB)
- Language parsing comparison
- Memory usage tests
- Parallel vs sequential speedup

## Development Conventions

**Code Style:**
- Chinese comments for documentation and explanations
- English for code identifiers (variables, functions, types)
- `#[inline(always)]` for hot path functions
- Extensive use of `unsafe` only in SIMD modules with proper target_feature gates

**Performance Optimizations:**
1. **Zero-copy**: Memory-mapped files for large inputs
2. **Byte-level parsing**: Avoid String/UTF-8 validation
3. **SIMD**: Platform-specific vectorized operations
4. **Memory pooling**: Reuse buffers across file operations
5. **Parallel processing**: Rayon for data parallelism with work-stealing
6. **Incremental caching**: Skip unchanged files on subsequent runs
7. **Smart scheduling**: Sort files by size for better load balancing

**File Size Thresholds:**
- Small files (< 1MB): Buffered I/O with memory pool
- Large files (> 1MB): Memory-mapped I/O
- Parallel threshold: < 100 files use sequential processing

## Supported Languages

**50+ Programming Languages:**
- Systems: Rust, C, C++, Go, Zig, Assembly
- Web: JavaScript, TypeScript, HTML, CSS, SCSS, PHP
- JVM: Java, Kotlin, Scala, Groovy, Clojure
- Mobile: Swift, Objective-C, Dart
- Scripting: Python, Ruby, Perl, Shell, PowerShell
- Functional: Haskell, Elixir, Erlang, F#, Lisp
- Data: R, MATLAB, Julia
- Config: JSON, YAML, XML, TOML, Markdown

## Cache System

**Cache File:** `.codestat-cache.json` (stored in target directory)

**Cache Validity:** Checked via file mtime and size comparison

**Cache Commands:**
- `--fresh` - Ignore existing cache
- `--rebuild-cache` - Clear and rebuild from scratch

**Security:** Cache file created with 0o600 permissions on Unix

## Security Considerations

- Memory-mapped files use `unsafe` but with bounds checking via the mmap crate
- SIMD operations use proper target feature detection
- No execution of arbitrary code
- Respects .gitignore for security (won't traverse sensitive paths)
- File permissions preserved for cache

## Performance Benchmarks (Reference)

Tested on M1 Pro, macOS:
- 4,600 files, 1.6M lines: ~0.42s
- 50K+ files, 20M+ lines: ~3s
- Parallel speedup: ~6.4x on 8-core system

Comparison with other tools:
- codestat: 0.42s (this tool)
- tokei: ~0.6s
- scc: ~1s
- cloc: ~4s
