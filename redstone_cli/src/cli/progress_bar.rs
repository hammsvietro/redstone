use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use std::fmt::Write;

pub fn download_progress_bar(total_download_size: u64) -> ProgressBar {
    let pb = ProgressBar::new(total_download_size);
    pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-"));
    pb
}
