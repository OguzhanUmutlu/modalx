use modalx::prelude::*;
use std::thread;
use std::time::Duration;

fn main() -> modalx::Result<()> {
    let _alt = AltScreenGuard::enter();
    let mut stdout = std::io::stdout();

    let mut modal = WaitingModal::new("SYSTEM UPGRADE", "Applying latest patches...")
        .with_step("Verify digital signature", true)
        .with_step("Back up existing configuration", true)
        .with_step("Download binary assets", false)
        .with_step("Run database migrations", false);

    // Simulate animated step execution
    for i in 0..30 {
        modal.render_spinner(i, &mut stdout)?;
        thread::sleep(Duration::from_millis(80));
    }

    modal.steps[2].1 = true;

    for i in 30..60 {
        modal.render_spinner(i, &mut stdout)?;
        thread::sleep(Duration::from_millis(80));
    }

    modal.steps[3].1 = true;
    modal.message = "Upgrade completed successfully!".to_string();
    modal.render_static(&mut stdout)?;
    thread::sleep(Duration::from_secs(1));

    Ok(())
}
