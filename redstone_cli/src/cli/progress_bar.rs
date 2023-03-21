use indicatif::{ProgressBar, ProgressFinish, ProgressState, ProgressStyle};
use redstone_common::model::ipc::FileActionProgress;
use std::{borrow::Cow, fmt::Write, time::Duration};

pub struct FileTransferProgressBar {
    progress_bar: ProgressBar,
    current_file_name: String,
}

impl FileTransferProgressBar {
    pub fn new(file_action_progress: FileActionProgress) -> Self {
        let total_pb = ProgressBar::new(file_action_progress.total)
            .with_finish(ProgressFinish::WithMessage(Cow::Owned("Done! ✓".into())));
        total_pb.set_style(Self::get_style());
        total_pb.set_prefix(
            file_action_progress
                .operation
                .get_progress_bar_message(&file_action_progress.current_file_name),
        );
        total_pb.enable_steady_tick(Duration::from_millis(100));

        Self {
            progress_bar: total_pb,
            current_file_name: file_action_progress.current_file_name,
        }
    }

    pub fn handle_change(&mut self, progress: FileActionProgress) {
        self.current_file_name = progress.current_file_name;
        if progress.progress >= progress.total {
            self.progress_bar.set_prefix("");
        } else {
            self.progress_bar.set_prefix(
                progress
                    .operation
                    .get_progress_bar_message(&self.current_file_name),
            );
        }
        self.progress_bar.set_position(progress.progress);
    }

    fn get_style() -> ProgressStyle {
        ProgressStyle::with_template(
            "{spinner:.green} {prefix} [{elapsed_precise}] [{wide_bar}] {percent}% ({bytes}/{total_bytes}) {msg:.green} ",
        )
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
            write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
        })
        .progress_chars("=> ")
    }
}
