use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::counter::count_file;
use crate::language::{detect_language, Language};
use crate::stats::{FileStats, TotalStats};

/// 运行内置 benchmark
pub fn run_benchmark() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║              CODESTAT INTERNAL BENCHMARK                        ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let base_path = temp_dir.path();

    // 1. 语言检测性能测试
    bench_language_detection(base_path);

    // 2. 不同大小文件的行数统计
    bench_file_sizes(base_path);

    // 3. 不同语言的解析性能
    bench_languages(base_path);

    // 4. 大文件内存测试
    bench_memory_usage(base_path);

    // 5. 并行 vs 串行
    bench_parallel_vs_sequential(base_path);

    println!("\n══════════════════════════════════════════════════════════════════");
    println!("                      BENCHMARK COMPLETE                          ");
    println!("══════════════════════════════════════════════════════════════════");
}

/// 测试语言检测性能
fn bench_language_detection(_base_path: &Path) {
    println!("【测试 1】语言检测性能");
    println!("{:-<60}", "");

    let test_files: Vec<PathBuf> = (0..10000)
        .map(|i| {
            let ext = match i % 10 {
                0 => "rs",
                1 => "go",
                2 => "py",
                3 => "js",
                4 => "java",
                5 => "cpp",
                6 => "c",
                7 => "ts",
                8 => "md",
                _ => "json",
            };
            PathBuf::from(format!("/fake/path/file{}.{}", i, ext))
        })
        .collect();

    let start = Instant::now();
    let mut detected = 0;
    for path in &test_files {
        if detect_language(path) != Language::Unknown {
            detected += 1;
        }
    }
    let elapsed = start.elapsed();

    println!("  测试样本: {} 个文件路径", test_files.len());
    println!("  成功检测: {} 个", detected);
    println!("  总耗时:   {:?}", elapsed);
    println!(
        "  速度:     {:.0} 文件/秒",
        test_files.len() as f64 / elapsed.as_secs_f64()
    );
    println!(
        "  平均延迟: {:?}",
        elapsed / test_files.len() as u32
    );
    println!();
}

/// 测试不同大小文件的性能
fn bench_file_sizes(base_path: &Path) {
    println!("【测试 2】不同文件大小的处理性能");
    println!("{:-<60}", "");

    let sizes = vec![
        ("1KB", 1024),
        ("10KB", 10 * 1024),
        ("100KB", 100 * 1024),
        ("1MB", 1024 * 1024),
        ("10MB", 10 * 1024 * 1024),
    ];

    for (name, size) in sizes {
        let file_path = create_test_file(base_path, size, Language::Rust);
        
        // Warm up
        let _ = count_file(&file_path, Language::Rust);

        let runs = if size > 1024 * 1024 { 3 } else { 10 };
        let start = Instant::now();
        for _ in 0..runs {
            let _ = count_file(&file_path, Language::Rust);
        }
        let elapsed = start.elapsed() / runs;

        let throughput = size as f64 / 1024.0 / 1024.0 / elapsed.as_secs_f64();
        println!(
            "  {}: {:?} ({:.0} MB/s)",
            name, elapsed, throughput
        );

        let _ = fs::remove_file(&file_path);
    }
    println!();
}

/// 测试不同语言的解析性能
fn bench_languages(base_path: &Path) {
    println!("【测试 3】不同语言的解析性能");
    println!("{:-<60}", "");

    let languages = vec![
        (Language::Rust, "rs", "// comment\nfn main() {}\n"),
        (Language::Python, "py", "# comment\ndef main(): pass\n"),
        (Language::Go, "go", "// comment\nfunc main() {}\n"),
        (Language::JavaScript, "js", "// comment\nfunction main() {}\n"),
        (Language::C, "c", "/* comment */\nint main() { return 0; }\n"),
        (Language::Markdown, "md", "# Title\n\nSome content here\n"),
    ];

    let lines_per_file = 1000;
    let runs = 100;

    for (lang, ext, pattern) in languages {
        let file_path = base_path.join(format!("test.{}", ext));
        let content = pattern.repeat(lines_per_file / 2);
        fs::write(&file_path, content).unwrap();

        let start = Instant::now();
        for _ in 0..runs {
            let _ = count_file(&file_path, lang);
        }
        let elapsed = start.elapsed();
        let avg_time = elapsed / runs;

        println!(
            "  {:12} {:?} ({} runs, {} lines/file)",
            lang.as_str(),
            avg_time,
            runs,
            lines_per_file
        );

        let _ = fs::remove_file(&file_path);
    }
    println!();
}

/// 测试内存使用
fn bench_memory_usage(base_path: &Path) {
    println!("【测试 4】内存使用测试");
    println!("{:-<60}", "");

    let sizes = vec![
        ("10MB", 10 * 1024 * 1024),
        ("50MB", 50 * 1024 * 1024),
    ];

    for (name, size) in sizes {
        let file_path = create_test_file(base_path, size, Language::Rust);
        
        // 获取处理前的内存信息（简化版，仅显示时间）
        let start = Instant::now();
        let stats = count_file(&file_path, Language::Rust).unwrap();
        let elapsed = start.elapsed();

        println!("  {} 文件:", name);
        println!("    处理时间: {:?}", elapsed);
        println!("    统计行数: {}", stats.lines);
        println!(
            "    吞吐量:   {:.1} MB/s",
            size as f64 / 1024.0 / 1024.0 / elapsed.as_secs_f64()
        );

        let _ = fs::remove_file(&file_path);
    }
    println!();
}

/// 测试并行 vs 串行性能
fn bench_parallel_vs_sequential(base_path: &Path) {
    println!("【测试 5】并行 vs 串行处理");
    println!("{:-<60}", "");

    // 创建多个测试文件
    let num_files = 100;
    let lines_per_file = 1000;
    let mut file_paths = Vec::new();

    for i in 0..num_files {
        let file_path = base_path.join(format!("parallel_test_{}.rs", i));
        let content = generate_rust_content(lines_per_file);
        fs::write(&file_path, content).unwrap();
        file_paths.push(file_path);
    }

    // 串行处理
    let start = Instant::now();
    for path in &file_paths {
        let _ = count_file(path, Language::Rust);
    }
    let sequential_time = start.elapsed();

    // 并行处理 (使用 rayon)
    use rayon::prelude::*;
    let start = Instant::now();
    file_paths.par_iter().for_each(|path| {
        let _ = count_file(path, Language::Rust);
    });
    let parallel_time = start.elapsed();

    let speedup = sequential_time.as_secs_f64() / parallel_time.as_secs_f64();

    println!("  测试样本: {} 个文件", num_files);
    println!("  串行处理: {:?}", sequential_time);
    println!("  并行处理: {:?}", parallel_time);
    println!("  加速比:   {:.2}x", speedup);
    println!(
        "  并行效率: {:.0}%",
        speedup / num_cpus::get() as f64 * 100.0
    );

    for path in &file_paths {
        let _ = fs::remove_file(path);
    }
    println!();
}

/// 创建指定大小的测试文件
fn create_test_file(base_path: &Path, size: usize, lang: Language) -> PathBuf {
    let ext = match lang {
        Language::Rust => "rs",
        Language::Python => "py",
        Language::Go => "go",
        Language::JavaScript => "js",
        _ => "txt",
    };

    let file_path = base_path.join(format!("bench_{}b.{}", size, ext));
    let line = match lang {
        Language::Rust => "fn main() { println!(\"hello\"); }\n",
        Language::Python => "def main(): print('hello')\n",
        Language::Go => "func main() { fmt.Println(\"hello\") }\n",
        _ => "line content here\n",
    };

    let line_size = line.len();
    let num_lines = size / line_size + 1;
    let content = line.repeat(num_lines);
    
    fs::write(&file_path, content).unwrap();
    file_path
}

/// 生成 Rust 测试内容
fn generate_rust_content(lines: usize) -> String {
    let mut content = String::with_capacity(lines * 50);
    content.push_str("// This is a test file\n");
    
    for i in 0..lines {
        if i % 5 == 0 {
            content.push_str(&format!("// Comment line {}\n", i));
        } else if i % 7 == 0 {
            content.push('\n');
        } else {
            content.push_str(&format!(
                "fn function_{}() {{ println!(\"test\"); }}\n",
                i
            ));
        }
    }
    content
}
