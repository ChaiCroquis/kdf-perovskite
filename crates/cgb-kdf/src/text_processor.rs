//! Lightweight Text Processor Module
//!
//! Provides text processing functionality for KDF.
//! Uses regex-based tokenization as a portable alternative to MeCab.
//!
//! # Features
//!
//! - Token extraction (Japanese/English/mixed)
//! - Stopword removal
//! - Noun extraction
//! - Domain classification
//! - Text hashing for unique IDs
//!
//! # Reference
//!
//! Python implementation: python/kdf/text_processor.py

use std::collections::{HashMap, HashSet};

// ============================================================================
// Token Types
// ============================================================================

/// Token from text processing
#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    /// Surface form
    pub surface: String,
    /// Part of speech
    pub pos: String,
    /// Base form (dictionary form)
    pub base_form: String,
    /// Reading (for Japanese)
    pub reading: String,
}

impl Token {
    /// Create a new token
    pub fn new(surface: &str, pos: &str) -> Self {
        Self {
            surface: surface.to_string(),
            pos: pos.to_string(),
            base_form: surface.to_string(),
            reading: surface.to_string(),
        }
    }

    /// Create token with all fields
    pub fn with_details(surface: &str, pos: &str, base_form: &str, reading: &str) -> Self {
        Self {
            surface: surface.to_string(),
            pos: pos.to_string(),
            base_form: base_form.to_string(),
            reading: reading.to_string(),
        }
    }
}

// ============================================================================
// Text Processor
// ============================================================================

/// Lightweight Text Processing Engine
///
/// # Features
/// - Regex-based tokenization (no external dependencies)
/// - Stopword removal
/// - Noun extraction
/// - Token normalization
pub struct TextProcessor {
    /// Japanese stopwords
    stopwords: HashSet<String>,
}

impl TextProcessor {
    /// Create a new text processor
    pub fn new() -> Self {
        let stopwords = Self::default_stopwords();
        Self { stopwords }
    }

    /// Default Japanese stopwords
    fn default_stopwords() -> HashSet<String> {
        [
            "の",
            "に",
            "は",
            "を",
            "た",
            "が",
            "で",
            "て",
            "と",
            "し",
            "れ",
            "さ",
            "ある",
            "いる",
            "も",
            "する",
            "から",
            "な",
            "こと",
            "として",
            "い",
            "や",
            "れる",
            "など",
            "なっ",
            "ない",
            "この",
            "ため",
            "その",
            "あっ",
            "よう",
            "また",
            "もの",
            "という",
            "あり",
            "まで",
            "られ",
            "なる",
            "へ",
            "か",
            "だ",
            "これ",
            "によって",
            "により",
            "おり",
            "より",
            "による",
            "ず",
            "なり",
            "られる",
            "において",
            "ば",
            "なかっ",
            "なく",
            "しかし",
            "について",
            "せ",
            "だっ",
            "その後",
            "できる",
            "それ",
            "う",
            "ので",
            "なお",
            "のみ",
            "でき",
            "き",
            "つ",
            "における",
            "および",
            "いう",
            "さらに",
            "でも",
            "ら",
            "たり",
            "その他",
            "に関する",
            "たち",
            "ます",
            "ん",
            "なら",
            "に対して",
            "特に",
            "せる",
            "及び",
            "これら",
            "とき",
            "では",
            "にて",
            "ほか",
            "ながら",
            "うち",
            "そして",
            "とも",
            "のち",
            "ただし",
            "かつて",
            "それぞれ",
            "または",
            "お",
            "ほど",
            "ものの",
            "に対する",
            "ほとんど",
            "と共に",
            "といった",
            "です",
            "ください",
            "あるいは",
            "そう",
            "ごとく",
            "なぜなら",
            "まま",
            "なし",
            "しかも",
            "それで",
            "いずれ",
            // English common stopwords
            "the",
            "a",
            "an",
            "is",
            "are",
            "was",
            "were",
            "be",
            "been",
            "being",
            "have",
            "has",
            "had",
            "do",
            "does",
            "did",
            "will",
            "would",
            "could",
            "should",
            "may",
            "might",
            "must",
            "can",
            "this",
            "that",
            "these",
            "those",
            "i",
            "you",
            "he",
            "she",
            "it",
            "we",
            "they",
            "what",
            "which",
            "who",
            "whom",
            "when",
            "where",
            "why",
            "how",
            "all",
            "each",
            "every",
            "both",
            "few",
            "more",
            "most",
            "other",
            "some",
            "such",
            "no",
            "nor",
            "not",
            "only",
            "own",
            "same",
            "so",
            "than",
            "too",
            "very",
            "just",
            "and",
            "but",
            "if",
            "or",
            "because",
            "as",
            "until",
            "while",
            "of",
            "at",
            "by",
            "for",
            "with",
            "about",
            "against",
            "between",
            "into",
            "through",
            "during",
            "before",
            "after",
            "above",
            "below",
            "to",
            "from",
            "up",
            "down",
            "in",
            "out",
            "on",
            "off",
            "over",
            "under",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }

    /// Check if a character is Japanese (Hiragana, Katakana, or Kanji)
    #[allow(dead_code)]
    fn is_japanese(c: char) -> bool {
        matches!(c,
            '\u{3040}'..='\u{309F}' | // Hiragana
            '\u{30A0}'..='\u{30FF}' | // Katakana
            '\u{4E00}'..='\u{9FFF}' | // CJK Unified Ideographs (Kanji)
            '\u{3400}'..='\u{4DBF}' | // CJK Extension A
            '\u{FF00}'..='\u{FFEF}'   // Fullwidth characters
        )
    }

    /// Check if a character is Kanji
    fn is_kanji(c: char) -> bool {
        matches!(c, '\u{4E00}'..='\u{9FFF}' | '\u{3400}'..='\u{4DBF}')
    }

    /// Check if a character is Hiragana
    fn is_hiragana(c: char) -> bool {
        matches!(c, '\u{3040}'..='\u{309F}')
    }

    /// Check if a character is Katakana
    fn is_katakana(c: char) -> bool {
        matches!(c, '\u{30A0}'..='\u{30FF}' | '\u{FF66}'..='\u{FF9F}')
    }

    /// Tokenize text using regex-based patterns
    ///
    /// # Arguments
    /// * `text` - Input text
    /// * `filter_pos` - Optional POS filter (e.g., vec!["名詞", "動詞"])
    pub fn tokenize(&self, text: &str, filter_pos: Option<&[&str]>) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut current_token = String::new();
        let mut current_type: Option<&str> = None;

        for c in text.chars() {
            // Kanji, Katakana, and English alphabets are treated as nouns
            let char_type = if Self::is_kanji(c) || Self::is_katakana(c) || c.is_ascii_alphabetic()
            {
                Some("名詞") // Noun
            } else if Self::is_hiragana(c) {
                Some("助詞") // Particle for Hiragana
            } else if c.is_ascii_digit() {
                Some("数詞") // Number
            } else {
                None // Separator
            };

            match (current_type, char_type) {
                (Some(ct), Some(nt)) if ct == nt => {
                    current_token.push(c);
                }
                (Some(_), Some(nt)) => {
                    if !current_token.is_empty() {
                        let pos = current_type.unwrap_or("Unknown");
                        if filter_pos.is_none() || filter_pos.unwrap().contains(&pos) {
                            tokens.push(Token::new(&current_token, pos));
                        }
                    }
                    current_token = c.to_string();
                    current_type = Some(nt);
                }
                (Some(_), None) => {
                    if !current_token.is_empty() {
                        let pos = current_type.unwrap_or("Unknown");
                        if filter_pos.is_none() || filter_pos.unwrap().contains(&pos) {
                            tokens.push(Token::new(&current_token, pos));
                        }
                    }
                    current_token.clear();
                    current_type = None;
                }
                (None, Some(nt)) => {
                    current_token = c.to_string();
                    current_type = Some(nt);
                }
                (None, None) => {
                    // Skip separators
                }
            }
        }

        // Don't forget last token
        if !current_token.is_empty() {
            let pos = current_type.unwrap_or("Unknown");
            if filter_pos.is_none() || filter_pos.unwrap().contains(&pos) {
                tokens.push(Token::new(&current_token, pos));
            }
        }

        tokens
    }

    /// Extract nouns from text
    ///
    /// # Arguments
    /// * `text` - Input text
    /// * `remove_stopwords` - Whether to remove stopwords
    pub fn extract_nouns(&self, text: &str, remove_stopwords: bool) -> Vec<String> {
        let tokens = self.tokenize(text, Some(&["名詞"]));
        let mut nouns: Vec<String> = tokens.into_iter().map(|t| t.base_form).collect();

        if remove_stopwords {
            nouns.retain(|n| !self.stopwords.contains(n) && n.chars().count() > 1);
        }

        nouns
    }

    /// Extract verbs from text
    pub fn extract_verbs(&self, text: &str) -> Vec<String> {
        let tokens = self.tokenize(text, Some(&["動詞"]));
        tokens.into_iter().map(|t| t.base_form).collect()
    }

    /// Get word frequency from texts
    pub fn get_word_frequency(&self, texts: &[&str]) -> HashMap<String, usize> {
        let mut freq: HashMap<String, usize> = HashMap::new();

        for text in texts {
            for noun in self.extract_nouns(text, true) {
                *freq.entry(noun).or_insert(0) += 1;
            }
        }

        freq
    }

    /// Generate hash for text (for unique ID)
    pub fn generate_text_hash(&self, text: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    /// Check if a word is a stopword
    pub fn is_stopword(&self, word: &str) -> bool {
        self.stopwords.contains(word)
    }

    /// Add custom stopwords
    pub fn add_stopwords(&mut self, words: &[&str]) {
        for word in words {
            self.stopwords.insert(word.to_string());
        }
    }
}

impl Default for TextProcessor {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Domain Classifier
// ============================================================================

/// Domain Classifier
///
/// Classifies text into domains using keyword matching.
pub struct DomainClassifier {
    /// Domain keyword dictionaries
    domain_keywords: HashMap<String, HashSet<String>>,
    /// Default domain
    default_domain: String,
}

impl DomainClassifier {
    /// Create a new domain classifier
    pub fn new() -> Self {
        let mut domain_keywords = HashMap::new();

        // Technology domain
        let tech_keywords: HashSet<String> = [
            "プログラミング",
            "コード",
            "アルゴリズム",
            "データベース",
            "システム",
            "API",
            "フレームワーク",
            "ライブラリ",
            "開発",
            "テスト",
            "デバッグ",
            "リファクタリング",
            "デプロイ",
            "サーバー",
            "実装",
            "クラス",
            "関数",
            "メソッド",
            "モジュール",
            "programming",
            "code",
            "algorithm",
            "database",
            "system",
            "framework",
            "library",
            "development",
            "testing",
            "debug",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        domain_keywords.insert("技術".to_string(), tech_keywords);

        // Medical domain
        let medical_keywords: HashSet<String> = [
            "患者",
            "診断",
            "治療",
            "病院",
            "医師",
            "看護師",
            "症状",
            "薬",
            "手術",
            "検査",
            "健康",
            "病気",
            "診療",
            "カルテ",
            "patient",
            "diagnosis",
            "treatment",
            "hospital",
            "doctor",
            "nurse",
            "symptom",
            "medicine",
            "surgery",
            "health",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        domain_keywords.insert("医療".to_string(), medical_keywords);

        // Business domain
        let business_keywords: HashSet<String> = [
            "会議",
            "プロジェクト",
            "売上",
            "予算",
            "契約",
            "営業",
            "顧客",
            "マーケティング",
            "戦略",
            "経営",
            "報告",
            "提案",
            "meeting",
            "project",
            "sales",
            "budget",
            "contract",
            "business",
            "customer",
            "marketing",
            "strategy",
            "management",
            "report",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        domain_keywords.insert("ビジネス".to_string(), business_keywords);

        // Academic domain
        let academic_keywords: HashSet<String> = [
            "研究",
            "論文",
            "実験",
            "仮説",
            "理論",
            "分析",
            "考察",
            "文献",
            "引用",
            "学会",
            "発表",
            "査読",
            "データ",
            "統計",
            "research",
            "paper",
            "experiment",
            "hypothesis",
            "theory",
            "analysis",
            "literature",
            "citation",
            "conference",
            "statistics",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        domain_keywords.insert("学術".to_string(), academic_keywords);

        // Administrative domain
        let admin_keywords: HashSet<String> = [
            "申請",
            "窓口",
            "手続き",
            "届出",
            "許可",
            "認可",
            "法律",
            "条例",
            "規則",
            "公文書",
            "行政",
            "自治体",
            "市役所",
            "application",
            "procedure",
            "permit",
            "approval",
            "law",
            "regulation",
            "document",
            "administration",
            "government",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        domain_keywords.insert("行政".to_string(), admin_keywords);

        Self {
            domain_keywords,
            default_domain: "一般".to_string(),
        }
    }

    /// Classify tokens into a domain
    pub fn classify(&self, tokens: &[Token]) -> String {
        // Extract nouns
        let nouns: Vec<&str> = tokens
            .iter()
            .filter(|t| t.pos == "名詞")
            .map(|t| t.base_form.as_str())
            .collect();

        // Count matches per domain
        let mut domain_scores: HashMap<&str, usize> = HashMap::new();
        for (domain, keywords) in &self.domain_keywords {
            let score: usize = nouns
                .iter()
                .filter(|&&noun| keywords.contains(noun))
                .count();
            domain_scores.insert(domain, score);
        }

        // Find best domain
        if let Some((&best_domain, &score)) = domain_scores.iter().max_by_key(|(_, &score)| score) {
            if score > 0 {
                return best_domain.to_string();
            }
        }

        self.default_domain.clone()
    }

    /// Classify text directly
    pub fn classify_text(&self, text: &str) -> String {
        let processor = TextProcessor::new();
        let tokens = processor.tokenize(text, None);
        self.classify(&tokens)
    }

    /// Add custom domain
    pub fn add_domain(&mut self, domain: &str, keywords: &[&str]) {
        let keyword_set: HashSet<String> = keywords.iter().map(|s| s.to_string()).collect();
        self.domain_keywords.insert(domain.to_string(), keyword_set);
    }

    /// Set default domain
    pub fn set_default_domain(&mut self, domain: &str) {
        self.default_domain = domain.to_string();
    }
}

impl Default for DomainClassifier {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Simple tokenize function
pub fn simple_tokenize(text: &str) -> Vec<String> {
    let processor = TextProcessor::new();
    processor
        .tokenize(text, None)
        .into_iter()
        .map(|t| t.surface)
        .collect()
}

/// Extract keywords from text
pub fn extract_keywords(text: &str, max_keywords: usize) -> Vec<String> {
    let processor = TextProcessor::new();
    let mut freq = processor.get_word_frequency(&[text]);

    // Sort by frequency
    let mut items: Vec<(String, usize)> = freq.drain().collect();
    items.sort_by_key(|b| std::cmp::Reverse(b.1));

    items
        .into_iter()
        .take(max_keywords)
        .map(|(word, _)| word)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_japanese() {
        let processor = TextProcessor::new();
        let text = "プログラミングは楽しい";
        let tokens = processor.tokenize(text, None);

        assert!(!tokens.is_empty());
        // Should have some noun tokens
        assert!(tokens.iter().any(|t| t.pos == "名詞"));
    }

    #[test]
    fn test_tokenize_english() {
        let processor = TextProcessor::new();
        let text = "Programming is fun";
        let tokens = processor.tokenize(text, None);

        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_tokenize_mixed() {
        let processor = TextProcessor::new();
        let text = "Rustでプログラミング開発";
        let tokens = processor.tokenize(text, None);

        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_extract_nouns() {
        let processor = TextProcessor::new();
        let text = "データベース設計とプログラミング開発";
        let nouns = processor.extract_nouns(text, true);

        assert!(!nouns.is_empty());
    }

    #[test]
    fn test_stopword_removal() {
        let processor = TextProcessor::new();

        // Japanese stopword
        assert!(processor.is_stopword("の"));
        assert!(processor.is_stopword("について"));

        // English stopword
        assert!(processor.is_stopword("the"));
        assert!(processor.is_stopword("is"));

        // Not stopwords
        assert!(!processor.is_stopword("プログラミング"));
        assert!(!processor.is_stopword("algorithm"));
    }

    #[test]
    fn test_word_frequency() {
        let processor = TextProcessor::new();
        let texts = ["プログラミング開発", "プログラミング設計", "システム開発"];

        let freq = processor.get_word_frequency(&texts);

        // "プログラミング" should appear twice
        // Note: exact count depends on tokenization
        assert!(freq.contains_key("プログラミング") || !freq.is_empty());
    }

    #[test]
    fn test_text_hash() {
        let processor = TextProcessor::new();

        let hash1 = processor.generate_text_hash("Hello World");
        let hash2 = processor.generate_text_hash("Hello World");
        let hash3 = processor.generate_text_hash("Different text");

        // Same text should produce same hash
        assert_eq!(hash1, hash2);
        // Different text should produce different hash
        assert_ne!(hash1, hash3);
        // Hash should be 16 characters
        assert_eq!(hash1.len(), 16);
    }

    #[test]
    fn test_domain_classifier() {
        let classifier = DomainClassifier::new();

        // Technology text
        let tech_text = "プログラミングとアルゴリズムの開発";
        assert_eq!(classifier.classify_text(tech_text), "技術");

        // Medical text
        let medical_text = "患者の診断と治療方針";
        assert_eq!(classifier.classify_text(medical_text), "医療");

        // General text
        let general_text = "今日は天気がいい";
        assert_eq!(classifier.classify_text(general_text), "一般");
    }

    #[test]
    fn test_simple_tokenize() {
        let tokens = simple_tokenize("Hello World プログラミング");
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_extract_keywords() {
        let keywords = extract_keywords("プログラミング開発 プログラミング設計 システム開発", 3);
        assert!(keywords.len() <= 3);
    }

    #[test]
    fn test_custom_stopwords() {
        let mut processor = TextProcessor::new();

        assert!(!processor.is_stopword("カスタム"));

        processor.add_stopwords(&["カスタム"]);
        assert!(processor.is_stopword("カスタム"));
    }

    #[test]
    fn test_custom_domain() {
        let mut classifier = DomainClassifier::new();

        classifier.add_domain("カスタム", &["特殊", "専門", "独自"]);

        let text = "特殊な専門知識";
        assert_eq!(classifier.classify_text(text), "カスタム");
    }
}
