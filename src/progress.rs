use indicatif::{ProgressBar, ProgressStyle};

pub fn config() -> ProgressBar {
    let progress_bar = ProgressBar::new(0);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.blue} [{elapsed_precise:.blue}] {msg:.magenta} [{bar:2.blue/cyan}]\n",
            )
            .progress_chars("▸"),
    );
    progress_bar
}
