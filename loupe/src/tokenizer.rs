use unicode_segmentation::UnicodeSegmentation;
use unidecode::unidecode;

pub trait TokenizerExt {
  fn tokenize(&self) -> Vec<String>;
}

impl<'a> TokenizerExt for &'a str {
  fn tokenize(&self) -> Vec<String> {
    self
      .unicode_words()
      .map(|word| unidecode(word).to_lowercase())
      .collect() // TODO: do not collect here
  }
}
