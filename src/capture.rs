use image::DynamicImage;
use serde::Serialize;
use tracing::{debug, info, instrument, warn};
use xcap::{Monitor, Window};

/// Window information returned by list_windows.
#[derive(Debug, Serialize)]
pub struct WindowInfo {
    pub id: u32,
    pub title: String,
    pub app_name: String,
    pub width: u32,
    pub height: u32,
    pub is_focused: bool,
}

/// List all visible windows with non-empty titles.
#[instrument]
pub fn list_windows() -> anyhow::Result<Vec<WindowInfo>> {
    let windows = Window::all().map_err(|e| {
        warn!(error = %e, "Failed to enumerate windows — check Screen Recording permission");
        anyhow::anyhow!(
            "Failed to enumerate windows: {}. \
             On macOS, grant Screen Recording permission in \
             System Settings → Privacy & Security → Screen Recording.",
            e
        )
    })?;

    debug!(total_raw = windows.len(), "Raw windows enumerated");

    let mut infos = Vec::new();
    let mut filtered = 0u32;
    for w in windows {
        let id = w.id().unwrap_or(0);
        let title = match w.title() {
            Ok(t) if !t.is_empty() => t,
            Ok(_) => {
                debug!(id, app = %w.app_name().unwrap_or_default(), "Skipping window: empty title");
                filtered += 1;
                continue;
            }
            Err(e) => {
                debug!(id, error = %e, "Skipping window: title error");
                filtered += 1;
                continue;
            }
        };
        let is_minimized = w.is_minimized().unwrap_or(false);
        if is_minimized {
            debug!(id, %title, "Skipping window: minimized");
            filtered += 1;
            continue;
        }
        let width = w.width().unwrap_or(0);
        let height = w.height().unwrap_or(0);
        if width == 0 || height == 0 {
            debug!(id, %title, "Skipping window: zero size");
            filtered += 1;
            continue;
        }
        infos.push(WindowInfo {
            id,
            title,
            app_name: w.app_name().unwrap_or_default(),
            width,
            height,
            is_focused: w.is_focused().unwrap_or(false),
        });
    }

    info!(count = infos.len(), filtered, "Windows listed");
    if infos.len() <= 1 {
        warn!(
            "Very few windows found ({}) — if windows are open but not listed, \
             check Screen Recording permission in \
             System Settings → Privacy & Security → Screen Recording",
            infos.len()
        );
    }
    Ok(infos)
}

/// Find a window by ID, title (case-insensitive substring), or focused state.
/// Returns the window and its info.
#[instrument]
pub fn find_window(id: Option<u32>, title: Option<&str>) -> anyhow::Result<(Window, WindowInfo)> {
    let windows = Window::all().map_err(|e| {
        anyhow::anyhow!(
            "Failed to enumerate windows: {}. \
             On macOS, grant Screen Recording permission in \
             System Settings → Privacy & Security → Screen Recording.",
            e
        )
    })?;

    debug!(total = windows.len(), "Searching windows");

    // Search by ID (exact match)
    if let Some(target_id) = id {
        info!(target_id, "Finding window by ID");
        for w in &windows {
            if w.id().unwrap_or(0) == target_id {
                let info = window_to_info(w)?;
                return Ok((w.clone(), info));
            }
        }
        anyhow::bail!("No window found with id {target_id}");
    }

    // Search by title (case-insensitive substring)
    if let Some(target_title) = title {
        let lower = target_title.to_lowercase();
        info!(target_title, "Finding window by title");
        for w in &windows {
            let t = w.title().unwrap_or_default();
            if t.to_lowercase().contains(&lower) {
                let info = window_to_info(w)?;
                return Ok((w.clone(), info));
            }
        }
        anyhow::bail!("No window found matching title \"{target_title}\"");
    }

    // Default: find focused window
    info!("Finding focused window");
    for w in &windows {
        if w.is_focused().unwrap_or(false) {
            let info = window_to_info(w)?;
            return Ok((w.clone(), info));
        }
    }

    anyhow::bail!("No focused window found. Specify a title or id.")
}

fn window_to_info(w: &Window) -> anyhow::Result<WindowInfo> {
    Ok(WindowInfo {
        id: w.id().unwrap_or(0),
        title: w.title().unwrap_or_default(),
        app_name: w.app_name().unwrap_or_default(),
        width: w.width().unwrap_or(0),
        height: w.height().unwrap_or(0),
        is_focused: w.is_focused().unwrap_or(false),
    })
}

/// Capture a window screenshot and return it as a DynamicImage.
#[instrument(skip(window))]
pub fn capture_window(window: &Window) -> anyhow::Result<DynamicImage> {
    info!(id = window.id().unwrap_or(0), "Capturing window");
    let buffer = window
        .capture_image()
        .map_err(|e| anyhow::anyhow!("Failed to capture window: {e}"))?;
    let img = DynamicImage::ImageRgba8(buffer);
    debug!(w = img.width(), h = img.height(), "Window captured");
    Ok(img)
}

/// Monitor info for screenshot_screen results.
#[derive(Debug, Serialize)]
pub struct MonitorInfo {
    pub id: u32,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub is_primary: bool,
}

/// Capture a screen/monitor screenshot.
#[instrument]
pub fn capture_screen(monitor_id: Option<u32>) -> anyhow::Result<(DynamicImage, MonitorInfo)> {
    let monitors =
        Monitor::all().map_err(|e| anyhow::anyhow!("Failed to enumerate monitors: {e}"))?;

    debug!(count = monitors.len(), "Monitors enumerated");

    let monitor = if let Some(target_id) = monitor_id {
        info!(target_id, "Finding monitor by ID");
        monitors
            .into_iter()
            .find(|m| m.id().unwrap_or(0) == target_id)
            .ok_or_else(|| anyhow::anyhow!("No monitor found with id {target_id}"))?
    } else {
        info!("Using primary monitor");
        monitors
            .into_iter()
            .find(|m| m.is_primary().unwrap_or(false))
            .ok_or_else(|| anyhow::anyhow!("No primary monitor found"))?
    };

    let info = MonitorInfo {
        id: monitor.id().unwrap_or(0),
        name: monitor.name().unwrap_or_default(),
        width: monitor.width().unwrap_or(0),
        height: monitor.height().unwrap_or(0),
        is_primary: monitor.is_primary().unwrap_or(false),
    };

    let buffer = monitor
        .capture_image()
        .map_err(|e| anyhow::anyhow!("Failed to capture screen: {e}"))?;
    let img = DynamicImage::ImageRgba8(buffer);
    info!(w = img.width(), h = img.height(), monitor_name = %info.name, "Screen captured");
    Ok((img, info))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires Screen Recording permission and a display"]
    fn test_list_windows() {
        let windows = list_windows().unwrap();
        assert!(!windows.is_empty(), "Should find at least one window");
        for w in &windows {
            assert!(!w.title.is_empty());
            assert!(w.width > 0);
            assert!(w.height > 0);
        }
    }

    #[test]
    #[ignore = "requires Screen Recording permission and a display"]
    fn test_find_focused_window() {
        let (window, info) = find_window(None, None).unwrap();
        assert!(!info.title.is_empty());
        let img = capture_window(&window).unwrap();
        assert!(img.width() > 0);
        assert!(img.height() > 0);
    }

    #[test]
    #[ignore = "requires Screen Recording permission and a display"]
    fn test_capture_primary_screen() {
        let (img, info) = capture_screen(None).unwrap();
        assert!(img.width() > 0);
        assert!(img.height() > 0);
        assert!(info.is_primary);
    }
}
