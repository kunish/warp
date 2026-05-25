//! CLI argument conversion into shared local-control selectors.
use local_control::protocol::{
    ControlError, ErrorCode, PaneSelector, PaneTarget, SessionSelector, SessionTarget, TabSelector,
    TabTarget, TargetSelector, WindowSelector, WindowTarget,
};
use local_control::selection::InstanceSelector;

use crate::local_control::TargetArgs;

pub(super) fn instance_selector(args: TargetArgs) -> InstanceSelector {
    if let Some(instance_id) = args.instance {
        return InstanceSelector::Id(local_control::discovery::InstanceId(instance_id));
    }
    if let Some(pid) = args.pid {
        return InstanceSelector::Pid(pid);
    }
    InstanceSelector::Active
}

pub(super) fn target_selector(args: &TargetArgs) -> Result<TargetSelector, ControlError> {
    Ok(TargetSelector {
        window: args
            .window
            .as_deref()
            .map(parse_window_target)
            .transpose()?,
        tab: args.tab.as_deref().map(parse_tab_target).transpose()?,
        pane: args.pane.as_deref().map(parse_pane_target).transpose()?,
        session: args
            .session
            .as_deref()
            .map(parse_session_target)
            .transpose()?,
        block: None,
        file: None,
        drive: None,
    })
}

fn parse_window_target(value: &str) -> Result<WindowTarget, ControlError> {
    if value == "active" {
        return Ok(WindowTarget::Active);
    }
    if let Some(id) = value.strip_prefix("id:") {
        return Ok(WindowTarget::Id {
            id: WindowSelector(id.to_owned()),
        });
    }
    if let Some(index) = value.strip_prefix("index:") {
        return Ok(WindowTarget::Index {
            index: parse_selector_index("window", index)?,
        });
    }
    if let Some(title) = value.strip_prefix("title:") {
        return Ok(WindowTarget::Title {
            title: title.to_owned(),
        });
    }
    Err(invalid_selector("window", value))
}

fn parse_tab_target(value: &str) -> Result<TabTarget, ControlError> {
    if value == "active" {
        return Ok(TabTarget::Active);
    }
    if let Some(id) = value.strip_prefix("id:") {
        return Ok(TabTarget::Id {
            id: TabSelector(id.to_owned()),
        });
    }
    if let Some(index) = value.strip_prefix("index:") {
        return Ok(TabTarget::Index {
            index: parse_selector_index("tab", index)?,
        });
    }
    if let Some(title) = value.strip_prefix("title:") {
        return Ok(TabTarget::Title {
            title: title.to_owned(),
        });
    }
    Err(invalid_selector("tab", value))
}

fn parse_pane_target(value: &str) -> Result<PaneTarget, ControlError> {
    if value == "active" {
        return Ok(PaneTarget::Active);
    }
    if let Some(id) = value.strip_prefix("id:") {
        return Ok(PaneTarget::Id {
            id: PaneSelector(id.to_owned()),
        });
    }
    if let Some(index) = value.strip_prefix("index:") {
        return Ok(PaneTarget::Index {
            index: parse_selector_index("pane", index)?,
        });
    }
    Err(invalid_selector("pane", value))
}

fn parse_session_target(value: &str) -> Result<SessionTarget, ControlError> {
    if value == "active" {
        return Ok(SessionTarget::Active);
    }
    if let Some(id) = value.strip_prefix("id:") {
        return Ok(SessionTarget::Id {
            id: SessionSelector(id.to_owned()),
        });
    }
    Err(invalid_selector("session", value))
}

fn parse_selector_index(selector_name: &str, value: &str) -> Result<u32, ControlError> {
    value.parse::<u32>().map_err(|err| {
        ControlError::with_details(
            ErrorCode::InvalidSelector,
            format!("{selector_name} selector index must be a non-negative integer"),
            err.to_string(),
        )
    })
}

fn invalid_selector(selector_name: &str, value: &str) -> ControlError {
    ControlError::new(
        ErrorCode::InvalidSelector,
        format!("invalid {selector_name} selector {value:?}"),
    )
}
