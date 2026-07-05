use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

const BAR_WIDTH: usize = 30;

pub fn with_progress<T>(prompt: &str, f: impl FnOnce() -> T) -> T {
    let estimated = estimate_duration(prompt);
    let done = Arc::new(AtomicBool::new(false));
    let bar_done = Arc::clone(&done);

    let bar = thread::spawn(move || render_progress(estimated, bar_done));

    let result = f();

    done.store(true, Ordering::SeqCst);
    bar.join().expect("progress bar thread panicked");

    eprint!("\r\x1b[2K\r");
    let _ = std::io::Write::write(&mut std::io::stderr(), b"");

    result
}

fn estimate_duration(prompt: &str) -> Duration {
    let ms = (prompt.len() as f64 / 100.0 * 50.0).clamp(200.0, 5_000.0);
    Duration::from_millis(ms as u64)
}

fn render_progress(total: Duration, done: Arc<AtomicBool>) {
    let start = std::time::Instant::now();

    loop {
        if done.load(Ordering::SeqCst) {
            let bar = "#".repeat(BAR_WIDTH);
            eprint!("\r  [{bar}] 100%");
            break;
        }

        let elapsed = start.elapsed();
        let fraction = (elapsed.as_secs_f64() / total.as_secs_f64()).min(1.0);
        let filled = (fraction * BAR_WIDTH as f64).round() as usize;
        let pct = (fraction * 100.0).round() as usize;

        let bar: String = std::iter::repeat_n('#', filled)
            .chain(std::iter::repeat_n('-', BAR_WIDTH - filled))
            .collect();
        eprint!("\r  [{bar}] {pct:3}%");

        thread::sleep(if elapsed >= total {
            Duration::from_millis(200)
        } else {
            Duration::from_millis(100)
        });
    }
}

#[cfg(test)]
#[path = "progress_test.rs"]
mod tests;
