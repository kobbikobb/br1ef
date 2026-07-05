use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

const SPINNERS: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub fn with_progress<T>(prompt: &str, f: impl FnOnce() -> T) -> T {
    let done = Arc::new(AtomicBool::new(false));
    let bar_done = Arc::clone(&done);
    let prompt = prompt.to_string();

    let bar = thread::spawn(move || render_spinner(&prompt, bar_done));

    let result = f();

    done.store(true, Ordering::SeqCst);
    bar.join().expect("progress bar thread panicked");

    eprint!("\r\x1b[2K\r");
    let _ = std::io::Write::write(&mut std::io::stderr(), b"");

    result
}

fn render_spinner(prompt: &str, done: Arc<AtomicBool>) {
    let mut i = 0;
    loop {
        if done.load(Ordering::SeqCst) {
            eprint!("\r  {prompt}  ✓");
            break;
        }

        let s = SPINNERS[i % SPINNERS.len()];
        eprint!("\r  {s} {prompt}");
        i += 1;

        thread::sleep(Duration::from_millis(100));
    }
}

#[cfg(test)]
#[path = "progress_test.rs"]
mod tests;
