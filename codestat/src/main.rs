mod benchmark;
mod cache;
mod counter;
mod language;
mod mempool;
mod simd_scanner;
mod stats;

use benchmark::run_benchmark;
use cache::IncrementalContext;
use clap::Parser;
use counter::count_file;
use language::{detect_language, Language};
use rayon::prelude::*;
use stats::{FileStats, Summary, TotalStats};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(
    name = "codestat",
    about = "Fast and safe code statistics tool",
    version
)]
struct Args {
    /// Path to analyze (file or directory)
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Output format (table, json, csv)
    #[arg(short, long, default_value = "table")]
    format: String,

    /// Show files (not just languages)
    #[arg(short, long)]
    files: bool,

    /// Include hidden files
    #[arg(long)]
    hidden: bool,

    /// Follow symlinks
    #[arg(short = 'L', long)]
    follow_links: bool,

    /// Exclude patterns (comma-separated)
    #[arg(short, long, value_delimiter = ',')]
    exclude: Vec<String>,

    /// Include only specific languages (comma-separated)
    #[arg(short, long, value_delimiter = ',')]
    languages: Vec<String>,

    /// Disable parallel processing (slower but uses less memory)
    #[arg(long)]
    no_parallel: bool,

    /// Show progress
    #[arg(short, long)]
    progress: bool,
    
    /// Run internal benchmark
    #[arg(long)]
    benchmark: bool,
    
    /// Disable cache, force fresh full scan
    #[arg(long)]
    fresh: bool,
    
    /// Rebuild cache (clear and rebuild from scratch)
    #[arg(long)]
    rebuild_cache: bool,
}

fn main() {
    let args = Args::parse();

    // Run benchmark if requested
    if args.benchmark {
        run_benchmark();
        return;
    }

    let start = Instant::now();
    let path = &args.path;

    if !path.exists() {
        eprintln!("Error: Path '{}' does not exist", path.display());
        std::process::exit(1);
    }

    // 获取绝对路径作为缓存根目录
    let root_path = path.canonicalize().unwrap_or_else(|_| path.clone());
    
    // 初始化增量缓存上下文（默认启用）
    let use_cache = !args.fresh;
    let cache_file_path = root_path.join(cache::CACHE_FILENAME);
    let mut cache_ctx = if use_cache {
        if args.rebuild_cache {
            // 强制重建缓存
            if cache_file_path.exists() {
                let _ = std::fs::remove_file(&cache_file_path);
            }
        }
        Some(IncrementalContext::new(&root_path, !args.rebuild_cache))
    } else {
        None
    };

    // 阶段 1: 收集文件
    let collect_start = Instant::now();
    let files_to_analyze = collect_files(path, &args);
    let collect_time = collect_start.elapsed();
    
    if files_to_analyze.is_empty() {
        println!("No code files found in '{}'", path.display());
        return;
    }

    let total_files_found = files_to_analyze.len();
    
    if args.progress {
        eprintln!("Found {} files in {:?}", total_files_found, collect_time);
    }

    // 阶段 2: 检查缓存，分离出需要处理的文件
    let mut cached_results: Vec<(PathBuf, Language, FileStats)> = Vec::new();
    let mut files_to_process: Vec<(PathBuf, Language, u64)> = Vec::with_capacity(files_to_analyze.len());
    
    if let Some(ref mut ctx) = cache_ctx {
        for (file_path, lang, size) in files_to_analyze {
            // 尝试从缓存获取
            if let Some((cached_lang, cached_stats)) = ctx.try_get(&file_path) {
                cached_results.push((file_path, cached_lang, cached_stats));
                ctx.record_hit();
            } else {
                files_to_process.push((file_path, lang, size));
                ctx.record_new();
            }
        }
        
        if args.progress {
            let hit_rate = ctx.hit_rate();
            if hit_rate == 100.0 {
                eprintln!("  ✓ {} files cached (100%)", ctx.hits);
            } else if hit_rate > 0.0 {
                eprintln!(
                    "  Cache: {} cached, {} new ({:.0}% hit)",
                    ctx.hits,
                    files_to_process.len(),
                    hit_rate
                );
            } else {
                eprintln!("  Building cache for {} files...", files_to_process.len());
            }
        }
    } else {
        files_to_process = files_to_analyze;
    }

    // 阶段 3: 按文件大小排序，优化负载均衡
    files_to_process.par_sort_unstable_by(|a, b| b.2.cmp(&a.2));

    // 阶段 4: 处理需要统计的文件
    let process_start = Instant::now();
    let errors: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let processed_count = Arc::new(AtomicUsize::new(0));
    let num_to_process = files_to_process.len();

    let new_results = if args.no_parallel || num_to_process < 100 {
        process_sequential(
            files_to_process,
            &args,
            &errors,
            &processed_count,
            num_to_process.max(1),
        )
    } else {
        process_parallel(
            files_to_process,
            &args,
            &errors,
            &processed_count,
            num_to_process,
        )
    };

    let process_time = process_start.elapsed();

    // 阶段 5: 合并结果并更新缓存
    let mut total_stats = TotalStats::new();
    let mut file_details: Vec<(PathBuf, Language, FileStats)> = Vec::new();

    // 添加缓存的结果
    for (path, lang, stats) in cached_results {
        total_stats.add_file(lang, &stats);
        if args.files {
            file_details.push((path, lang, stats));
        }
    }

    // 添加新处理的结果并更新缓存
    for (path, lang, stats) in &new_results {
        total_stats.add_file(*lang, stats);
        if args.files {
            file_details.push((path.clone(), *lang, stats.clone()));
        }
        
        // 更新缓存
        if let Some(ref mut ctx) = cache_ctx {
            ctx.update(path.clone(), *lang, stats.clone());
        }
    }

    // 保存缓存
    let mut cache_created = false;
    if let Some(ref ctx) = cache_ctx {
        match ctx.save() {
            Ok(()) => {
                // 检测是否是首次创建缓存
                if ctx.misses > 0 && ctx.hits == 0 {
                    cache_created = true;
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to save cache to {}: {}", 
                    cache_file_path.display(), e);
            }
        }
    }

    let elapsed_ms = start.elapsed().as_millis() as u64;
    let errors_vec = match Arc::try_unwrap(errors) {
        Ok(mutex) => mutex.into_inner().unwrap_or_default(),
        Err(arc) => arc.lock().map(|g| g.clone()).unwrap_or_default(),
    };

    if args.progress {
        let processed = new_results.len();
        let cached = total_files_found - processed;
        if cached > 0 && processed > 0 {
            eprintln!(
                "\n✓ {} files from cache, {} processed in {:?}",
                cached, processed, process_time
            );
        } else if processed > 0 {
            eprintln!("\nProcessed {} files in {:?}", processed, process_time);
        }
        
        // 显示缓存信息
        if cache_created {
            eprintln!("  Cache saved to {}", cache::CACHE_FILENAME);
        } else if args.rebuild_cache && use_cache {
            eprintln!("  Cache rebuilt ({})", cache::CACHE_FILENAME);
        }
    }

    let summary = Summary::new(total_stats, elapsed_ms, errors_vec);

    // 输出结果
    match args.format.as_str() {
        "json" => output_json(&summary, &file_details),
        "csv" => output_csv(&summary, &file_details),
        _ => output_table(&summary, &file_details, args.files),
    }
}

/// 串行处理（小批量优化）
fn process_sequential(
    files: Vec<(PathBuf, Language, u64)>,
    args: &Args,
    errors: &Arc<Mutex<Vec<String>>>,
    processed_count: &Arc<AtomicUsize>,
    total_files: usize,
) -> Vec<(PathBuf, Language, FileStats)> {
    files
        .into_iter()
        .filter_map(|(path, lang, _size)| {
            let result = count_file(&path, lang);
            
            if args.progress {
                let count = processed_count.fetch_add(1, Ordering::Relaxed) + 1;
                if count % 100 == 0 || count == total_files {
                    eprint!("\rProgress: {}/{} ({:.1}%)", 
                        count, total_files, 
                        count as f64 / total_files as f64 * 100.0);
                }
            }
            
            match result {
                Ok(stats) => Some((path, lang, stats)),
                Err(e) => {
                    errors.lock().unwrap().push(format!("{}: {}", path.display(), e));
                    None
                }
            }
        })
        .collect()
}

/// 并行处理（大批量优化）
fn process_parallel(
    files: Vec<(PathBuf, Language, u64)>,
    args: &Args,
    errors: &Arc<Mutex<Vec<String>>>,
    processed_count: &Arc<AtomicUsize>,
    total_files: usize,
) -> Vec<(PathBuf, Language, FileStats)> {
    // 使用批量处理减少任务切换开销
    files
        .into_par_iter()
        .map(|(path, lang, _size)| {
            let result = count_file(&path, lang);
            
            if args.progress {
                let count = processed_count.fetch_add(1, Ordering::Relaxed) + 1;
                if count % 100 == 0 || count == total_files {
                    eprint!("\rProgress: {}/{} ({:.1}%)", 
                        count, total_files,
                        count as f64 / total_files as f64 * 100.0);
                }
            }
            
            match result {
                Ok(stats) => Some((path, lang, stats)),
                Err(e) => {
                    errors.lock().unwrap().push(format!("{}: {}", path.display(), e));
                    None
                }
            }
        })
        .filter_map(|x| x)
        .collect()
}

/// 收集文件并预获取元数据
fn collect_files(path: &Path, args: &Args) -> Vec<(PathBuf, Language, u64)> {
    let mut files: Vec<(PathBuf, Language, u64)> = Vec::with_capacity(1024);

    if path.is_file() {
        if let Ok(metadata) = std::fs::metadata(path) {
            let lang = detect_language(path);
            if lang != Language::Unknown {
                files.push((path.to_path_buf(), lang, metadata.len()));
            }
        }
        return files;
    }

    // 构建 ignore  walker
    let mut builder = ignore::WalkBuilder::new(path);
    builder
        .hidden(!args.hidden)
        .follow_links(args.follow_links)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .ignore(true)
        .threads(num_cpus::get().min(8)); // 限制遍历线程数

    for pattern in &args.exclude {
        builder.add_ignore(pattern);
    }

    let walker = builder.build_parallel();
    
    // 使用 channel 收集结果
    let (tx, rx) = std::sync::mpsc::channel();
    
    walker.run(|| {
        let tx = tx.clone();
        Box::new(move |entry| {
            if let Ok(entry) = entry {
                let path = entry.path();
                if !path.is_file() {
                    return ignore::WalkState::Continue;
                }
                
                let lang = detect_language(path);
                if lang == Language::Unknown {
                    return ignore::WalkState::Continue;
                }
                
                // 预获取文件大小
                if let Ok(metadata) = entry.metadata() {
                    let _ = tx.send((path.to_path_buf(), lang, metadata.len()));
                }
            }
            ignore::WalkState::Continue
        })
    });
    
    drop(tx);
    
    // 收集结果
    for (path, lang, size) in rx {
        // 过滤语言
        if !args.languages.is_empty() {
            let lang_str = lang.as_str().to_lowercase();
            if !args.languages.iter().any(|l| lang_str == l.to_lowercase()) {
                continue;
            }
        }
        files.push((path, lang, size));
    }

    files
}

fn output_table(summary: &Summary, file_details: &[(PathBuf, Language, FileStats)], show_files: bool) {
    summary.print_summary();

    if show_files && !file_details.is_empty() {
        println!("\n{:=<100}", "");
        println!(" {:<50} {:<15} {:>10} {:>10}", "File", "Language", "Lines", "Code");
        println!("{:-<100}", "");

        // 按路径排序
        let mut sorted = file_details.to_vec();
        sorted.sort_by(|a, b| a.0.cmp(&b.0));

        for (path, lang, stats) in sorted {
            let display_path = if path.as_os_str().len() > 48 {
                format!("...{}", &path.to_string_lossy()[path.to_string_lossy().len() - 45..])
            } else {
                path.to_string_lossy().to_string()
            };
            println!(" {:<50} {:<15} {:>10} {:>10}",
                display_path, lang.as_str(), stats.lines, stats.code_lines);
        }
        println!("{:=<100}", "");
    }
}

fn output_json(summary: &Summary, file_details: &[(PathBuf, Language, FileStats)]) {
    #[derive(serde::Serialize)]
    struct JsonOutput {
        summary: serde_json::Value,
        languages: Vec<serde_json::Value>,
        files: Option<Vec<serde_json::Value>>,
    }

    let languages: Vec<_> = summary.total_stats.sorted_by_code_lines()
        .into_iter()
        .map(|l| serde_json::json!({
            "name": l.language.as_str(),
            "files": l.files,
            "lines": l.lines,
            "code": l.code_lines,
            "comments": l.comment_lines,
            "blanks": l.blank_lines,
            "bytes": l.bytes,
        }))
        .collect();

    let files = if !file_details.is_empty() {
        Some(file_details.iter().map(|(p, l, s)| {
            serde_json::json!({
                "path": p.to_string_lossy(),
                "language": l.as_str(),
                "lines": s.lines,
                "code": s.code_lines,
                "comments": s.comment_lines,
                "blanks": s.blank_lines,
                "bytes": s.bytes,
            })
        }).collect())
    } else {
        None
    };

    let output = JsonOutput {
        summary: serde_json::json!({
            "total_files": summary.total_stats.total_files,
            "total_lines": summary.total_stats.total_lines,
            "total_code": summary.total_stats.total_code_lines,
            "total_comments": summary.total_stats.total_comment_lines,
            "total_blanks": summary.total_stats.total_blank_lines,
            "total_bytes": summary.total_stats.total_bytes,
            "elapsed_ms": summary.elapsed_ms,
            "error_count": summary.errors.len(),
        }),
        languages,
        files,
    };

    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

fn output_csv(summary: &Summary, file_details: &[(PathBuf, Language, FileStats)]) {
    println!("Type,Name,Files,Lines,Code,Comments,Blanks,Bytes");

    for lang in summary.total_stats.sorted_by_code_lines() {
        println!("Language,{},{},{},{},{},{},{}",
            lang.language.as_str(),
            lang.files,
            lang.lines,
            lang.code_lines,
            lang.comment_lines,
            lang.blank_lines,
            lang.bytes,
        );
    }

    println!("Total,Total,{},{},{},{},{},{}",
        summary.total_stats.total_files,
        summary.total_stats.total_lines,
        summary.total_stats.total_code_lines,
        summary.total_stats.total_comment_lines,
        summary.total_stats.total_blank_lines,
        summary.total_stats.total_bytes,
    );

    if !file_details.is_empty() {
        println!("\nFile,Language,Lines,Code,Comments,Blanks,Bytes");
        for (path, lang, stats) in file_details {
            println!("{},{},{},{},{},{},{}",
                path.to_string_lossy(),
                lang.as_str(),
                stats.lines,
                stats.code_lines,
                stats.comment_lines,
                stats.blank_lines,
                stats.bytes,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_language() {
        assert_eq!(detect_language(Path::new("test.rs")), Language::Rust);
        assert_eq!(detect_language(Path::new("test.py")), Language::Python);
        assert_eq!(detect_language(Path::new("test.js")), Language::JavaScript);
        assert_eq!(detect_language(Path::new("Dockerfile")), Language::Dockerfile);
    }
}
