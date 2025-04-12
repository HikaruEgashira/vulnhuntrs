use crate::security_patterns::SecurityRiskPatterns;
use anyhow::Result;
use std::{
    fs::{read_dir, read_to_string, File},
    io::{BufRead, BufReader, Result as IoResult},
    path::{Path, PathBuf},
};
#[derive(Default)]
pub struct LanguageExclusions {
    pub file_patterns: Vec<String>,
}

pub struct RepoOps {
    repo_path: PathBuf,
    gitignore_patterns: Vec<String>,
    language_exclusions: LanguageExclusions,
    security_patterns: Option<SecurityRiskPatterns>,
    supported_extensions: Vec<String>,
    code_parser: crate::parser::CodeParser,
    parser_initialized: bool,
}

impl RepoOps {
    /// RepoOpsインスタンスを生成する。
    pub fn new(repo_path: PathBuf) -> Self {
        let gitignore_patterns = Self::read_gitignore(&repo_path).unwrap_or_default();

        let language_exclusions = LanguageExclusions {
            file_patterns: vec!["test_".to_string(), "conftest".to_string()],
        };

        let code_parser = crate::parser::CodeParser::new().unwrap();
        let supported_extensions = vec![
            "py".to_string(),
            "js".to_string(),
            "jsx".to_string(),
            "ts".to_string(),
            "tsx".to_string(),
            "rs".to_string(),
            "go".to_string(),
            "java".to_string(),
        ];

        let security_patterns = Some(SecurityRiskPatterns::new(
            crate::security_patterns::Language::Other,
        ));
        let security_patterns = Some(SecurityRiskPatterns::new(
            crate::security_patterns::Language::Other,
        ));
        let security_patterns = Some(SecurityRiskPatterns::new(
            crate::security_patterns::Language::Other,
        ));

        Self {
            repo_path,
            gitignore_patterns,
            language_exclusions,
            security_patterns,
            supported_extensions,
            code_parser,
            parser_initialized: false,
        }
    }

    /// セキュリティパターン該当ファイルを起点にContextを構築する。
    pub fn collect_context_for_security_pattern(
        &mut self,
        file_path: &std::path::Path,
    ) -> anyhow::Result<crate::parser::Context> {
        // TODO: parser.build_context_from_fileを利用してContextを構築
        self.code_parser.build_context_from_file(file_path)
    }

    /// リポジトリの.gitignoreパターンを読み込む。
    fn read_gitignore(repo_path: &Path) -> IoResult<Vec<String>> {
        let gitignore_path = repo_path.join(".gitignore");
        if !gitignore_path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(gitignore_path)?;
        let reader = BufReader::new(file);
        let mut patterns = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                patterns.push(trimmed.to_string());
            };
        }

        Ok(patterns)
    }

    /// ディレクトリを再帰的に走査し、各ファイルにコールバックを適用する。
    fn visit_dirs(&self, dir: &Path, cb: &mut dyn FnMut(&Path)) -> std::io::Result<()> {
        if dir.is_dir() {
            for entry in read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    self.visit_dirs(&path, cb)?;
                } else {
                    cb(&path);
                }
            }
        }
        Ok(())
    }

    /// パターンに基づきパスを除外すべきか判定する。
    fn should_exclude_path(&self, path: &Path) -> bool {
        if let Ok(relative_path) = path.strip_prefix(&self.repo_path) {
            let relative_str = relative_path.to_string_lossy();

            for pattern in &self.gitignore_patterns {
                if Self::matches_gitignore_pattern(&relative_str, pattern) {
                    return true;
                }
            }

            if let Some(file_name) = path.file_name() {
                let file_name = file_name.to_string_lossy().to_lowercase();
                if self
                    .language_exclusions
                    .file_patterns
                    .iter()
                    .any(|pattern| file_name.contains(pattern))
                {
                    return true;
                }
            }
        }
        false
    }

    /// パスが.gitignoreパターンに一致するか判定する。
    fn matches_gitignore_pattern(path: &str, pattern: &str) -> bool {
        let pattern = pattern.trim_start_matches('/');
        let path = path.trim_start_matches('/');

        if pattern.starts_with('*') {
            path.ends_with(&pattern[1..])
        } else if pattern.ends_with('*') {
            path.starts_with(&pattern[..pattern.len() - 1])
        } else {
            path == pattern || path.starts_with(&format!("{}/", pattern))
        }
    }

    /// リポジトリ内の関連ソースファイル一覧を返す。
    pub fn get_relevant_files(&self) -> Vec<PathBuf> {
        let mut files = Vec::new();

        let mut callback = |path: &Path| {
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if !self.supported_extensions.contains(&ext_str) {
                    return;
                }

                if self.should_exclude_path(path) {
                    return;
                }

                files.push(path.to_path_buf());
            }
        };

        if let Err(e) = self.visit_dirs(&self.repo_path, &mut callback) {
            eprintln!("ディレクトリの走査中にエラーが発生しました: {}", e);
        }

        files
    }

    /// ネットワーク関連パターンに該当するファイル一覧を返す。
    pub fn get_network_related_files(&self, files: &[PathBuf]) -> Vec<PathBuf> {
        let Some(security_patterns) = &self.security_patterns else {
            return Vec::new();
        };

        let mut network_files = Vec::new();
        for file_path in files {
            if let Ok(content) = read_to_string(file_path) {
                if security_patterns.matches(&content) {
                    network_files.push(file_path.clone());
                }
            }
        }

        network_files
    }

    /// 指定パスに基づき解析対象ファイル一覧を返す。
    pub fn get_files_to_analyze(&self, analyze_path: Option<PathBuf>) -> Result<Vec<PathBuf>> {
        let path_to_analyze = analyze_path.unwrap_or_else(|| self.repo_path.clone());

        if path_to_analyze.is_file() {
            if let Some(ext) = path_to_analyze.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if self.supported_extensions.contains(&ext_str) {
                    return Ok(vec![path_to_analyze]);
                }
            }
            Ok(vec![])
        } else if path_to_analyze.is_dir() {
            let mut files = Vec::new();
            let mut callback = |path: &Path| {
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if self.supported_extensions.contains(&ext_str) {
                        files.push(path.to_path_buf());
                    }
                }
            };

            self.visit_dirs(&path_to_analyze, &mut callback)?;
            Ok(files)
        } else {
            anyhow::bail!(
                "指定された解析パスが存在しません: {}",
                path_to_analyze.display()
            )
        }
    }

    /// コードパーサーにファイルを読み込む。
    pub fn parse_repo_files(&mut self, analyze_path: Option<PathBuf>) -> Result<()> {
        let files = self.get_files_to_analyze(analyze_path)?;
        for file in &files {
            self.code_parser.add_file(file)?;
        }
        self.parser_initialized = true;

        Ok(())
    }

    /// リポジトリ内で定義を検索する。
    pub fn find_definition_in_repo(
        &mut self,
        name: &str,
        source_file: &Path,
    ) -> anyhow::Result<Option<(PathBuf, crate::parser::Definition)>> {
        if !self.parser_initialized {
            self.parse_repo_files(None)?;
        }

        self.code_parser.find_definition(name, source_file)
    }

    /// リポジトリ内で参照を検索する。
    pub fn find_references_in_repo(
        &mut self,
        name: &str,
    ) -> anyhow::Result<Vec<(PathBuf, crate::parser::Definition)>> {
        if !self.parser_initialized {
            self.parse_repo_files(None)?;
        }
        self.code_parser.find_references(name)
    }
    /// 指定ファイルをCodeParserに追加
    pub fn add_file_to_parser(&mut self, path: &std::path::Path) -> anyhow::Result<()> {
        self.code_parser.add_file(path)
    }
}
