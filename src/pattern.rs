use std::fmt::{self, Display};
use std::fs::read_to_string;

use transpose::transpose;

use crate::{Args, PATTERN_HEIGHT};

#[derive(Debug)]
pub struct Pattern {
  template: String,
  width: usize,
  mask: Vec<bool>,
}

impl Pattern {
  pub fn new(args: &Args) -> Pattern {
    let template = match (args.template.as_ref(), args.template_file.as_ref()) {
      (Some(s), Some(_)) | (Some(s), None) => String::from(s),
      (None, Some(p)) => read_to_string(p).expect(&format!("Failed to read file: {}", p.display())),
      (None, None) => panic!("You must either pass a template or a path to a template file!"),
    };

    let template = template.trim().to_owned();

    // Remove non-mask characters and map the template into bools.
    let template_mask: Vec<bool> = template
      .chars()
      .filter_map(|c| match c {
        'X' => Some(true),
        ' ' => Some(false),
        _ => None,
      })
      .collect();

    // Transpose so rows become columns and vice versa (the contributions grid
    // flows top -> down, left -> right).
    let mut mask = vec![false; template_mask.len()];
    transpose(
      &template_mask,
      &mut mask,
      template_mask.len() / PATTERN_HEIGHT,
      PATTERN_HEIGHT,
    );

    if mask.len() % PATTERN_HEIGHT != 0 {
      panic!(
        "Template length {} must be divisible by {}",
        mask.len(),
        PATTERN_HEIGHT
      );
    }

    let template_width = mask.len() / PATTERN_HEIGHT;
    let width = args.template_repeat * template_width;
    let mask = mask
      .iter()
      .cycle()
      .take(width * PATTERN_HEIGHT)
      .map(|r| *r)
      .collect();

    Pattern {
      template,
      width,
      mask,
    }
  }

  /// Returns the original template for the pattern.
  pub fn template(&self) -> String {
    self.template.clone()
  }

  /// Returns the width of the pattern.
  pub fn width(&self) -> usize {
    self.width
  }

  /// Returns a mask for the pattern.
  pub fn mask(&self) -> Vec<bool> {
    self.mask.clone()
  }

  /// Returns the mask as a string pattern
  pub fn mask_as_pattern(&self) -> String {
    let mut transposed = vec![false; self.mask.len()];
    transpose(&self.mask, &mut transposed, PATTERN_HEIGHT, self.width);

    let mut lines = vec![];
    for chunk in transposed.chunks_exact(self.width) {
      let line = chunk
        .iter()
        .map(|b| if *b { "X" } else { " " })
        .collect::<Vec<&str>>()
        .join("");
      lines.push(line);
    }

    lines.join("\n")
  }
}

impl Display for Pattern {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.mask_as_pattern())
  }
}
