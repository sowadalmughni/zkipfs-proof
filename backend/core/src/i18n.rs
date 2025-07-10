//! Internationalization (i18n) support for zkIPFS-Proof
//! 
//! This module provides comprehensive internationalization capabilities
//! including message translation, locale detection, and formatting.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use serde::{Deserialize, Serialize};

/// Supported languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    English,
    Spanish,
    French,
    German,
    Chinese,
    Japanese,
    Korean,
    Portuguese,
    Russian,
    Arabic,
    Hindi,
    Italian,
}

impl Language {
    /// Get language code (ISO 639-1)
    pub fn code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Spanish => "es",
            Language::French => "fr",
            Language::German => "de",
            Language::Chinese => "zh",
            Language::Japanese => "ja",
            Language::Korean => "ko",
            Language::Portuguese => "pt",
            Language::Russian => "ru",
            Language::Arabic => "ar",
            Language::Hindi => "hi",
            Language::Italian => "it",
        }
    }

    /// Get language name in English
    pub fn name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Spanish => "Spanish",
            Language::French => "French",
            Language::German => "German",
            Language::Chinese => "Chinese",
            Language::Japanese => "Japanese",
            Language::Korean => "Korean",
            Language::Portuguese => "Portuguese",
            Language::Russian => "Russian",
            Language::Arabic => "Arabic",
            Language::Hindi => "Hindi",
            Language::Italian => "Italian",
        }
    }

    /// Get language name in native script
    pub fn native_name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Spanish => "Español",
            Language::French => "Français",
            Language::German => "Deutsch",
            Language::Chinese => "中文",
            Language::Japanese => "日本語",
            Language::Korean => "한국어",
            Language::Portuguese => "Português",
            Language::Russian => "Русский",
            Language::Arabic => "العربية",
            Language::Hindi => "हिन्दी",
            Language::Italian => "Italiano",
        }
    }

    /// Parse language from code
    pub fn from_code(code: &str) -> Option<Self> {
        match code.to_lowercase().as_str() {
            "en" => Some(Language::English),
            "es" => Some(Language::Spanish),
            "fr" => Some(Language::French),
            "de" => Some(Language::German),
            "zh" => Some(Language::Chinese),
            "ja" => Some(Language::Japanese),
            "ko" => Some(Language::Korean),
            "pt" => Some(Language::Portuguese),
            "ru" => Some(Language::Russian),
            "ar" => Some(Language::Arabic),
            "hi" => Some(Language::Hindi),
            "it" => Some(Language::Italian),
            _ => None,
        }
    }

    /// Check if language is right-to-left
    pub fn is_rtl(&self) -> bool {
        matches!(self, Language::Arabic)
    }
}

/// Translation message with pluralization support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationMessage {
    /// Singular form
    pub singular: String,
    /// Plural form (optional)
    pub plural: Option<String>,
    /// Context for disambiguation (optional)
    pub context: Option<String>,
}

impl TranslationMessage {
    /// Create a simple message
    pub fn simple(message: &str) -> Self {
        Self {
            singular: message.to_string(),
            plural: None,
            context: None,
        }
    }

    /// Create a message with plural form
    pub fn with_plural(singular: &str, plural: &str) -> Self {
        Self {
            singular: singular.to_string(),
            plural: Some(plural.to_string()),
            context: None,
        }
    }

    /// Create a message with context
    pub fn with_context(message: &str, context: &str) -> Self {
        Self {
            singular: message.to_string(),
            plural: None,
            context: Some(context.to_string()),
        }
    }

    /// Get the appropriate form based on count
    pub fn get_form(&self, count: i32) -> &str {
        if count == 1 {
            &self.singular
        } else if let Some(plural) = &self.plural {
            plural
        } else {
            &self.singular
        }
    }
}

/// Translation catalog for a specific language
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationCatalog {
    /// Language for this catalog
    pub language: Language,
    /// Translation messages
    pub messages: HashMap<String, TranslationMessage>,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

impl TranslationCatalog {
    /// Create a new translation catalog
    pub fn new(language: Language) -> Self {
        Self {
            language,
            messages: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a simple translation
    pub fn add_translation(&mut self, key: &str, message: &str) {
        self.messages.insert(key.to_string(), TranslationMessage::simple(message));
    }

    /// Add a translation with plural form
    pub fn add_plural_translation(&mut self, key: &str, singular: &str, plural: &str) {
        self.messages.insert(key.to_string(), TranslationMessage::with_plural(singular, plural));
    }

    /// Get a translation
    pub fn get_translation(&self, key: &str) -> Option<&TranslationMessage> {
        self.messages.get(key)
    }

    /// Get translated text
    pub fn translate(&self, key: &str, count: Option<i32>) -> Option<String> {
        let message = self.get_translation(key)?;
        let count = count.unwrap_or(1);
        Some(message.get_form(count).to_string())
    }

    /// Get translated text with interpolation
    pub fn translate_with_args(&self, key: &str, args: &HashMap<String, String>, count: Option<i32>) -> Option<String> {
        let mut text = self.translate(key, count)?;
        
        // Simple interpolation: replace {key} with values
        for (arg_key, arg_value) in args {
            let placeholder = format!("{{{}}}", arg_key);
            text = text.replace(&placeholder, arg_value);
        }
        
        Some(text)
    }
}

/// Internationalization manager
pub struct I18nManager {
    /// Current language
    current_language: Arc<RwLock<Language>>,
    /// Translation catalogs
    catalogs: Arc<RwLock<HashMap<Language, TranslationCatalog>>>,
    /// Fallback language
    fallback_language: Language,
}

impl I18nManager {
    /// Create a new i18n manager
    pub fn new() -> Self {
        let mut manager = Self {
            current_language: Arc::new(RwLock::new(Language::English)),
            catalogs: Arc::new(RwLock::new(HashMap::new())),
            fallback_language: Language::English,
        };
        
        // Load default English translations
        manager.load_default_translations();
        manager
    }

    /// Set current language
    pub fn set_language(&self, language: Language) {
        if let Ok(mut current) = self.current_language.write() {
            *current = language;
        }
    }

    /// Get current language
    pub fn get_language(&self) -> Language {
        self.current_language.read().unwrap_or_else(|_| self.fallback_language).clone()
    }

    /// Add translation catalog
    pub fn add_catalog(&self, catalog: TranslationCatalog) {
        if let Ok(mut catalogs) = self.catalogs.write() {
            catalogs.insert(catalog.language, catalog);
        }
    }

    /// Translate a message
    pub fn translate(&self, key: &str) -> String {
        self.translate_with_count(key, None)
    }

    /// Translate a message with count for pluralization
    pub fn translate_with_count(&self, key: &str, count: Option<i32>) -> String {
        let current_lang = self.get_language();
        
        // Try current language first
        if let Ok(catalogs) = self.catalogs.read() {
            if let Some(catalog) = catalogs.get(&current_lang) {
                if let Some(translation) = catalog.translate(key, count) {
                    return translation;
                }
            }
            
            // Fallback to English
            if current_lang != self.fallback_language {
                if let Some(catalog) = catalogs.get(&self.fallback_language) {
                    if let Some(translation) = catalog.translate(key, count) {
                        return translation;
                    }
                }
            }
        }
        
        // Return key if no translation found
        key.to_string()
    }

    /// Translate with arguments
    pub fn translate_with_args(&self, key: &str, args: &HashMap<String, String>) -> String {
        self.translate_with_args_and_count(key, args, None)
    }

    /// Translate with arguments and count
    pub fn translate_with_args_and_count(&self, key: &str, args: &HashMap<String, String>, count: Option<i32>) -> String {
        let current_lang = self.get_language();
        
        // Try current language first
        if let Ok(catalogs) = self.catalogs.read() {
            if let Some(catalog) = catalogs.get(&current_lang) {
                if let Some(translation) = catalog.translate_with_args(key, args, count) {
                    return translation;
                }
            }
            
            // Fallback to English
            if current_lang != self.fallback_language {
                if let Some(catalog) = catalogs.get(&self.fallback_language) {
                    if let Some(translation) = catalog.translate_with_args(key, args, count) {
                        return translation;
                    }
                }
            }
        }
        
        // Return key if no translation found
        key.to_string()
    }

    /// Detect language from environment
    pub fn detect_language_from_env(&self) -> Language {
        // Check LANG environment variable
        if let Ok(lang) = std::env::var("LANG") {
            let lang_code = lang.split('.').next().unwrap_or(&lang);
            let lang_code = lang_code.split('_').next().unwrap_or(lang_code);
            if let Some(language) = Language::from_code(lang_code) {
                return language;
            }
        }
        
        // Check LC_ALL
        if let Ok(lang) = std::env::var("LC_ALL") {
            let lang_code = lang.split('.').next().unwrap_or(&lang);
            let lang_code = lang_code.split('_').next().unwrap_or(lang_code);
            if let Some(language) = Language::from_code(lang_code) {
                return language;
            }
        }
        
        // Default to English
        Language::English
    }

    /// Load default English translations
    fn load_default_translations(&mut self) {
        let mut catalog = TranslationCatalog::new(Language::English);
        
        // CLI messages
        catalog.add_translation("cli.generate.success", "Proof generated successfully");
        catalog.add_translation("cli.generate.error", "Failed to generate proof");
        catalog.add_translation("cli.verify.success", "Proof verified successfully");
        catalog.add_translation("cli.verify.error", "Proof verification failed");
        catalog.add_translation("cli.file.not_found", "File not found: {path}");
        catalog.add_translation("cli.invalid.format", "Invalid file format");
        catalog.add_translation("cli.network.error", "Network connection error");
        catalog.add_translation("cli.ipfs.error", "IPFS operation failed");
        
        // Web UI messages
        catalog.add_translation("web.upload.drag_drop", "Drag and drop files here or click to browse");
        catalog.add_translation("web.upload.processing", "Processing file...");
        catalog.add_translation("web.proof.generating", "Generating zero-knowledge proof...");
        catalog.add_translation("web.proof.complete", "Proof generation complete");
        catalog.add_translation("web.verify.title", "Verify Proof");
        catalog.add_translation("web.verify.paste", "Paste proof JSON here");
        catalog.add_translation("web.verify.valid", "Proof is valid");
        catalog.add_translation("web.verify.invalid", "Proof is invalid");
        
        // Error messages
        catalog.add_translation("error.file.too_large", "File is too large (max {max_size})");
        catalog.add_translation("error.unsupported.format", "Unsupported file format: {format}");
        catalog.add_translation("error.network.timeout", "Network request timed out");
        catalog.add_translation("error.permission.denied", "Permission denied");
        catalog.add_translation("error.disk.space", "Insufficient disk space");
        
        // Success messages
        catalog.add_translation("success.file.uploaded", "File uploaded successfully");
        catalog.add_translation("success.proof.shared", "Proof shared successfully");
        catalog.add_translation("success.settings.saved", "Settings saved");
        
        // Pluralization examples
        catalog.add_plural_translation("files.count", "{count} file", "{count} files");
        catalog.add_plural_translation("proofs.generated", "{count} proof generated", "{count} proofs generated");
        
        self.add_catalog(catalog);
    }
}

impl Default for I18nManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Global i18n instance
static mut GLOBAL_I18N: Option<I18nManager> = None;
static INIT: std::sync::Once = std::sync::Once::new();

/// Initialize global i18n
pub fn init_i18n() -> &'static I18nManager {
    unsafe {
        INIT.call_once(|| {
            GLOBAL_I18N = Some(I18nManager::new());
        });
        GLOBAL_I18N.as_ref().unwrap()
    }
}

/// Get global i18n instance
pub fn get_i18n() -> &'static I18nManager {
    unsafe {
        GLOBAL_I18N.as_ref().expect("I18n not initialized. Call init_i18n() first.")
    }
}

/// Convenience macro for translation
#[macro_export]
macro_rules! t {
    ($key:expr) => {
        $crate::i18n::get_i18n().translate($key)
    };
    
    ($key:expr, $count:expr) => {
        $crate::i18n::get_i18n().translate_with_count($key, Some($count))
    };
    
    ($key:expr, $($arg_key:expr => $arg_value:expr),+) => {
        {
            let mut args = std::collections::HashMap::new();
            $(
                args.insert($arg_key.to_string(), $arg_value.to_string());
            )+
            $crate::i18n::get_i18n().translate_with_args($key, &args)
        }
    };
}

/// Date and time formatting for different locales
pub struct LocaleFormatter {
    language: Language,
}

impl LocaleFormatter {
    pub fn new(language: Language) -> Self {
        Self { language }
    }

    /// Format date according to locale
    pub fn format_date(&self, date: &chrono::DateTime<chrono::Utc>) -> String {
        match self.language {
            Language::English => date.format("%B %d, %Y").to_string(),
            Language::Spanish => date.format("%d de %B de %Y").to_string(),
            Language::French => date.format("%d %B %Y").to_string(),
            Language::German => date.format("%d. %B %Y").to_string(),
            Language::Chinese => date.format("%Y年%m月%d日").to_string(),
            Language::Japanese => date.format("%Y年%m月%d日").to_string(),
            Language::Korean => date.format("%Y년 %m월 %d일").to_string(),
            _ => date.format("%Y-%m-%d").to_string(), // ISO format as fallback
        }
    }

    /// Format time according to locale
    pub fn format_time(&self, time: &chrono::DateTime<chrono::Utc>) -> String {
        match self.language {
            Language::English => time.format("%I:%M %p").to_string(),
            Language::Spanish | Language::French | Language::German | Language::Italian => {
                time.format("%H:%M").to_string()
            }
            Language::Chinese | Language::Japanese | Language::Korean => {
                time.format("%H:%M").to_string()
            }
            _ => time.format("%H:%M").to_string(),
        }
    }

    /// Format file size according to locale
    pub fn format_file_size(&self, bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        match self.language {
            Language::French => format!("{:.1} {}", size, UNITS[unit_index]).replace('.', ","),
            Language::German => format!("{:.1} {}", size, UNITS[unit_index]).replace('.', ","),
            _ => format!("{:.1} {}", size, UNITS[unit_index]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_codes() {
        assert_eq!(Language::English.code(), "en");
        assert_eq!(Language::Spanish.code(), "es");
        assert_eq!(Language::from_code("en"), Some(Language::English));
        assert_eq!(Language::from_code("invalid"), None);
    }

    #[test]
    fn test_translation_message() {
        let msg = TranslationMessage::with_plural("file", "files");
        assert_eq!(msg.get_form(1), "file");
        assert_eq!(msg.get_form(2), "files");
    }

    #[test]
    fn test_translation_catalog() {
        let mut catalog = TranslationCatalog::new(Language::English);
        catalog.add_translation("hello", "Hello, World!");
        catalog.add_plural_translation("item", "item", "items");
        
        assert_eq!(catalog.translate("hello", None), Some("Hello, World!".to_string()));
        assert_eq!(catalog.translate("item", Some(1)), Some("item".to_string()));
        assert_eq!(catalog.translate("item", Some(2)), Some("items".to_string()));
    }

    #[test]
    fn test_i18n_manager() {
        let manager = I18nManager::new();
        
        // Test default English translation
        let translation = manager.translate("cli.generate.success");
        assert_eq!(translation, "Proof generated successfully");
        
        // Test missing translation returns key
        let missing = manager.translate("missing.key");
        assert_eq!(missing, "missing.key");
    }

    #[test]
    fn test_locale_formatter() {
        let formatter = LocaleFormatter::new(Language::English);
        let date = chrono::Utc::now();
        
        let formatted_date = formatter.format_date(&date);
        assert!(!formatted_date.is_empty());
        
        let formatted_size = formatter.format_file_size(1024);
        assert_eq!(formatted_size, "1.0 KB");
    }
}

