use colored::Colorize;
use std::io::{self, Write};
use std::time::{Duration, Instant};

use crate::error::Result;
use crate::frame::BoxFrame;
use crate::terminal::get_content_width;
use crate::theme::is_utf8_supported;

use crate::modals::waiting::SPINNER_FRAMES;
const FRACTIONAL_BLOCKS: &[&str] = &[" ", "▏", "▎", "▍", "▌", "▋", "▊", "▉", "█"];

/// Visual styling options for the progress bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressStyle {
    /// High-precision fractional Unicode blocks: `▏▎▍▌▋▊▉█`.
    UnicodeSmooth,
    /// Solid Unicode blocks: `█` and `░`.
    UnicodeBlock,
    /// Standard ASCII characters: `=` and `-` with `>`.
    Ascii,
}

impl Default for ProgressStyle {
    fn default() -> Self {
        if is_utf8_supported() {
            Self::UnicodeSmooth
        } else {
            Self::Ascii
        }
    }
}

/// A flexible progress bar tracking bytes or items, calculating speeds, and estimating ETAs.
#[derive(Debug, Clone)]
pub struct ProgressBar {
    pub current: u64,
    pub total: u64,
    pub style: ProgressStyle,
    pub start_time: Instant,
    pub last_update_time: Instant,
    pub last_bytes: u64,
    pub smoothed_speed: f64,
}

impl ProgressBar {
    /// Creates a new `ProgressBar` with a known total count (e.g. total bytes).
    pub fn new(total: u64) -> Self {
        let now = Instant::now();
        Self {
            current: 0,
            total,
            style: ProgressStyle::default(),
            start_time: now,
            last_update_time: now,
            last_bytes: 0,
            smoothed_speed: 0.0,
        }
    }

    /// Creates an indeterminate progress bar (total size unknown).
    pub fn indeterminate() -> Self {
        Self::new(0)
    }

    /// Sets the current progress position.
    pub fn set_position(&mut self, pos: u64) {
        let now = Instant::now();
        let elapsed_since_last = now.duration_since(self.last_update_time).as_secs_f64();

        if elapsed_since_last >= 0.15 {
            let bytes_delta = pos.saturating_sub(self.last_bytes) as f64;
            let current_speed = bytes_delta / elapsed_since_last;

            if self.smoothed_speed == 0.0 {
                self.smoothed_speed = current_speed;
            } else {
                // Exponential moving average for smooth speed display
                self.smoothed_speed = 0.7 * self.smoothed_speed + 0.3 * current_speed;
            }

            self.last_update_time = now;
            self.last_bytes = pos;
        }

        self.current = pos;
    }

    /// Increments the current position by a delta.
    pub fn inc(&mut self, delta: u64) {
        self.set_position(self.current.saturating_add(delta));
    }

    /// Returns the progress fraction from 0.0 to 1.0.
    pub fn fraction(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.current as f64 / self.total as f64).clamp(0.0, 1.0)
        }
    }

    /// Returns progress percentage from 0.0 to 100.0.
    pub fn percent(&self) -> f64 {
        self.fraction() * 100.0
    }

    /// Returns true if the total size is unknown (indeterminate).
    pub fn is_indeterminate(&self) -> bool {
        self.total == 0
    }

    /// Estimated transfer rate in bytes per second.
    pub fn bytes_per_second(&self) -> f64 {
        if self.smoothed_speed > 0.0 {
            self.smoothed_speed
        } else {
            let elapsed = self.start_time.elapsed().as_secs_f64();
            if elapsed > 0.1 {
                self.current as f64 / elapsed
            } else {
                0.0
            }
        }
    }

    /// Formats a raw byte count into human-readable string (B, KB, MB, GB).
    pub fn format_bytes(bytes: u64) -> String {
        const KB: f64 = 1024.0;
        const MB: f64 = 1024.0 * KB;
        const GB: f64 = 1024.0 * MB;

        let b = bytes as f64;
        if b >= GB {
            format!("{:.1} GB", b / GB)
        } else if b >= MB {
            format!("{:.1} MB", b / MB)
        } else if b >= KB {
            format!("{:.1} KB", b / KB)
        } else {
            format!("{} B", bytes)
        }
    }

    /// Formats current speed as human-readable string (e.g. `4.2 MB/s`).
    pub fn format_speed(&self) -> String {
        let bps = self.bytes_per_second();
        if bps <= 0.0 {
            "-- B/s".to_string()
        } else {
            format!("{}/s", Self::format_bytes(bps as u64))
        }
    }

    /// Estimates time remaining based on current speed.
    pub fn eta(&self) -> Option<Duration> {
        if self.is_indeterminate() || self.current >= self.total {
            return None;
        }

        let bps = self.bytes_per_second();
        if bps > 10.0 {
            let remaining_bytes = self.total - self.current;
            let secs = (remaining_bytes as f64 / bps).clamp(0.0, 86400.0);
            Some(Duration::from_secs_f64(secs))
        } else {
            None
        }
    }

    /// Formats ETA into human-readable string (e.g. `ETA: 12s`, `ETA: 1m 30s`).
    pub fn format_eta(&self) -> String {
        match self.eta() {
            Some(d) => {
                let secs = d.as_secs();
                if secs < 60 {
                    format!("ETA: {}s", secs)
                } else {
                    let mins = secs / 60;
                    let rem_secs = secs % 60;
                    format!("ETA: {}m {}s", mins, rem_secs)
                }
            }
            None => {
                if self.is_indeterminate() {
                    "--".to_string()
                } else if self.current >= self.total {
                    "Complete".to_string()
                } else {
                    "ETA: --".to_string()
                }
            }
        }
    }

    /// Formats `current / total` counts (e.g. `18.4 MB / 35.1 MB`).
    pub fn format_counts(&self) -> String {
        if self.is_indeterminate() {
            Self::format_bytes(self.current)
        } else {
            format!(
                "{} / {}",
                Self::format_bytes(self.current),
                Self::format_bytes(self.total)
            )
        }
    }

    /// Renders the visual bar component into an ANSI-formatted string of given character width.
    pub fn render_bar(&self, width: usize, spinner_frame: usize) -> String {
        let width = width.max(10);

        if self.is_indeterminate() {
            // Indeterminate animated bounce bar
            let block_size = (width / 4).max(3);
            let travel = width.saturating_sub(block_size);
            let pos = if travel > 0 {
                let cycle = spinner_frame % (travel * 2);
                if cycle < travel {
                    cycle
                } else {
                    travel * 2 - cycle
                }
            } else {
                0
            };

            let left = "░".repeat(pos);
            let middle = "█".repeat(block_size);
            let right = "░".repeat(width.saturating_sub(pos + block_size));

            format!(
                "[{}{}{}]",
                left.dimmed(),
                middle.cyan().bold(),
                right.dimmed()
            )
        } else {
            let frac = self.fraction();

            match self.style {
                ProgressStyle::UnicodeSmooth => {
                    let total_subunits = (width as f64 * frac * 8.0).round() as usize;
                    let full_blocks = (total_subunits / 8).min(width);
                    let remainder = total_subunits % 8;

                    let mut bar = String::with_capacity(width * 4);
                    for _ in 0..full_blocks {
                        bar.push('█');
                    }
                    if full_blocks < width && remainder > 0 {
                        bar.push_str(FRACTIONAL_BLOCKS[remainder]);
                    }
                    let unfilled_len = width.saturating_sub(full_blocks + if remainder > 0 { 1 } else { 0 });
                    let unfilled = "░".repeat(unfilled_len);

                    format!(
                        "[{}{}]",
                        bar.cyan().bold(),
                        unfilled.dimmed()
                    )
                }
                ProgressStyle::UnicodeBlock => {
                    let filled_len = ((width as f64 * frac).round() as usize).min(width);
                    let unfilled_len = width.saturating_sub(filled_len);

                    let filled = "█".repeat(filled_len);
                    let unfilled = "░".repeat(unfilled_len);

                    format!(
                        "[{}{}]",
                        filled.cyan().bold(),
                        unfilled.dimmed()
                    )
                }
                ProgressStyle::Ascii => {
                    let filled_len = ((width as f64 * frac).round() as usize).min(width);
                    let (arrow, unfilled_len) = if filled_len > 0 && filled_len < width {
                        (">", width.saturating_sub(filled_len + 1))
                    } else {
                        ("", width.saturating_sub(filled_len))
                    };

                    let body_len = filled_len.saturating_sub(if arrow.is_empty() { 0 } else { 1 });
                    let filled = "=".repeat(body_len);
                    let unfilled = "-".repeat(unfilled_len);

                    format!(
                        "[{}{}{}]",
                        filled.cyan().bold(),
                        arrow.cyan().bold(),
                        unfilled.dimmed()
                    )
                }
            }
        }
    }
}

/// A dedicated, airtight terminal progress dialog modal.
#[derive(Debug, Clone)]
pub struct ProgressModal {
    pub title: String,
    pub message: String,
    pub progress: ProgressBar,
    pub steps: Vec<(String, bool)>,
    pub max_width: u16,
    pub shortcuts: Option<crate::shortcuts::Shortcuts>,
    pub last_render: Option<Instant>,
    pub spinner_idx: usize,
    pub render_throttle_ms: u64,
}

impl ProgressModal {
    /// Creates a new `ProgressModal` with a known total (e.g. bytes to download).
    pub fn new(title: impl Into<String>, message: impl Into<String>, total: u64) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            progress: ProgressBar::new(total),
            steps: Vec::new(),
            max_width: 76,
            shortcuts: None,
            last_render: None,
            spinner_idx: 0,
            render_throttle_ms: 35,
        }
    }

    /// Creates an indeterminate `ProgressModal` (unknown total size).
    pub fn indeterminate(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(title, message, 0)
    }

    /// Adds a step to the checklist (e.g. `✓ Query manifest`, `• Downloading jar`).
    pub fn with_step(mut self, step: impl Into<String>, completed: bool) -> Self {
        self.steps.push((step.into(), completed));
        self
    }

    /// Sets the maximum box width (defaults to 76).
    pub fn with_max_width(mut self, width: u16) -> Self {
        self.max_width = width;
        self
    }

    /// Sets custom shortcuts for the footer bar.
    pub fn with_shortcuts(mut self, shortcuts: impl Into<crate::shortcuts::Shortcuts>) -> Self {
        self.shortcuts = Some(shortcuts.into());
        self
    }

    /// Sets the total count or size (e.g. total bytes).
    pub fn with_total(mut self, total: u64) -> Self {
        self.progress.total = total;
        self
    }

    /// Sets the progress bar visual style.
    pub fn with_style(mut self, style: ProgressStyle) -> Self {
        self.progress.style = style;
        self
    }

    /// Sets render throttling limit in milliseconds (default: 35ms).
    pub fn with_throttle_ms(mut self, ms: u64) -> Self {
        self.render_throttle_ms = ms;
        self
    }

    /// Updates the completion status of a step by index.
    pub fn set_step_status(&mut self, idx: usize, completed: bool) {
        if let Some(step) = self.steps.get_mut(idx) {
            step.1 = completed;
        }
    }

    /// Updates the main status message.
    pub fn set_message(&mut self, msg: impl Into<String>) {
        self.message = msg.into();
    }

    /// Updates current progress position.
    pub fn update(&mut self, current: u64) {
        self.progress.set_position(current);
    }

    /// Increments current progress position by delta.
    pub fn inc(&mut self, delta: u64) {
        self.progress.inc(delta);
    }

    /// Builds a formatted BoxFrame containing all progress visual components.
    pub fn build_frame(&self) -> BoxFrame {
        let width = get_content_width(self.max_width);
        let inner_width = width.saturating_sub(4);

        let mut frame = BoxFrame::new(width);
        frame.title = Some((self.title.clone(), false));

        // 1. Message Line with Animated Spinner
        let spinner_char = SPINNER_FRAMES[self.spinner_idx % SPINNER_FRAMES.len()];
        let main_line = format!(
            "{} {}",
            spinner_char.cyan().bold(),
            self.message.white().bold()
        );
        frame.row(main_line);
        frame.empty_row();

        // 2. Progress Bar Row
        // Calculate appropriate bar width leaving room for percent label
        let bar_width = inner_width.saturating_sub(10).clamp(20, 48);
        let bar_visual = self.progress.render_bar(bar_width, self.spinner_idx);

        let bar_line = if self.progress.is_indeterminate() {
            format!("  {}", bar_visual)
        } else {
            format!("  {} {:>5.1}%", bar_visual, self.progress.percent())
        };
        frame.row(bar_line);

        // 3. Telemetry Row (Counts, Speed, ETA)
        let telemetry = if self.progress.is_indeterminate() {
            format!(
                "  {} downloaded  •  {}",
                self.progress.format_counts().cyan(),
                self.progress.format_speed().dimmed()
            )
        } else {
            format!(
                "  {}  •  {}  •  {}",
                self.progress.format_counts().cyan(),
                self.progress.format_speed().dimmed(),
                self.progress.format_eta().yellow()
            )
        };
        frame.row(telemetry);

        // 4. Checklist Steps
        if !self.steps.is_empty() {
            frame.empty_row();
            for (step, completed) in &self.steps {
                if *completed {
                    frame.row(format!("  {} {}", "✓".green().bold(), step));
                } else {
                    frame.row(format!("  {} {}", "•".dimmed(), step.dimmed()));
                }
            }
        }

        // 5. Footer Shortcuts / Status
        if let Some(ref sc) = self.shortcuts {
            frame.shortcuts(sc);
        } else {
            frame.footer("Please wait...".dimmed().to_string());
        }

        frame
    }

    /// Renders the progress modal if sufficient time has elapsed since last frame.
    pub fn render(&mut self, stdout: &mut io::Stdout) -> Result<()> {
        let now = Instant::now();
        if let Some(last) = self.last_render {
            if now.duration_since(last) < Duration::from_millis(self.render_throttle_ms) {
                return Ok(());
            }
        }

        self.spinner_idx = self.spinner_idx.wrapping_add(1);
        self.last_render = Some(now);

        let frame = self.build_frame();
        frame.render(stdout)
    }

    /// Forces a render immediately, bypassing throttle.
    pub fn render_forced(&mut self, stdout: &mut io::Stdout) -> Result<()> {
        self.last_render = Some(Instant::now());
        self.spinner_idx = self.spinner_idx.wrapping_add(1);
        let frame = self.build_frame();
        frame.render(stdout)
    }

    /// Marks progress as completed, renders the final state, and flushes stdout.
    pub fn finish(&mut self, final_message: impl Into<String>, stdout: &mut io::Stdout) -> Result<()> {
        self.progress.current = self.progress.total;
        self.message = final_message.into();
        self.render_forced(stdout)?;
        stdout.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar_percent_and_fraction() {
        let mut pb = ProgressBar::new(1000);
        assert_eq!(pb.fraction(), 0.0);
        assert_eq!(pb.percent(), 0.0);

        pb.set_position(500);
        assert_eq!(pb.fraction(), 0.5);
        assert_eq!(pb.percent(), 50.0);

        pb.inc(500);
        assert_eq!(pb.fraction(), 1.0);
        assert_eq!(pb.percent(), 100.0);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(ProgressBar::format_bytes(512), "512 B");
        assert_eq!(ProgressBar::format_bytes(1024), "1.0 KB");
        assert_eq!(ProgressBar::format_bytes(10 * 1024 * 1024), "10.0 MB");
        assert_eq!(ProgressBar::format_bytes(2 * 1024 * 1024 * 1024), "2.0 GB");
    }

    #[test]
    fn test_progress_modal_builder() {
        let modal = ProgressModal::new("DOWNLOADING ASSETS", "Fetching server jar...", 50_000_000)
            .with_step("Verify manifest", true)
            .with_step("Download binary", false)
            .with_max_width(80);

        assert_eq!(modal.title, "DOWNLOADING ASSETS");
        assert_eq!(modal.progress.total, 50_000_000);
        assert_eq!(modal.steps.len(), 2);
        assert!(modal.steps[0].1);
        assert!(!modal.steps[1].1);
    }

    #[test]
    fn test_progress_bar_rendering_not_empty() {
        let pb = ProgressBar::new(100);
        let bar_smooth = pb.render_bar(30, 0);
        assert!(!bar_smooth.is_empty());

        let mut pb_ascii = pb.clone();
        pb_ascii.style = ProgressStyle::Ascii;
        let bar_ascii = pb_ascii.render_bar(30, 0);
        assert!(!bar_ascii.is_empty());
        assert!(bar_ascii.contains('[') && bar_ascii.contains(']'));
    }
}
