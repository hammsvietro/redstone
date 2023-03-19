use indicatif::{MultiProgress, ProgressBar, ProgressFinish, ProgressState, ProgressStyle};
use redstone_common::model::ipc::FileActionProgress;
use std::{fmt::Write, time::Duration};

pub struct FileTransferProgressBar {
    total_progress_bar: ProgressBar,
    file_progress_bar: ProgressBar,
    current_file_name: String,
}

impl FileTransferProgressBar {
    pub fn new(file_action_progress: FileActionProgress) -> Self {
        let m = MultiProgress::new();
        let style = ProgressStyle::with_template(
            "  {spinner:.green} [{elapsed_precise}] [{bar:40.green}] {bytes}/{total_bytes} ({eta}) {msg}  ",
        )
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
            write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
        })
        .progress_chars("#>-");

        let total_pb = m
            .add(ProgressBar::new(file_action_progress.total))
            .with_finish(ProgressFinish::AndLeave);
        total_pb.set_style(style.clone());
        total_pb.set_message(file_action_progress.operation.get_progress_bar_message());
        total_pb.enable_steady_tick(Duration::from_millis(200));

        let file_pb = m
            .add(ProgressBar::new(file_action_progress.file_progress))
            .with_finish(ProgressFinish::AndLeave);

        file_pb.set_style(style);
        file_pb.set_position(file_action_progress.file_progress);
        file_pb.set_message(file_action_progress.current_file_name.to_owned());
        file_pb.enable_steady_tick(Duration::from_millis(200));

        Self {
            total_progress_bar: total_pb,
            file_progress_bar: file_pb,
            current_file_name: file_action_progress.current_file_name,
        }
    }

    pub fn handle_change(&mut self, progress: FileActionProgress) {
        if progress.current_file_name != self.current_file_name
            && Some(self.file_progress_bar.position()) == self.file_progress_bar.length()
        {
            self.current_file_name = progress.current_file_name;
            self.file_progress_bar
                .set_message(self.current_file_name.to_owned());
            self.file_progress_bar.set_length(progress.file_total);
            self.file_progress_bar.set_position(progress.file_progress);
        } else {
            self.file_progress_bar.set_position(progress.file_progress)
        }

        self.total_progress_bar
            .set_position(progress.total_progress);

        if [&self.total_progress_bar, &self.file_progress_bar]
            .iter()
            .all(|i| i.is_finished())
        {
            self.finish();
        }
    }

    fn finish(&mut self) {
        println!("finishing");
        self.total_progress_bar.finish();
        self.file_progress_bar.finish();
        println!("finished");
    }
}
