# CodeStat

用 Rust 写的代码统计工具，可以统计代码行数和编程语言分布。

[English Documentation](./README.md)

## 功能

- 使用 Rayon 并行处理
- Rust 实现，内存安全
- 分别统计代码、注释和空行
- 支持 50 多种编程语言
- 遵循 .gitignore 规则
- 支持表格、JSON、CSV 三种输出格式
- 增量缓存，重复运行更快

## 安装

```bash
cargo install codestat
```

或者从源码编译：

```bash
git clone https://github.com/Cactusinhand/codestat
cd codestat
cargo build --release
```

编译后的二进制文件在 `target/release/codestat`。

## 用法

基本用法：

```bash
# 分析当前目录
codestat

# 分析指定目录
codestat /path/to/project

# 分析单个文件
codestat src/main.rs
```

输出格式：

```bash
# 表格（默认）
codestat --format table .

# JSON
codestat --format json .

# CSV
codestat --format csv .
```

选项：

```bash
# 显示每个文件的统计
codestat -F .

# 显示进度
codestat --progress .

# 包含隐藏文件
codestat --hidden .

# 只统计特定语言
codestat --languages rust,python,go .

# 排除特定模式
codestat --exclude "*.test.js,*.min.css" .

# 禁用并行处理（减少内存占用）
codestat --no-parallel .

# 跟随符号链接
codestat --follow-links .
```

## 示例输出

表格格式：

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

JSON 格式：

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

## 实现说明

提升性能的技术：

- **内存映射**: 大文件使用 mmap，避免拷贝开销
- **字节级解析**: 直接操作 `&[u8]`，不做 UTF-8 验证
- **并行批处理**: 按文件大小排序，平衡负载
- **缓存检测**: 语言检测用静态 HashMap，O(1) 查找

## 许可证

MIT
