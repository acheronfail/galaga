use std::fs::read_to_string;

use transpose::transpose;

use crate::{Args, PATTERN_HEIGHT};

pub fn prepare_mask(args: &Args, num_days: usize) -> Result<Vec<bool>, String> {
  let pattern = match (args.pattern.as_ref(), args.pattern_file.as_ref()) {
    (Some(s), Some(_)) | (Some(s), None) => String::from(s),
    (None, Some(p)) => read_to_string(p).expect(&format!("Failed to read file: {}", p.display())),
    (None, None) => {
      return Err(format!(
        "You must either pass a pattern or a path to a pattern file!"
      ))
    }
  };

  // Map the pattern into bools.
  let mask_pre: Vec<bool> = pattern
    .trim()
    .chars()
    .filter_map(|c| match c {
      'X' => Some(true),
      ' ' => Some(false),
      _ => None,
    })
    .collect();

  // Transpose the mask to fit into the contributions grid.
  let mut mask = vec![false; mask_pre.len()];
  transpose(
    &mask_pre,
    &mut mask,
    mask_pre.len() / PATTERN_HEIGHT,
    PATTERN_HEIGHT,
  );

  // Ensure the mask will fit in the grid.
  if mask.len() % PATTERN_HEIGHT != 0 {
    return Err(format!(
      "Pattern length ({}) must be divisible by {}",
      mask.len(),
      PATTERN_HEIGHT
    ));
  }

  // Extend the pattern to be as long as the number of days.
  mask = mask.iter().cycle().take(num_days).map(|r| *r).collect();

  Ok(mask)
}
