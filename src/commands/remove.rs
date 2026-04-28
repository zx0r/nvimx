use anyhow::Result;
use crate::core::{xdg, output};

pub fn execute(profile: String) -> Result<()> {
    output::header(&format!("Removing: {}", profile));
    
    let mut paths = vec![
        ("Config", xdg::profile_dir(&profile)),
        ("Data", xdg::data_root().join(&profile)),
        ("State", xdg::state_root().join(&profile)),
        ("Cache", xdg::cache_root().join(&profile)),
    ];

    let mut removed_any = false;
    for (label, path) in paths {
        if path.exists() {
            if let Err(e) = std::fs::remove_dir_all(&path) {
                output::bad(&format!("{}: {}", label, e));
            } else {
                output::success(&format!("Deleted {}", label));
                removed_any = true;
            }
        }
    }
    
    if removed_any {
        output::success(&format!("Profile '{}' completely removed", profile));
    } else {
        output::error("Profile not found");
    }
    
    println!();
    Ok(())
}
