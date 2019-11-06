use unicode_segmentation::UnicodeSegmentation;
use unidecode::unidecode;

pub fn normalize(string: &str) -> impl Iterator<Item = String> + '_ {
  string
    .unicode_words()
    .map(|word| unidecode(word).to_lowercase())
}
