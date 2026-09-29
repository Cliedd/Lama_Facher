pub mod java;
pub mod rust;
pub mod traits;

use java::JavaLanguage;
use rust::RustLanguage;
use std::sync::Arc;
use traits::LanguageAdapter;

pub fn get_adapter(lang: &str) -> Option<Arc<dyn LanguageAdapter>> {
    match lang.to_lowercase().as_str() {
        "java" => Some(Arc::new(JavaLanguage::new())),
        "rust" | "rs" => Some(Arc::new(RustLanguage::new())),
        _ => None,
    }
}
