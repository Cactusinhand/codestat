use crate::language::Language;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct FileStats {
    pub lines: usize,
    pub code_lines: usize,
    pub comment_lines: usize,
    pub blank_lines: usize,
    pub bytes: u64,
}

impl FileStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, other: &FileStats) {
        self.lines += other.lines;
        self.code_lines += other.code_lines;
        self.comment_lines += other.comment_lines;
        self.blank_lines += other.blank_lines;
        self.bytes += other.bytes;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageStats {
    pub language: Language,
    pub files: usize,
    pub lines: usize,
    pub code_lines: usize,
    pub comment_lines: usize,
    pub blank_lines: usize,
    pub bytes: u64,
}

impl LanguageStats {
    pub fn new(language: Language) -> Self {
        Self {
            language,
            files: 0,
            lines: 0,
            code_lines: 0,
            comment_lines: 0,
            blank_lines: 0,
            bytes: 0,
        }
    }

    pub fn add_file_stats(&mut self, stats: &FileStats) {
        self.files += 1;
        self.lines += stats.lines;
        self.code_lines += stats.code_lines;
        self.comment_lines += stats.comment_lines;
        self.blank_lines += stats.blank_lines;
        self.bytes += stats.bytes;
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TotalStats {
    pub total_files: usize,
    pub total_lines: usize,
    pub total_code_lines: usize,
    pub total_comment_lines: usize,
    pub total_blank_lines: usize,
    pub total_bytes: u64,
    pub by_language: HashMap<Language, LanguageStats>,
}

impl TotalStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_file(&mut self, language: Language, stats: &FileStats) {
        self.total_files += 1;
        self.total_lines += stats.lines;
        self.total_code_lines += stats.code_lines;
        self.total_comment_lines += stats.comment_lines;
        self.total_blank_lines += stats.blank_lines;
        self.total_bytes += stats.bytes;

        self.by_language
            .entry(language)
            .or_insert_with(|| LanguageStats::new(language))
            .add_file_stats(stats);
    }

    pub fn sorted_by_code_lines(&self) -> Vec<&LanguageStats> {
        let mut stats: Vec<&LanguageStats> = self.by_language.values().collect();
        stats.sort_by(|a, b| b.code_lines.cmp(&a.code_lines));
        stats
    }

    pub fn sorted_by_files(&self) -> Vec<&LanguageStats> {
        let mut stats: Vec<&LanguageStats> = self.by_language.values().collect();
        stats.sort_by(|a, b| b.files.cmp(&a.files));
        stats
    }
}

impl fmt::Display for TotalStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "\n{:=<80}", "")?;
        writeln!(f, " {:<20} {:>10} {:>10} {:>10} {:>10} {:>10}", 
            "Language", "Files", "Lines", "Code", "Comments", "Blanks")?;
        writeln!(f, "{:-<80}", "")?;

        for lang_stats in self.sorted_by_code_lines() {
            writeln!(f, " {:<20} {:>10} {:>10} {:>10} {:>10} {:>10}",
                lang_stats.language.as_str(),
                lang_stats.files,
                lang_stats.lines,
                lang_stats.code_lines,
                lang_stats.comment_lines,
                lang_stats.blank_lines,
            )?;
        }

        writeln!(f, "{:-<80}", "")?;
        writeln!(f, " {:<20} {:>10} {:>10} {:>10} {:>10} {:>10}",
            "Total",
            self.total_files,
            self.total_lines,
            self.total_code_lines,
            self.total_comment_lines,
            self.total_blank_lines,
        )?;
        writeln!(f, "{:=<80}", "")?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Summary {
    pub total_stats: TotalStats,
    pub elapsed_ms: u64,
    pub errors: Vec<String>,
}

impl Summary {
    pub fn new(total_stats: TotalStats, elapsed_ms: u64, errors: Vec<String>) -> Self {
        Self {
            total_stats,
            elapsed_ms,
            errors,
        }
    }

    pub fn print_summary(&self) {
        println!("{}", self.total_stats);
        
        let seconds = self.elapsed_ms as f64 / 1000.0;
        let files_per_sec = self.total_stats.total_files as f64 / seconds;
        let lines_per_sec = self.total_stats.total_lines as f64 / seconds;
        
        println!("\n Statistics completed in {:.3}s ({:.0} files/s, {:.0} lines/s)",
            seconds, files_per_sec, lines_per_sec);
        
        if !self.errors.is_empty() {
            println!("\n {} errors occurred:", self.errors.len());
            for (i, err) in self.errors.iter().take(5).enumerate() {
                println!("   {}. {}", i + 1, err);
            }
            if self.errors.len() > 5 {
                println!("   ... and {} more", self.errors.len() - 5);
            }
        }
    }
}
