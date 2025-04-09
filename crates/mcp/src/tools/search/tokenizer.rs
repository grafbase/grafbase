use std::mem;

use tantivy::tokenizer::{
    AsciiFoldingFilter, LowerCaser, RemoveLongFilter, SimpleTokenizer, TextAnalyzer, Token, TokenFilter, TokenStream,
    Tokenizer,
};
use unicode_normalization::UnicodeNormalization as _;

// MUST NOT be renamed, existing indices will rely on this name
// If a new tokenizer is added you might want to change the schema.digest() to force
// new index creation.
pub(super) const TOKENIZER_NAME: &str = "simple_normalized";

pub(super) fn analyzer() -> TextAnalyzer {
    // Tantivy default tokenizer + Unicode normalization ('i⁹' -> 'i9') + AsciiFoldingFilter ('è' -> e)
    TextAnalyzer::builder(SimpleTokenizer::default())
        .filter(RemoveLongFilter::limit(40))
        .filter(LowerCaser)
        // FIXME: Unicode normalization should happen during tokenization as symbols like '¼' would
        // be treated a 1/4 and wouldn't be entirely removed. Unfortunately, it's complex
        // currently to keep track of the original byte offsets while normalizing with current APIs.
        .filter(UnicodeNormalization)
        .filter(AsciiFoldingFilter)
        .build()
}

#[derive(Clone)]
pub struct UnicodeNormalization;

impl TokenFilter for UnicodeNormalization {
    type Tokenizer<T: Tokenizer> = UnicodeNormalizationFilter<T>;

    fn transform<T: Tokenizer>(self, tokenizer: T) -> Self::Tokenizer<T> {
        UnicodeNormalizationFilter {
            tokenizer,
            buffer: String::new(),
        }
    }
}

// Basically the same as AsciiFoldingFilter except we just normalize unicode.
#[derive(Clone)]
pub struct UnicodeNormalizationFilter<T> {
    tokenizer: T,
    buffer: String,
}

impl<T: Tokenizer> Tokenizer for UnicodeNormalizationFilter<T> {
    type TokenStream<'a> = UnicodeNormalizationTokenStream<'a, T::TokenStream<'a>>;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        UnicodeNormalizationTokenStream {
            tail: self.tokenizer.token_stream(text),
            buffer: &mut self.buffer,
        }
    }
}

pub struct UnicodeNormalizationTokenStream<'a, T> {
    buffer: &'a mut String,
    tail: T,
}

impl<T: TokenStream> TokenStream for UnicodeNormalizationTokenStream<'_, T> {
    fn advance(&mut self) -> bool {
        if !self.tail.advance() {
            return false;
        }
        if !self.token_mut().text.is_ascii() {
            // ignore its already ascii
            normalize(&self.tail.token().text, self.buffer);
            mem::swap(&mut self.tail.token_mut().text, self.buffer);
        }
        true
    }

    fn token(&self) -> &Token {
        self.tail.token()
    }

    fn token_mut(&mut self) -> &mut Token {
        self.tail.token_mut()
    }
}

fn normalize(text: &str, output: &mut String) {
    output.clear();

    // We want compatibility equivalence, it only matters that characters have similar meaning.
    // See: https://www.unicode.org/reports/tr15/
    output.extend(text.nfkc());
}
