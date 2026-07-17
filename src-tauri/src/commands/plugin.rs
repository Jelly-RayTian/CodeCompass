use crate::analysis::plugin::{build_registry, PluginInfo};

#[tauri::command]
pub fn get_plugin_info() -> Vec<PluginInfo> {
    let registry = build_registry();
    registry.plugin_list().to_vec()
}
