use std::collections::HashMap;
use std::sync::Arc;

use crate::analysis::css_analyzer::CssAnalyzer;
use crate::analysis::{LanguageAnalyzer, TypeScriptJavaScriptAnalyzer};

/// Metadata for a registered analyzer plugin.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub extensions: Vec<String>,
}

/// A registry mapping file extensions to analyzer implementations.
pub struct AnalyzerRegistry {
    map: HashMap<String, Arc<dyn LanguageAnalyzer>>,
    plugins: Vec<PluginInfo>,
}

impl AnalyzerRegistry {
    pub fn all_extensions(&self) -> Vec<String> {
        let mut exts: Vec<String> = self.map.keys().cloned().collect();
        exts.sort();
        exts.dedup();
        exts
    }

    pub fn resolve(&self, extension: &str) -> Option<Arc<dyn LanguageAnalyzer>> {
        self.map.get(&extension.to_lowercase()).cloned()
    }

    pub fn plugin_list(&self) -> &[PluginInfo] {
        &self.plugins
    }

    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }
}

/// Builds the default analyzer registry with all built-in plugins.
pub fn build_registry() -> AnalyzerRegistry {
    let mut reg = AnalyzerRegistry {
        map: HashMap::new(),
        plugins: Vec::new(),
    };
    reg.register(TypeScriptJavaScriptAnalyzer);
    reg.register(CssAnalyzer);
    reg
}

impl AnalyzerRegistry {
    fn register(&mut self, analyzer: impl LanguageAnalyzer + 'static) {
        let info = PluginInfo {
            name: analyzer.name().to_string(),
            version: analyzer.version().to_string(),
            description: analyzer.description().to_string(),
            extensions: analyzer
                .supported_extensions()
                .iter()
                .map(|e| e.to_string())
                .collect(),
        };
        let shared: Arc<dyn LanguageAnalyzer> = Arc::new(analyzer);
        for ext in shared.supported_extensions() {
            self.map.insert(ext.to_string(), Arc::clone(&shared));
        }
        self.plugins.push(info);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_registry_has_ts_js_and_css() {
        let reg = build_registry();
        assert!(reg.plugin_count() >= 2);
    }

    #[test]
    fn resolves_ts_extension() {
        let reg = build_registry();
        assert!(reg.resolve("ts").is_some());
        assert!(reg.resolve("tsx").is_some());
        assert!(reg.resolve("js").is_some());
        assert!(reg.resolve("css").is_some());
    }

    #[test]
    fn returns_none_for_unknown_extension() {
        let reg = build_registry();
        assert!(reg.resolve("png").is_none());
        assert!(reg.resolve("md").is_none());
    }

    #[test]
    fn all_extensions_includes_ts_and_css() {
        let reg = build_registry();
        let exts = reg.all_extensions();
        assert!(exts.contains(&"ts".to_string()));
        assert!(exts.contains(&"css".to_string()));
    }

    #[test]
    fn plugin_info_contains_metadata() {
        let reg = build_registry();
        let plugins = reg.plugin_list();
        let ts = plugins
            .iter()
            .find(|p| p.name == "TypeScript/JavaScript")
            .unwrap();
        assert!(!ts.extensions.is_empty());
        assert!(!ts.description.is_empty());
    }
}
