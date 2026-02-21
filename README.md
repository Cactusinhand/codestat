# CodeStat

一个使用 Rust 编写的高性能代码统计工具，用于统计 Git 仓库或指定目录中的代码行数和编程语言分布。

## 特性

- **⚡ 极速**: 使用并行处理 (Rayon) 和优化的文件遍历
- **🛡️ 安全**: 纯 Rust 实现，内存安全，无数据竞争
- **📊 详细统计**: 代码行、注释行、空行分别统计
- **🔍 智能检测**: 支持 50+ 种编程语言
- **🎯 尊重 .gitignore**: 自动忽略不应统计的文件
- **📁 灵活输出**: 支持表格、JSON、CSV 格式

## 支持的编程语言

- **系统编程**: Rust, C, C++, Go, Zig, Assembly
- **Web 开发**: JavaScript, TypeScript, HTML, CSS, SCSS, PHP
- **JVM 语言**: Java, Kotlin, Scala, Groovy, Clojure
- **移动开发**: Swift, Objective-C, Dart
- **脚本语言**: Python, Ruby, Perl, Shell, PowerShell
- **函数式**: Haskell, Elixir, Erlang, F#, Lisp
- **数据科学**: R, MATLAB, Julia
- **配置/标记**: JSON, YAML, XML, TOML, Markdown
- 更多...

## 安装

### 从源码构建

```bash
git clone <repository>
cd codestat
cargo build --release
```

编译后的二进制文件位于 `target/release/codestat`

## 使用方法

### 基本用法

```bash
# 统计当前目录
codestat

# 统计指定目录
codestat /path/to/project

# 统计单个文件
codestat src/main.rs
```

### 输出格式

```bash
# 表格格式 (默认)
codestat --format table .

# JSON 格式
codestat --format json .

# CSV 格式
codestat --format csv .
```

### 高级选项

```bash
# 显示每个文件的统计
codestat --files .

# 显示进度
codestat --progress .

# 包含隐藏文件
codestat --hidden .

# 只统计特定语言
codestat --languages rust,python,go .

# 排除特定模式
codestat --exclude "*.test.js,*.min.css" .

# 禁用并行处理 (降低内存使用)
codestat --no-parallel .

# 跟随符号链接
codestat --follow-links .
```

## 性能

经过深度优化，性能表现如下：

| 项目规模 | 文件数 | 代码行数 | 耗时 | 处理速度 |
|---------|-------|---------|------|---------|
| 小型 | 500 | 30K | 0.08s | 6,500 文件/s |
| 中型 | 4,600 | 1.6M | 0.42s | 11,000 文件/s |
| 大型 | 50K+ | 20M+ | ~3s | 15,000+ 文件/s |

**优化亮点：**
- 内存映射大文件（零拷贝）
- 字节级分析（无 UTF-8 验证开销）
- 智能负载均衡（按文件大小排序）
- 批量并行处理（减少任务切换）

## 与其他工具对比

| 工具 | 4,600 文件 | 特性 |
|-----|-----------|-----|
| **codestat** | **0.42s** | Rust，零拷贝，并行 |
| tokei | ~0.6s | Rust，功能丰富 |
| cloc | ~4s | Perl，准确但慢 |
| scc | ~1s | Go，功能丰富 |

*测试环境: macOS, M1 Pro, 16GB RAM*

## 示例输出

### 表格格式

```
================================================================================
 Language                  Files      Lines       Code   Comments     Blanks
--------------------------------------------------------------------------------
 Rust                          6       1108        951         40        117
 TOML                          1         22         20          0          2
--------------------------------------------------------------------------------
 Total                         7       1130        971         40        119
================================================================================

 Statistics completed in 0.003s (2333 files/s, 376667 lines/s)
```

### JSON 格式

```json
{
  "summary": {
    "total_files": 7,
    "total_lines": 1130,
    "total_code": 971,
    "total_comments": 40,
    "total_blanks": 119,
    "elapsed_ms": 3
  },
  "languages": [
    {
      "name": "Rust",
      "files": 6,
      "lines": 1108,
      "code": 951,
      "comments": 40,
      "blanks": 117
    }
  ]
}
```

## 性能 Benchmark

运行内置 benchmark 测试各模块性能：

```bash
codestat --benchmark
```

### 典型测试结果 (M1 Pro)

```
【语言检测性能】
  速度:     4,141,859 文件/秒
  平均延迟: 241ns

【不同文件大小处理】
  1KB:    38µs   (26 MB/s)
  10KB:   89µs   (109 MB/s)
  100KB:  558µs  (175 MB/s)
  1MB:    5.3ms  (189 MB/s)
  10MB:   40ms   (250 MB/s)

【语言解析性能】
  Markdown: 33µs  (1000 lines)
  Python:   70µs  (1000 lines)
  Rust:     76µs  (1000 lines)
  JavaScript: 82µs (1000 lines)

【并行加速】
  串行: 23.7ms
  并行: 3.7ms
  加速比: 6.38x (80% 并行效率)
```

## 技术栈

- **Rust 2024 Edition**: 最新的 Rust 语言特性
- **Rayon**: 数据并行处理，工作窃取调度
- **ignore**: 智能文件遍历，支持 .gitignore
- **memmap2**: 内存映射大文件（零拷贝）
- **clap**: 命令行参数解析
- **serde**: JSON 序列化

## 优化技术详解

### 1. 零拷贝文件读取
- **小文件** (< 1MB): 预分配缓冲区一次性读取
- **大文件** (> 1MB): 使用 `mmap` 内存映射，避免内核态到用户态的拷贝

### 2. 字节级解析
- 直接操作 `&[u8]`，避免 `String` 的 UTF-8 验证开销
- 手写行扫描器，比 `lines()` 迭代器更快
- 快速空行检测：`O(n)` 字节检查

### 3. 智能并行策略
- **小批量** (< 100 文件): 串行处理，避免并行开销
- **大批量**: 按文件大小降序排序，实现负载均衡
- **批量处理**: 减少 Rayon 任务切换开销

### 4. 算法优化
- **静态 HashMap**: `OnceLock` 懒加载，O(1) 语言检测
- **字节查找**: 简化版 Boyer-Moore 子串匹配
- **单次遍历**: 一行代码完成所有统计类型

## 许可证

MIT License
