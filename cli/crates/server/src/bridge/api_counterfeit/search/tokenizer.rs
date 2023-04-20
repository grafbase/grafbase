use std::mem;

use tantivy::tokenizer::{
    AsciiFoldingFilter, BoxTokenStream, LowerCaser, RemoveLongFilter, SimpleTokenizer, TextAnalyzer, Token,
    TokenFilter, TokenStream,
};
use unicode_normalization::UnicodeNormalization;

// MUST NOT be renamed, existing indices will rely on this name
// If a new tokenizer is added you might want to change the schema.digest() to force
// new index creation.
pub(super) const TOKENIZER_NAME: &str = "simple_normalized";

pub(super) fn simple_normalized_tokenizer() -> TextAnalyzer {
    // Tantivy default tokenizer + Unicode normalization ('i⁹' -> 'i9') + AsciiFoldingFilter ('è' -> e)
    TextAnalyzer::from(SimpleTokenizer)
        .filter(RemoveLongFilter::limit(40))
        .filter(LowerCaser)
        // FIXME: Unicode normalization should happen during tokenization as symbols like '¼' would
        // be treated a 1/4 and wouldn't be entirely removed. Unfortunately, it's complex
        // currently to keep track of the original byte offsets while normalizing with current APIs.
        .filter(UnicodeNormalizationFilter)
        .filter(AsciiFoldingFilter)
}

// Basically the same as AsciiFoldingFilter except we just normalize unicode.
#[derive(Clone)]
struct UnicodeNormalizationFilter;

impl TokenFilter for UnicodeNormalizationFilter {
    fn transform<'a>(&self, token_stream: BoxTokenStream<'a>) -> BoxTokenStream<'a> {
        From::from(UnicodeNormalizationTokenStream {
            tail: token_stream,
            buffer: String::with_capacity(100),
        })
    }
}

struct UnicodeNormalizationTokenStream<'a> {
    buffer: String,
    tail: BoxTokenStream<'a>,
}

impl<'a> TokenStream for UnicodeNormalizationTokenStream<'a> {
    fn advance(&mut self) -> bool {
        if !self.tail.advance() {
            return false;
        }
        if !self.token_mut().text.is_ascii() {
            // ignore its already ascii
            normalize(&self.tail.token().text, &mut self.buffer);
            mem::swap(&mut self.tail.token_mut().text, &mut self.buffer);
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
