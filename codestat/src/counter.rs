use crate::language::{CommentSyntax, Language};
use crate::mempool::{acquire_buffer, release_buffer, open_with_advise};
use crate::simd_scanner::{count_newlines, is_blank_line_simd};
use crate::stats::FileStats;
use memmap2::Mmap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// 文件大小阈值：超过此大小使用内存映射 (1MB)
const MMAP_THRESHOLD: u64 = 1024 * 1024;

pub fn count_file(path: &Path, language: Language) -> Result<FileStats, std::io::Error> {
    let metadata = std::fs::metadata(path)?;
    let file_size = metadata.len();
    
    if file_size == 0 {
        return Ok(FileStats::default());
    }
    
    // 根据文件大小选择最佳读取策略
    if file_size > MMAP_THRESHOLD {
        count_file_mmap(path, language, file_size)
    } else {
        count_file_buffered(path, language, file_size)
    }
}

/// 使用内存映射读取大文件（零拷贝）
fn count_file_mmap(path: &Path, language: Language, file_size: u64) -> Result<FileStats, std::io::Error> {
    let file = File::open(path)?;
    let mmap = unsafe { Mmap::map(&file)? };
    
    // 直接操作字节切片，避免 String 转换开销
    let stats = analyze_bytes(&mmap, language);
    
    Ok(FileStats {
        lines: stats.lines,
        code_lines: stats.code_lines,
        comment_lines: stats.comment_lines,
        blank_lines: stats.blank_lines,
        bytes: file_size,
    })
}

/// 使用缓冲区读取小文件（内存池优化）
fn count_file_buffered(path: &Path, language: Language, file_size: u64) -> Result<FileStats, std::io::Error> {
    // 使用内存池获取缓冲区
    let mut buffer = acquire_buffer(file_size as usize);
    
    // 使用预读取提示打开文件
    let mut file = open_with_advise(path)?;
    file.read_to_end(&mut buffer)?;
    
    let stats = analyze_bytes(&buffer, language);
    
    // 归还缓冲区到内存池
    release_buffer(buffer);
    
    Ok(FileStats {
        lines: stats.lines,
        code_lines: stats.code_lines,
        comment_lines: stats.comment_lines,
        blank_lines: stats.blank_lines,
        bytes: file_size,
    })
}

/// 快速模式 - 不解析注释，只统计总行数和空行
#[allow(dead_code)]
pub fn count_file_fast(path: &Path, _language: Language) -> Result<FileStats, std::io::Error> {
    let metadata = std::fs::metadata(path)?;
    let file_size = metadata.len();
    
    if file_size == 0 {
        return Ok(FileStats::default());
    }
    
    let (lines, blank_lines) = if file_size > MMAP_THRESHOLD {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        count_lines_fast(&mmap)
    } else {
        let content = std::fs::read(path)?;
        count_lines_fast(&content)
    };
    
    let code_lines = lines - blank_lines;
    
    Ok(FileStats {
        lines,
        code_lines,
        comment_lines: 0,
        blank_lines,
        bytes: file_size,
    })
}

#[derive(Debug, Default)]
struct AnalysisResult {
    lines: usize,
    code_lines: usize,
    comment_lines: usize,
    blank_lines: usize,
}

/// 高性能字节级分析（避免 UTF-8 验证和 String 分配）
fn analyze_bytes(content: &[u8], language: Language) -> AnalysisResult {
    let syntax = language.get_comment_syntax();
    let mut result = AnalysisResult::default();
    let mut in_block_comment = false;
    let mut i = 0;
    let len = content.len();
    
    while i < len {
        // 找到行尾或文件尾
        let line_start = i;
        while i < len && content[i] != b'\n' {
            i += 1;
        }
        let line_end = i;
        if i < len && content[i] == b'\n' {
            i += 1; // 跳过换行符
        }
        
        result.lines += 1;
        
        // 快速检查空行（只包含空白字符）
        let line = &content[line_start..line_end];
        if is_blank_line(line) {
            result.blank_lines += 1;
            continue;
        }
        
        // 分析注释和代码
        let line_stats = analyze_line_bytes(line, &syntax, &mut in_block_comment);
        if line_stats.is_comment {
            result.comment_lines += 1;
        } else {
            result.code_lines += 1;
        }
    }
    
    result
}

/// 快速检查是否为空行（使用 SIMD）
#[inline(always)]
fn is_blank_line(line: &[u8]) -> bool {
    is_blank_line_simd(line)
}

#[derive(Debug, Default)]
struct LineStats {
    is_comment: bool,
}

/// 分析单行字节内容
#[inline(always)]
fn analyze_line_bytes(line: &[u8], syntax: &CommentSyntax, in_block: &mut bool) -> LineStats {
    let mut stats = LineStats::default();
    let mut i = 0;
    let len = line.len();
    
    // 跳过前导空白
    while i < len && matches!(line[i], b' ' | b'\t' | b'\r') {
        i += 1;
    }
    
    if i >= len {
        stats.is_comment = *in_block;
        return stats;
    }
    
    let remaining = &line[i..];
    
    // 处理块注释
    if let (Some(block_start), Some(block_end)) = (syntax.block_start, syntax.block_end) {
        let bs = block_start.as_bytes();
        let be = block_end.as_bytes();
        
        loop {
            if *in_block {
                // 寻找块注释结束
                if let Some(pos) = find_subsequence(remaining, be) {
                    *in_block = false;
                    i += pos + be.len();
                    if i >= len {
                        stats.is_comment = true;
                        return stats;
                    }
                    // 检查剩余部分
                    let rest = &line[i..];
                    let rest_trimmed = trim_bytes_start(rest);
                    if rest_trimmed.is_empty() {
                        stats.is_comment = true;
                        return stats;
                    }
                    // 检查剩余部分是否是行注释
                    if let Some(lc) = syntax.line {
                        if rest_trimmed.starts_with(lc.as_bytes()) {
                            stats.is_comment = true;
                            return stats;
                        }
                    }
                    stats.is_comment = false;
                    return stats;
                } else {
                    stats.is_comment = true;
                    return stats;
                }
            } else {
                // 寻找块注释开始
                if let Some(pos) = find_subsequence(remaining, bs) {
                    // 检查块注释前是否有代码
                    let before = &remaining[..pos];
                    if !trim_bytes_start(before).is_empty() {
                        stats.is_comment = false;
                        return stats;
                    }
                    
                    *in_block = true;
                    i += pos + bs.len();
                    
                    // 检查是否在同一行结束
                    let after = &line[i..];
                    if let Some(end_pos) = find_subsequence(after, be) {
                        *in_block = false;
                        i += end_pos + be.len();
                        let rest = &line[i..];
                        if trim_bytes_start(rest).is_empty() {
                            stats.is_comment = true;
                            return stats;
                        }
                        if let Some(lc) = syntax.line {
                            if trim_bytes_start(rest).starts_with(lc.as_bytes()) {
                                stats.is_comment = true;
                                return stats;
                            }
                        }
                        stats.is_comment = false;
                        return stats;
                    }
                } else {
                    break;
                }
            }
        }
    }
    
    // 检查行注释
    if !*in_block {
        if let Some(lc) = syntax.line {
            if remaining.starts_with(lc.as_bytes()) {
                stats.is_comment = true;
                return stats;
            }
        }
    } else {
        stats.is_comment = true;
    }
    
    stats
}

/// 字节序列查找（简化版 Boyer-Moore 适用于短模式）
#[inline(always)]
fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    if haystack.len() < needle.len() {
        return None;
    }
    
    haystack.windows(needle.len()).position(|window| window == needle)
}

/// 跳过字节的前导空白
#[inline(always)]
fn trim_bytes_start(bytes: &[u8]) -> &[u8] {
    let mut i = 0;
    while i < bytes.len() && matches!(bytes[i], b' ' | b'\t' | b'\r') {
        i += 1;
    }
    &bytes[i..]
}

/// 快速行计数（SIMD 优化实现）
#[allow(dead_code)]
fn count_lines_fast(content: &[u8]) -> (usize, usize) {
    // 使用 SIMD 加速统计换行符
    let total_newlines = count_newlines(content);
    
    // 快速空行检测
    let mut blank_lines = 0;
    let mut line_start = 0;
    
    for (i, &byte) in content.iter().enumerate() {
        if byte == b'\n' {
            let line = &content[line_start..i];
            if is_blank_line_simd(line) {
                blank_lines += 1;
            }
            line_start = i + 1;
        }
    }
    
    // 处理最后一行
    if line_start < content.len() {
        let line = &content[line_start..];
        if is_blank_line_simd(line) {
            blank_lines += 1;
        }
    }
    
    // 如果文件以换行符结尾，total_newlines 就是总行数
    // 否则需要 +1
    let lines = if content.last() == Some(&b'\n') {
        total_newlines
    } else {
        total_newlines + if content.is_empty() { 0 } else { 1 }
    };
    
    (lines, blank_lines)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rust_comments() {
        let content = b"// This is a line comment\nfn main() {\n    println!(\"Hello\");\n}\n/* Block comment */\n/* Multi\n   line\n   comment */\nfn test() {} // trailing comment\n";
        let result = analyze_bytes(content, Language::Rust);
        
        assert_eq!(result.lines, 9);
        assert_eq!(result.blank_lines, 0);  // 该测试内容中没有空行
        assert!(result.comment_lines >= 3);
        assert!(result.code_lines > 0);
    }
    
    #[test]
    fn test_python_comments() {
        let content = b"# This is a comment\n\ndef hello():\n    print(\"world\")  # inline comment\n\n\"\"\"\nDocstring comment\n\"\"\"\n";
        let result = analyze_bytes(content, Language::Python);
        
        assert_eq!(result.lines, 8);
        assert_eq!(result.blank_lines, 2);
        assert!(result.comment_lines >= 2);
    }
    
    #[test]
    fn test_empty_file() {
        let content = b"";
        let result = analyze_bytes(content, Language::Rust);
        assert_eq!(result.lines, 0);
    }
    
    #[test]
    fn test_blank_lines() {
        let content = b"\n   \n\t\n\r\n";
        let result = analyze_bytes(content, Language::Rust);
        assert_eq!(result.lines, 4);
        assert_eq!(result.blank_lines, 4);
        assert_eq!(result.code_lines, 0);
    }
    
    #[test]
    fn test_find_subsequence() {
        assert_eq!(find_subsequence(b"hello world", b"world"), Some(6));
        assert_eq!(find_subsequence(b"hello world", b"foo"), None);
        assert_eq!(find_subsequence(b"", b"foo"), None);
        assert_eq!(find_subsequence(b"hello", b""), Some(0));
    }
}
