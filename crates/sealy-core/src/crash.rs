//! Crash-report analysis.
//!
//! The launcher reads the newest crash report (or the end of `latest.log` when
//! there is none) and turns common failure signatures into short findings with
//! suggestions the user can act on.

use std::path::{Path, PathBuf};

/// One actionable finding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrashFinding {
    /// Chinese description.
    pub zh: String,
    /// English description.
    pub en: String,
}

impl CrashFinding {
    fn new(zh: &str, en: &str) -> Self {
        Self {
            zh: zh.to_owned(),
            en: en.to_owned(),
        }
    }

    /// Description in the requested language.
    pub fn text(&self, lang: crate::config::Language) -> &str {
        match lang {
            crate::config::Language::ZhCn => &self.zh,
            crate::config::Language::EnUs => &self.en,
        }
    }
}

/// Analysis result for one launch failure.
#[derive(Debug, Clone, Default)]
pub struct CrashReport {
    /// File that was analysed, when one was found.
    pub source: Option<PathBuf>,
    /// One-line summary ("Description: ..." or the last exception).
    pub summary: String,
    /// Findings with suggestions, most useful first.
    pub findings: Vec<CrashFinding>,
    /// Top of the report, for the expandable raw view.
    pub excerpt: Vec<String>,
}

impl CrashReport {
    /// Whether any report was found at all.
    pub fn found(&self) -> bool {
        self.source.is_some()
    }
}

/// Looks for the newest crash report and analyses it.
///
/// `game_dir` is the shared `.minecraft` folder; per-instance
/// `crash-reports` folders are searched as well.
pub fn analyze(game_dir: &Path) -> CrashReport {
    if let Some(path) = newest_report(game_dir) {
        let text = std::fs::read_to_string(&path).unwrap_or_default();
        let mut report = analyze_text(&text);
        report.source = Some(path);
        return report;
    }

    let log = game_dir.join("logs").join("latest.log");
    if let Ok(text) = std::fs::read_to_string(&log) {
        let mut report = analyze_text(&text);
        report.source = Some(log);
        return report;
    }

    CrashReport::default()
}

/// Newest `crash-reports/*.txt` under the game folder or an instance folder.
fn newest_report(game_dir: &Path) -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    collect_reports(&game_dir.join("crash-reports"), &mut candidates);
    if let Ok(instances) = std::fs::read_dir(game_dir.join("instances")) {
        for entry in instances.flatten() {
            collect_reports(&entry.path().join("crash-reports"), &mut candidates);
        }
    }

    candidates.into_iter().max_by_key(|path| {
        std::fs::metadata(path)
            .and_then(|meta| meta.modified())
            .ok()
    })
}

fn collect_reports(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "txt") {
            out.push(path);
        }
    }
}

/// Analyses report text without touching the file system.
pub fn analyze_text(text: &str) -> CrashReport {
    let mut report = CrashReport::default();
    let mut seen = std::collections::HashSet::new();
    let mut add = |finding: CrashFinding, report: &mut CrashReport| {
        if seen.insert(finding.en.clone()) {
            report.findings.push(finding);
        }
    };

    for line in text.lines() {
        let trimmed = line.trim();
        let lower = trimmed.to_ascii_lowercase();

        if report.summary.is_empty() && trimmed.starts_with("Description:") {
            report.summary = trimmed.trim_start_matches("Description:").trim().to_owned();
        }
        if report.summary.is_empty()
            && (lower.contains("exception") || lower.contains("error:"))
            && (trimmed.starts_with("java.")
                || trimmed.starts_with("net.")
                || trimmed.contains(": "))
        {
            report.summary = trimmed.chars().take(180).collect();
        }

        if lower.contains("outofmemoryerror") {
            add(
                CrashFinding::new(
                    "内存不足：调大实例或全局最大内存，或关闭其它占用内存的程序",
                    "Out of memory: raise the max memory or close other memory-hungry apps",
                ),
                &mut report,
            );
        }
        if lower.contains("unsupportedclassversionerror") {
            add(
                CrashFinding::new(
                    "Java 版本不匹配：检查实例使用的 Java 版本是否满足该 Minecraft/加载器要求",
                    "Java version mismatch: check the Java version against the loader/Minecraft requirements",
                ),
                &mut report,
            );
        }
        if lower.contains("nosuchmethoderror")
            || lower.contains("nosuchfielderror")
            || lower.contains("abstractmethoderror")
        {
            add(
                CrashFinding::new(
                    "模组与游戏/加载器版本不兼容（NoSuchMethod/NoSuchField），请更换对应版本的模组",
                    "A mod does not match the game/loader version (NoSuchMethod/NoSuchField)",
                ),
                &mut report,
            );
        }
        if lower.contains("classnotfoundexception") || lower.contains("noclassdeffounderror") {
            add(
                CrashFinding::new(
                    "缺少类文件：通常是缺少前置模组、模组文件损坏或使用了错误的加载器",
                    "Missing classes: usually a missing dependency, a corrupt jar or the wrong loader",
                ),
                &mut report,
            );
        }
        if lower.contains("duplicate mod") || lower.contains("duplicatemodsfoundexception") {
            add(
                CrashFinding::new(
                    "存在重复的模组，请在模组列表里删除重复文件",
                    "Duplicate mods found: remove the duplicated jars",
                ),
                &mut report,
            );
        }
        if lower.contains("mixin apply failed") || lower.contains("mixinapplyerror") {
            add(
                CrashFinding::new(
                    "Mixin 注入失败：常见于模组冲突或模组版本过旧，建议先禁用最近新增的模组",
                    "Mixin apply failed: usually a mod conflict or outdated mod; disable recent additions",
                ),
                &mut report,
            );
        }
        if lower.contains("modresolutionexception") || lower.contains("incompatible mod set") {
            add(
                CrashFinding::new(
                    "模组依赖无法解析：按提示补全或移除冲突模组，可用二分法排查",
                    "Mod resolution failed: add the missing dependency or remove the conflicting mod",
                ),
                &mut report,
            );
        }
        if lower.contains("pixel format not accelerated")
            || lower.contains("glfw error")
            || lower.contains("opengl")
        {
            add(
                CrashFinding::new(
                    "图形驱动/OpenGL 问题：更新显卡驱动，或关闭光影后再试",
                    "Graphics driver/OpenGL issue: update the GPU driver or disable shaders",
                ),
                &mut report,
            );
        }
        if lower.contains("accessdeniedexception") || lower.contains("securityexception") {
            add(
                CrashFinding::new(
                    "文件访问被拒绝：检查游戏目录权限，或把启动器放到可写目录",
                    "Access denied: check game folder permissions or move the launcher to a writable folder",
                ),
                &mut report,
            );
        }
        if lower.contains("failed to download") || lower.contains("couldn't download") {
            add(
                CrashFinding::new(
                    "文件下载失败或缺失：在实例里重新安装该版本/加载器，或检查网络与镜像",
                    "Files failed to download: reinstall the version/loader or check the network/mirror",
                ),
                &mut report,
            );
        }
        if lower.contains("java.lang.nullpointerexception") {
            add(
                CrashFinding::new(
                    "空指针异常：多为某个模组与环境组合触发，查看下面的堆栈定位模组",
                    "NullPointerException: often triggered by a specific mod; check the stack below",
                ),
                &mut report,
            );
        }
    }

    if report.findings.is_empty() && report.found_hint() {
        report.findings.push(CrashFinding::new(
            "没有识别出已知错误模式，建议查看原始崩溃报告与最新日志",
            "No known pattern matched; check the raw crash report and latest log",
        ));
    }

    report.excerpt = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .take(40)
        .map(|line| line.trim_end().to_owned())
        .collect();
    report
}

impl CrashReport {
    fn found_hint(&self) -> bool {
        !self.summary.is_empty() || !self.excerpt.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_out_of_memory() {
        let report = analyze_text(
            "Description: Rendering screen\njava.lang.OutOfMemoryError: Java heap space\n",
        );
        assert_eq!(report.summary, "Rendering screen");
        assert!(
            report
                .findings
                .iter()
                .any(|item| item.en.contains("Out of memory"))
        );
    }

    #[test]
    fn detects_mixin_and_duplicate_mods() {
        let report = analyze_text(
            "Mixin apply failed: mixin.example.json\nFound a duplicate mod example at a.jar\n",
        );
        assert!(report.findings.iter().any(|item| item.en.contains("Mixin")));
        assert!(
            report
                .findings
                .iter()
                .any(|item| item.en.contains("Duplicate mods"))
        );
    }

    #[test]
    fn falls_back_to_generic_tip() {
        let report = analyze_text("Description: Something odd happened\nat a.b.C(D.java:1)\n");
        assert_eq!(report.findings.len(), 1);
        assert!(report.findings[0].en.contains("No known pattern"));
        assert!(report.source.is_none());
    }

    #[test]
    fn empty_text_has_no_findings() {
        let report = analyze_text("");
        assert!(report.findings.is_empty());
        assert!(!report.found());
    }
}
