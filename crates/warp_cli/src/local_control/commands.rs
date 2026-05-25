//! Implementations for user-facing `warpctrl` command groups.
use local_control::protocol::{
    Action, ActionGetParams, ActionKind, ActionMetadata, ControlError, EmptyParams, ErrorCode,
    RequestEnvelope,
};
use local_control::selection::select_instance;
use serde::Serialize;
use serde_json::json;

use crate::agent::OutputFormat;
use crate::local_control::output::{write_json, write_json_line};
use crate::local_control::selectors::instance_selector;
use crate::local_control::{
    ActionCommand, AppCommand, AppSurfaceArgs, AppSurfaceCommand, AppearanceCommand,
    AppearanceSizeAdjustArgs, BlockCommand, HistoryCommand, InputCommand, InstanceCommand,
    PaneCommand, SessionCommand, SettingCommand, TabCommand, TargetArgs, ThemeCommand,
    WindowCommand,
};

/// Display-oriented projection of a discoverable Warp instance.
#[derive(Serialize)]
struct InstanceSummary {
    instance_id: String,
    pid: u32,
    channel: String,
    app_id: String,
    app_version: Option<String>,
    started_at: String,
    endpoint: Option<local_control::discovery::ControlEndpoint>,
    outside_warp_control_enabled: bool,
    actions: Vec<ActionMetadata>,
}

impl From<local_control::discovery::InstanceRecord> for InstanceSummary {
    fn from(record: local_control::discovery::InstanceRecord) -> Self {
        Self {
            instance_id: record.instance_id.0,
            pid: record.pid,
            channel: record.channel,
            app_id: record.app_id,
            app_version: record.app_version,
            started_at: record.started_at.to_rfc3339(),
            endpoint: record.endpoint,
            outside_warp_control_enabled: record.outside_warp_control_enabled,
            actions: record.actions,
        }
    }
}

pub(super) fn run_instance_command(
    command: InstanceCommand,
    output_format: OutputFormat,
) -> Result<(), ControlError> {
    match command {
        InstanceCommand::List => {
            let summaries = local_control::discovery::list_instances()
                .into_iter()
                .map(InstanceSummary::from)
                .collect::<Vec<_>>();
            match output_format {
                OutputFormat::Json => write_json(&summaries),
                OutputFormat::Ndjson => {
                    for summary in summaries {
                        write_json_line(&summary)?;
                    }
                    Ok(())
                }
                OutputFormat::Pretty | OutputFormat::Text => {
                    for summary in summaries {
                        let endpoint = summary
                            .endpoint
                            .as_ref()
                            .map(|endpoint| format!("{}:{}", endpoint.host, endpoint.port))
                            .unwrap_or_else(|| "outside_warp_disabled".to_owned());
                        println!(
                            "{}\tpid={}\t{}\t{}",
                            summary.instance_id, summary.pid, summary.channel, endpoint
                        );
                    }
                    Ok(())
                }
            }
        }
    }
}

pub(super) fn run_app_command(
    command: AppCommand,
    output_format: OutputFormat,
) -> Result<(), ControlError> {
    match command {
        AppCommand::Ping(args) => run_action(args, ActionKind::AppPing, json!({}), output_format),
        AppCommand::Version(args) => {
            run_action(args, ActionKind::AppVersion, json!({}), output_format)
        }
        AppCommand::Active(args) => run_action_with_params(
            args,
            ActionKind::AppActive,
            local_control::AppActiveParams::default(),
            output_format,
        ),
        AppCommand::Inspect(args) => run_action_with_params(
            args,
            ActionKind::AppInspect,
            local_control::AppInspectParams::default(),
            output_format,
        ),
    }
}

pub(super) fn run_action_command(
    command: ActionCommand,
    output_format: OutputFormat,
) -> Result<(), ControlError> {
    match command {
        ActionCommand::List(args) => run_action_with_params(
            args,
            ActionKind::ActionList,
            local_control::ActionListParams::default(),
            output_format,
        ),
        ActionCommand::Get(args) => run_action_with_params(
            args.target,
            ActionKind::ActionGet,
            ActionGetParams {
                action: args.action,
            },
            output_format,
        ),
    }
}

pub(super) fn run_app_surface_command(
    command: AppSurfaceCommand,
    output_format: OutputFormat,
) -> Result<(), ControlError> {
    match command {
        AppSurfaceCommand::Focus(args) => {
            run_action(args, ActionKind::AppFocus, json!({}), output_format)
        }
        AppSurfaceCommand::SettingsOpen(args) => {
            run_app_surface_action(args, ActionKind::AppSettingsOpen, output_format)
        }
        AppSurfaceCommand::PaletteOpen(args) => {
            run_app_surface_action(args, ActionKind::AppCommandPaletteOpen, output_format)
        }
        AppSurfaceCommand::SearchOpen(args) => {
            run_app_surface_action(args, ActionKind::AppCommandSearchOpen, output_format)
        }
        AppSurfaceCommand::DriveOpen(args) => {
            run_action(args, ActionKind::AppWarpDriveOpen, json!({}), output_format)
        }
        AppSurfaceCommand::DriveToggle(args) => run_action(
            args,
            ActionKind::AppWarpDriveToggle,
            json!({}),
            output_format,
        ),
        AppSurfaceCommand::ResourceCenterToggle(args) => run_action(
            args,
            ActionKind::AppResourceCenterToggle,
            json!({}),
            output_format,
        ),
        AppSurfaceCommand::AiAssistantToggle(args) => run_action(
            args,
            ActionKind::AppAiAssistantToggle,
            json!({}),
            output_format,
        ),
        AppSurfaceCommand::CodeReviewToggle(args) => run_action(
            args,
            ActionKind::AppCodeReviewToggle,
            json!({}),
            output_format,
        ),
        AppSurfaceCommand::VerticalTabsToggle(args) => run_action(
            args,
            ActionKind::AppVerticalTabsToggle,
            json!({}),
            output_format,
        ),
    }
}

pub(super) fn run_window_command(
    command: WindowCommand,
    output_format: OutputFormat,
) -> Result<(), ControlError> {
    match command {
        WindowCommand::List(args) => {
            run_action_with_params(args, ActionKind::WindowList, EmptyParams {}, output_format)
        }
        WindowCommand::Create(args) => run_action_with_params(
            args,
            ActionKind::WindowCreate,
            local_control::WindowCreateParams::default(),
            output_format,
        ),
        WindowCommand::Focus(args) => {
            run_action(args, ActionKind::WindowFocus, json!({}), output_format)
        }
        WindowCommand::Close(args) => run_action_with_params(
            args.target,
            ActionKind::WindowClose,
            local_control::WindowCloseParams { force: args.force },
            output_format,
        ),
    }
}
pub(super) fn run_tab_command(
    command: TabCommand,
    output_format: OutputFormat,
) -> Result<(), ControlError> {
    match command {
        TabCommand::List(args) => {
            run_action_with_params(args, ActionKind::TabList, EmptyParams {}, output_format)
        }
        TabCommand::Create(args) => {
            run_action(args, ActionKind::TabCreate, json!({}), output_format)
        }
    }
}

pub(super) fn run_pane_command(
    command: PaneCommand,
    output_format: OutputFormat,
) -> Result<(), ControlError> {
    match command {
        PaneCommand::List(args) => {
            run_action_with_params(args, ActionKind::PaneList, EmptyParams {}, output_format)
        }
    }
}

pub(super) fn run_session_command(
    command: SessionCommand,
    output_format: OutputFormat,
) -> Result<(), ControlError> {
    match command {
        SessionCommand::List(args) => {
            run_action_with_params(args, ActionKind::SessionList, EmptyParams {}, output_format)
        }
    }
}

pub(super) fn run_block_command(
    command: BlockCommand,
    output_format: OutputFormat,
) -> Result<(), ControlError> {
    match command {
        BlockCommand::List(args) => run_action_with_params(
            args.target,
            ActionKind::BlockList,
            local_control::BlockListParams { limit: args.limit },
            output_format,
        ),
        BlockCommand::Get(args) => run_action_with_params(
            args.target,
            ActionKind::BlockGet,
            local_control::BlockGetParams {
                block_id: args.block_id,
            },
            output_format,
        ),
    }
}

pub(super) fn run_input_command(
    command: InputCommand,
    output_format: OutputFormat,
) -> Result<(), ControlError> {
    match command {
        InputCommand::Get(args) => run_action_with_params(
            args,
            ActionKind::InputGet,
            local_control::InputGetParams::default(),
            output_format,
        ),
    }
}

pub(super) fn run_theme_command(
    command: ThemeCommand,
    output_format: OutputFormat,
) -> Result<(), ControlError> {
    match command {
        ThemeCommand::List(args) => {
            run_action_with_params(args, ActionKind::ThemeList, EmptyParams {}, output_format)
        }
        ThemeCommand::Set(args) => run_action_with_params(
            args.target,
            ActionKind::ThemeSet,
            local_control::ThemeSetParams { name: args.name },
            output_format,
        ),
    }
}

pub(super) fn run_appearance_command(
    command: AppearanceCommand,
    output_format: OutputFormat,
) -> Result<(), ControlError> {
    match command {
        AppearanceCommand::Get(args) => run_action_with_params(
            args,
            ActionKind::AppearanceGet,
            EmptyParams {},
            output_format,
        ),
        AppearanceCommand::Set(args) => run_action_with_params(
            args.target,
            ActionKind::AppearanceSet,
            local_control::AppearanceSetParams {
                theme: args.theme,
                follow_system_theme: args.follow_system_theme,
                light_theme: args.light_theme,
                dark_theme: args.dark_theme,
            },
            output_format,
        ),
        AppearanceCommand::FontSize(args) => {
            run_appearance_size_action(args, ActionKind::AppearanceFontSize, output_format)
        }
        AppearanceCommand::Zoom(args) => {
            run_appearance_size_action(args, ActionKind::AppearanceZoom, output_format)
        }
    }
}

pub(super) fn run_history_command(
    command: HistoryCommand,
    output_format: OutputFormat,
) -> Result<(), ControlError> {
    match command {
        HistoryCommand::List(args) => run_action_with_params(
            args.target,
            ActionKind::HistoryList,
            local_control::HistoryListParams { limit: args.limit },
            output_format,
        ),
    }
}
pub(super) fn run_setting_command(
    command: SettingCommand,
    output_format: OutputFormat,
) -> Result<(), ControlError> {
    match command {
        SettingCommand::List(args) => run_action_with_params(
            args,
            ActionKind::SettingList,
            local_control::SettingListParams::default(),
            output_format,
        ),
        SettingCommand::Get(args) => run_action_with_params(
            args.target,
            ActionKind::SettingGet,
            local_control::SettingGetParams { key: args.key },
            output_format,
        ),
        SettingCommand::Set(args) => {
            let value = serde_json::from_str(&args.value).map_err(|err| {
                ControlError::with_details(
                    ErrorCode::InvalidParams,
                    "setting.set value must be valid JSON",
                    err.to_string(),
                )
            })?;
            run_action_with_params(
                args.target,
                ActionKind::SettingSet,
                local_control::SettingSetParams {
                    key: args.key,
                    value,
                },
                output_format,
            )
        }
        SettingCommand::Toggle(args) => run_action_with_params(
            args.target,
            ActionKind::SettingToggle,
            local_control::SettingToggleParams { key: args.key },
            output_format,
        ),
    }
}

fn run_app_surface_action(
    args: AppSurfaceArgs,
    action: ActionKind,
    output_format: OutputFormat,
) -> Result<(), ControlError> {
    run_action_with_params(
        args.target,
        action,
        local_control::AppSurfaceParams {
            query: args.query,
            page: args.page,
        },
        output_format,
    )
}

fn run_appearance_size_action(
    args: AppearanceSizeAdjustArgs,
    action: ActionKind,
    output_format: OutputFormat,
) -> Result<(), ControlError> {
    let adjustment = match args.adjustment.to_lowercase().as_str() {
        "increase" => local_control::SizeAdjustment::Increase,
        "decrease" => local_control::SizeAdjustment::Decrease,
        "reset" => local_control::SizeAdjustment::Reset,
        "set" => local_control::SizeAdjustment::Set,
        _ => {
            return Err(ControlError::new(
                ErrorCode::InvalidParams,
                "--adjustment must be increase, decrease, reset, or set",
            ));
        }
    };
    match action {
        ActionKind::AppearanceFontSize => run_action_with_params(
            args.target,
            action,
            local_control::AppearanceFontSizeParams {
                adjustment,
                value: args.value,
            },
            output_format,
        ),
        ActionKind::AppearanceZoom => run_action_with_params(
            args.target,
            action,
            local_control::AppearanceZoomParams {
                adjustment,
                value: args.value,
            },
            output_format,
        ),
        _ => Err(ControlError::new(
            ErrorCode::InvalidParams,
            format!("{} is not a size adjustment action", action.as_str()),
        )),
    }
}

fn run_action(
    args: TargetArgs,
    action: ActionKind,
    params: serde_json::Value,
    output_format: OutputFormat,
) -> Result<(), ControlError> {
    let records = local_control::discovery::list_instances();
    let selector = instance_selector(args);
    let instance = select_instance(&records, &selector)?;
    let request = RequestEnvelope::new(Action {
        kind: action,
        params,
    });
    let response = local_control::client::send_request(&instance, &request)?;
    let local_control::protocol::ControlResponse::Ok { data } = response.response else {
        return Err(ControlError::new(
            ErrorCode::Internal,
            "local-control request failed without an error payload",
        ));
    };
    match output_format {
        OutputFormat::Json => write_json(&data),
        OutputFormat::Ndjson => write_json_line(&data),
        OutputFormat::Pretty | OutputFormat::Text => write_json(&data),
    }
}

fn run_action_with_params<T: Serialize>(
    args: TargetArgs,
    action: ActionKind,
    params: T,
    output_format: OutputFormat,
) -> Result<(), ControlError> {
    let records = local_control::discovery::list_instances();
    let selector = instance_selector(args);
    let instance = select_instance(&records, &selector)?;
    let request = RequestEnvelope::new(Action::with_params(action, params)?);
    let response = local_control::client::send_request(&instance, &request)?;
    let local_control::protocol::ControlResponse::Ok { data } = response.response else {
        return Err(ControlError::new(
            ErrorCode::Internal,
            "local-control request failed without an error payload",
        ));
    };
    match output_format {
        OutputFormat::Json => write_json(&data),
        OutputFormat::Ndjson => write_json_line(&data),
        OutputFormat::Pretty | OutputFormat::Text => write_json(&data),
    }
}
