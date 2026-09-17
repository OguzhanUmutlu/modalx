use modalx::prelude::*;
use std::io;
use std::thread;
use std::time::Duration;

fn main() -> modalx::Result<()> {
    let _alt = AltScreenGuard::enter();
    let mut stdout = io::stdout();
    let total_bytes: u64 = 64 * 1024 * 1024; // 64 MB
    let chunk_size: u64 = 1024 * 1024; // 1 MB

    let mut modal = ProgressModal::new(
        "ARTIFACT DOWNLOAD",
        "Downloading runtime distribution package...",
        total_bytes,
    )
    .with_step("Verify repository signature", true)
    .with_step("Fetch server binary archive", false)
    .with_step("Validate SHA-256 checksum", false);

    modal.render_forced(&mut stdout)?;

    let mut downloaded = 0;
    while downloaded < total_bytes {
        downloaded = (downloaded + chunk_size).min(total_bytes);
        modal.update(downloaded);
        modal.render(&mut stdout)?;
        thread::sleep(Duration::from_millis(40));
    }

    modal.set_step_status(1, true);
    modal.set_step_status(2, true);
    modal.finish("Download verified successfully!", &mut stdout)?;
    thread::sleep(Duration::from_millis(800));

    Ok(())
}
