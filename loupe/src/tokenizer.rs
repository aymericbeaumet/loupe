use unicode_segmentation::UnicodeSegmentation;
use unidecode::unidecode;

pub trait TokenizerExt {
  type Iter: Iterator<Item = String>;

  fn tokenize(&self) -> Self::Iter;
}

impl<'a> TokenizerExt for &'a str {
  // TODO: use `impl ...` instead (when it supports lifetimes)
  type Iter = std::iter::Map<unicode_segmentation::UnicodeWords<'a>, fn(&str) -> String>;

  fn tokenize(&self) -> Self::Iter {
    self
      .unicode_words()
      .map(|word| unidecode(word).to_lowercase())
  }
}
