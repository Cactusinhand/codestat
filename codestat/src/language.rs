use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Java,
    C,
    Cpp,
    Go,
    Ruby,
    Php,
    Swift,
    Kotlin,
    Scala,
    Html,
    Css,
    Scss,
    Shell,
    Bash,
    Sql,
    Markdown,
    Json,
    Yaml,
    Xml,
    Toml,
    VimScript,
    Lua,
    Perl,
    R,
    Matlab,
    Dart,
    Elixir,
    Erlang,
    Haskell,
    Clojure,
    Lisp,
    FSharp,
    CSharp,
    ObjectiveC,
    Groovy,
    Dockerfile,
    Makefile,
    CMake,
    Zig,
    Nim,
    Crystal,
    Julia,
    Fortran,
    Cobol,
    Pascal,
    Assembly,
    Unknown,
}

impl Language {
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::Rust => "Rust",
            Language::Python => "Python",
            Language::JavaScript => "JavaScript",
            Language::TypeScript => "TypeScript",
            Language::Java => "Java",
            Language::C => "C",
            Language::Cpp => "C++",
            Language::Go => "Go",
            Language::Ruby => "Ruby",
            Language::Php => "PHP",
            Language::Swift => "Swift",
            Language::Kotlin => "Kotlin",
            Language::Scala => "Scala",
            Language::Html => "HTML",
            Language::Css => "CSS",
            Language::Scss => "SCSS",
            Language::Shell => "Shell",
            Language::Bash => "Bash",
            Language::Sql => "SQL",
            Language::Markdown => "Markdown",
            Language::Json => "JSON",
            Language::Yaml => "YAML",
            Language::Xml => "XML",
            Language::Toml => "TOML",
            Language::VimScript => "Vim Script",
            Language::Lua => "Lua",
            Language::Perl => "Perl",
            Language::R => "R",
            Language::Matlab => "MATLAB",
            Language::Dart => "Dart",
            Language::Elixir => "Elixir",
            Language::Erlang => "Erlang",
            Language::Haskell => "Haskell",
            Language::Clojure => "Clojure",
            Language::Lisp => "Lisp",
            Language::FSharp => "F#",
            Language::CSharp => "C#",
            Language::ObjectiveC => "Objective-C",
            Language::Groovy => "Groovy",
            Language::Dockerfile => "Dockerfile",
            Language::Makefile => "Makefile",
            Language::CMake => "CMake",
            Language::Zig => "Zig",
            Language::Nim => "Nim",
            Language::Crystal => "Crystal",
            Language::Julia => "Julia",
            Language::Fortran => "Fortran",
            Language::Cobol => "COBOL",
            Language::Pascal => "Pascal",
            Language::Assembly => "Assembly",
            Language::Unknown => "Unknown",
        }
    }

    pub fn get_comment_syntax(&self) -> CommentSyntax {
        match self {
            Language::Rust => CommentSyntax::new("//", Some(("/*", "*/"))),
            Language::C => CommentSyntax::new("//", Some(("/*", "*/"))),
            Language::Cpp => CommentSyntax::new("//", Some(("/*", "*/"))),
            Language::Java => CommentSyntax::new("//", Some(("/*", "*/"))),
            Language::JavaScript => CommentSyntax::new("//", Some(("/*", "*/"))),
            Language::TypeScript => CommentSyntax::new("//", Some(("/*", "*/"))),
            Language::Go => CommentSyntax::new("//", Some(("/*", "*/"))),
            Language::Swift => CommentSyntax::new("//", Some(("/*", "*/"))),
            Language::Kotlin => CommentSyntax::new("//", Some(("/*", "*/"))),
            Language::Scala => CommentSyntax::new("//", Some(("/*", "*/"))),
            Language::Php => CommentSyntax::new("//", Some(("/*", "*/"))),
            Language::CSharp => CommentSyntax::new("//", Some(("/*", "*/"))),
            Language::ObjectiveC => CommentSyntax::new("//", Some(("/*", "*/"))),
            Language::Dart => CommentSyntax::new("//", Some(("/*", "*/"))),
            Language::Zig => CommentSyntax::new("//", Some(("/*", "*/"))),
            Language::Python => CommentSyntax::new("#", Some(("\"\"\"", "\"\"\""))),
            Language::Ruby => CommentSyntax::new("#", Some(("=begin", "=end"))),
            Language::Perl => CommentSyntax::new("#", None),
            Language::Shell => CommentSyntax::new("#", None),
            Language::Bash => CommentSyntax::new("#", None),
            Language::Makefile => CommentSyntax::new("#", None),
            Language::Dockerfile => CommentSyntax::new("#", None),
            Language::Yaml => CommentSyntax::new("#", None),
            Language::Toml => CommentSyntax::new("#", None),
            Language::Sql => CommentSyntax::new("--", Some(("/*", "*/"))),
            Language::Lua => CommentSyntax::new("--", Some(("--[[", "]]"))),
            Language::Haskell => CommentSyntax::new("--", Some(("{-", "-}"))),
            Language::Lisp | Language::Clojure => CommentSyntax::new(";", Some(("#|", "|#"))),
            Language::Elixir => CommentSyntax::new("#", None),
            Language::Erlang => CommentSyntax::new("%", None),
            Language::Html => CommentSyntax::new_html(),
            Language::Xml => CommentSyntax::new_html(),
            Language::Css => CommentSyntax::new("", Some(("/*", "*/"))),
            Language::Scss => CommentSyntax::new("//", Some(("/*", "*/"))),
            Language::Assembly => CommentSyntax::new(";", None),
            Language::Fortran => CommentSyntax::new("!", None),
            Language::Matlab => CommentSyntax::new("%", Some(("%{", "}%"))),
            Language::R => CommentSyntax::new("#", None),
            Language::Julia => CommentSyntax::new("#", Some(("#=", "=#"))),
            _ => CommentSyntax::none(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommentSyntax {
    pub line: Option<&'static str>,
    pub block_start: Option<&'static str>,
    pub block_end: Option<&'static str>,
}

impl CommentSyntax {
    pub const fn new(line: &'static str, block: Option<(&'static str, &'static str)>) -> Self {
        let (block_start, block_end) = match block {
            Some((s, e)) => (Some(s), Some(e)),
            None => (None, None),
        };
        Self {
            line: Some(line),
            block_start,
            block_end,
        }
    }

    pub const fn new_html() -> Self {
        Self {
            line: None,
            block_start: Some("<!--"),
            block_end: Some("-->"),
        }
    }

    pub const fn none() -> Self {
        Self {
            line: None,
            block_start: None,
            block_end: None,
        }
    }
}

fn get_extension_map() -> &'static HashMap<&'static str, Language> {
    static MAP: OnceLock<HashMap<&str, Language>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert("rs", Language::Rust);
        m.insert("py", Language::Python);
        m.insert("js", Language::JavaScript);
        m.insert("mjs", Language::JavaScript);
        m.insert("cjs", Language::JavaScript);
        m.insert("ts", Language::TypeScript);
        m.insert("mts", Language::TypeScript);
        m.insert("cts", Language::TypeScript);
        m.insert("tsx", Language::TypeScript);
        m.insert("jsx", Language::JavaScript);
        m.insert("java", Language::Java);
        m.insert("c", Language::C);
        m.insert("h", Language::C);
        m.insert("cpp", Language::Cpp);
        m.insert("cc", Language::Cpp);
        m.insert("cxx", Language::Cpp);
        m.insert("hpp", Language::Cpp);
        m.insert("hh", Language::Cpp);
        m.insert("hxx", Language::Cpp);
        m.insert("go", Language::Go);
        m.insert("rb", Language::Ruby);
        m.insert("erb", Language::Ruby);
        m.insert("php", Language::Php);
        m.insert("phtml", Language::Php);
        m.insert("swift", Language::Swift);
        m.insert("kt", Language::Kotlin);
        m.insert("kts", Language::Kotlin);
        m.insert("scala", Language::Scala);
        m.insert("sc", Language::Scala);
        m.insert("html", Language::Html);
        m.insert("htm", Language::Html);
        m.insert("css", Language::Css);
        m.insert("scss", Language::Scss);
        m.insert("sass", Language::Scss);
        m.insert("sh", Language::Shell);
        m.insert("bash", Language::Bash);
        m.insert("zsh", Language::Shell);
        m.insert("fish", Language::Shell);
        m.insert("sql", Language::Sql);
        m.insert("md", Language::Markdown);
        m.insert("markdown", Language::Markdown);
        m.insert("mdx", Language::Markdown);
        m.insert("json", Language::Json);
        m.insert("yaml", Language::Yaml);
        m.insert("yml", Language::Yaml);
        m.insert("xml", Language::Xml);
        m.insert("toml", Language::Toml);
        m.insert("vim", Language::VimScript);
        m.insert("lua", Language::Lua);
        m.insert("pl", Language::Perl);
        m.insert("pm", Language::Perl);
        m.insert("r", Language::R);
        m.insert("m", Language::Matlab);
        m.insert("matlab", Language::Matlab);
        m.insert("dart", Language::Dart);
        m.insert("ex", Language::Elixir);
        m.insert("exs", Language::Elixir);
        m.insert("erl", Language::Erlang);
        m.insert("hrl", Language::Erlang);
        m.insert("hs", Language::Haskell);
        m.insert("lhs", Language::Haskell);
        m.insert("clj", Language::Clojure);
        m.insert("cljs", Language::Clojure);
        m.insert("cljc", Language::Clojure);
        m.insert("lisp", Language::Lisp);
        m.insert("lsp", Language::Lisp);
        m.insert("fs", Language::FSharp);
        m.insert("fsx", Language::FSharp);
        m.insert("fsi", Language::FSharp);
        m.insert("cs", Language::CSharp);
        m.insert("csx", Language::CSharp);
        m.insert("m", Language::ObjectiveC);
        m.insert("mm", Language::ObjectiveC);
        m.insert("groovy", Language::Groovy);
        m.insert("gvy", Language::Groovy);
        m.insert("dockerfile", Language::Dockerfile);
        m.insert("mk", Language::Makefile);
        m.insert("makefile", Language::Makefile);
        m.insert("cmake", Language::CMake);
        m.insert("zig", Language::Zig);
        m.insert("nim", Language::Nim);
        m.insert("cr", Language::Crystal);
        m.insert("jl", Language::Julia);
        m.insert("f90", Language::Fortran);
        m.insert("f95", Language::Fortran);
        m.insert("f03", Language::Fortran);
        m.insert("cob", Language::Cobol);
        m.insert("cbl", Language::Cobol);
        m.insert("pas", Language::Pascal);
        m.insert("pp", Language::Pascal);
        m.insert("asm", Language::Assembly);
        m.insert("s", Language::Assembly);
        m.insert("S", Language::Assembly);
        m
    })
}

fn get_filename_map() -> &'static HashMap<&'static str, Language> {
    static MAP: OnceLock<HashMap<&str, Language>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert("dockerfile", Language::Dockerfile);
        m.insert("makefile", Language::Makefile);
        m.insert("gemfile", Language::Ruby);
        m.insert("rakefile", Language::Ruby);
        m.insert("cargo.toml", Language::Toml);
        m.insert("cmakeLists.txt", Language::CMake);
        m
    })
}

pub fn detect_language(path: &Path) -> Language {
    // First check filename
    if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
        let filename_lower = filename.to_lowercase();
        if let Some(&lang) = get_filename_map().get(filename_lower.as_str()) {
            return lang;
        }
    }

    // Then check extension
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        let ext_lower = ext.to_lowercase();
        if let Some(&lang) = get_extension_map().get(ext_lower.as_str()) {
            return lang;
        }
    }

    // Check full filename for files like Dockerfile, Makefile
    if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
        let filename_lower = filename.to_lowercase();
        if let Some(&lang) = get_filename_map().get(filename_lower.as_str()) {
            return lang;
        }
    }

    Language::Unknown
}

#[allow(dead_code)]
pub fn is_code_file(path: &Path) -> bool {
    detect_language(path) != Language::Unknown
}
