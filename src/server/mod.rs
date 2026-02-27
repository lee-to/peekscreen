use rmcp::{
    ErrorData as McpError, ServerHandler, handler::server::router::tool::ToolRouter,
    handler::server::wrapper::Parameters, model::*, schemars, tool, tool_handler, tool_router,
};
use serde::Deserialize;
use tracing::{error, info};

use crate::capture;
use crate::imaging::{self, ImageFormat};

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ScreenshotWindowParams {
    /// Window title substring to search for (case-insensitive). If omitted and no id given, captures the focused window.
    pub title: Option<String>,
    /// Exact window ID from list_windows.
    pub id: Option<u32>,
    /// Maximum image width in pixels (default: 1920). Image is proportionally downscaled if wider.
    pub max_width: Option<u32>,
    /// Output format: "png" (default) or "jpeg".
    pub format: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ScreenshotScreenParams {
    /// Monitor ID to capture. If omitted, captures the primary monitor.
    pub monitor_id: Option<u32>,
    /// Maximum image width in pixels (default: 1920). Image is proportionally downscaled if wider.
    pub max_width: Option<u32>,
    /// Output format: "png" (default) or "jpeg".
    pub format: Option<String>,
}

#[derive(Clone)]
pub struct ScreenshotServer {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl ScreenshotServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        description = "List all visible application windows. Returns JSON array with window id, title, app_name, width, height, and is_focused for each window. Use this to find window IDs or titles for screenshot_window."
    )]
    pub fn list_windows(&self) -> Result<CallToolResult, McpError> {
        info!("list_windows tool called");
        match capture::list_windows() {
            Ok(windows) => {
                let json = serde_json::to_string_pretty(&windows).unwrap_or_default();
                info!(count = windows.len(), "list_windows returning results");
                Ok(CallToolResult::success(vec![Content::text(json)]))
            }
            Err(e) => {
                error!(error = %e, "list_windows failed");
                Ok(CallToolResult::error(vec![Content::text(e.to_string())]))
            }
        }
    }

    #[tool(
        description = "Capture a screenshot of an application window. Find a window by title (case-insensitive substring), exact id, or capture the currently focused window (if no title or id given). Returns the screenshot image and window info text."
    )]
    pub fn screenshot_window(
        &self,
        Parameters(params): Parameters<ScreenshotWindowParams>,
    ) -> Result<CallToolResult, McpError> {
        info!(?params, "screenshot_window tool called");
        let result = (|| -> anyhow::Result<CallToolResult> {
            let (window, window_info) = capture::find_window(params.id, params.title.as_deref())?;
            let img = capture::capture_window(&window)?;
            let fmt = ImageFormat::from_str_opt(params.format.as_deref());
            let (b64, mime) = imaging::image_to_base64(&img, params.max_width, fmt)?;
            let info_text = format!(
                "Window: {} ({})\nID: {}\nSize: {}x{}\nFocused: {}",
                window_info.title,
                window_info.app_name,
                window_info.id,
                window_info.width,
                window_info.height,
                window_info.is_focused
            );
            Ok(CallToolResult::success(vec![
                Content::image(b64, mime),
                Content::text(info_text),
            ]))
        })();

        match result {
            Ok(r) => Ok(r),
            Err(e) => {
                error!(error = %e, "screenshot_window failed");
                Ok(CallToolResult::error(vec![Content::text(e.to_string())]))
            }
        }
    }

    #[tool(
        description = "Capture a screenshot of an entire screen/monitor. By default captures the primary monitor. Specify monitor_id to capture a different monitor. Returns the screenshot image and monitor info text."
    )]
    pub fn screenshot_screen(
        &self,
        Parameters(params): Parameters<ScreenshotScreenParams>,
    ) -> Result<CallToolResult, McpError> {
        info!(?params, "screenshot_screen tool called");
        let result = (|| -> anyhow::Result<CallToolResult> {
            let (img, monitor_info) = capture::capture_screen(params.monitor_id)?;
            let fmt = ImageFormat::from_str_opt(params.format.as_deref());
            let (b64, mime) = imaging::image_to_base64(&img, params.max_width, fmt)?;
            let info_text = format!(
                "Monitor: {} (id: {})\nSize: {}x{}\nPrimary: {}",
                monitor_info.name,
                monitor_info.id,
                monitor_info.width,
                monitor_info.height,
                monitor_info.is_primary
            );
            Ok(CallToolResult::success(vec![
                Content::image(b64, mime),
                Content::text(info_text),
            ]))
        })();

        match result {
            Ok(r) => Ok(r),
            Err(e) => {
                error!(error = %e, "screenshot_screen failed");
                Ok(CallToolResult::error(vec![Content::text(e.to_string())]))
            }
        }
    }
}

#[tool_handler]
impl ServerHandler for ScreenshotServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "MCP server for capturing application window and screen screenshots.".into(),
            ),
        }
    }
}
