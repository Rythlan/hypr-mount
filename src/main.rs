use clap::Parser;
use hypr_mount::app::{CliArgs, MountApp};
use hypr_mount::core::error::HyprMountError;
use hypr_mount::core::{drive_handle, mount};

fn main() -> Result<(), HyprMountError> {
    let args = CliArgs::parse();
    let drives = drive_handle::list_drives()?;

    if args.auto_mount {
        return mount::auto_mount();
    }

    if args.generate_service {
        return mount::automount_drives_service();
    }

    let mut term = ratatui::init();

    let mut app = MountApp::new(drives, args);

    let app_res = app.run(&mut term);

    ratatui::restore();
    app_res?;
    Ok(())
}
