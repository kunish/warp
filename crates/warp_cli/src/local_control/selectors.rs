//! CLI argument conversion into shared local-control selectors.
use local_control::protocol::{
    ControlError, ErrorCode, TabSelector, TabTarget, TargetSelector, WindowSelector, WindowTarget,
};
use local_control::selection::InstanceSelector;

use crate::local_control::TargetArgs;

pub(super) fn instance_selector(args: &TargetArgs) -> InstanceSelector {
    if let Some(instance_id) = &args.instance {
        return InstanceSelector::Id(local_control::discovery::InstanceId(instance_id.clone()));
    }
    if let Some(pid) = args.pid {
        return InstanceSelector::Pid(pid);
    }
    InstanceSelector::Active
}

pub(super) fn target_selector(args: &TargetArgs) -> Result<TargetSelector, ControlError> {
    Ok(TargetSelector {
        window: window_target(args)?,
        tab: tab_target(args)?,
        ..TargetSelector::default()
    })
}

fn window_target(args: &TargetArgs) -> Result<Option<WindowTarget>, ControlError> {
    if let Some(raw) = &args.window {
        return parse_window_target(raw).map(Some);
    }
    if let Some(id) = &args.window_id {
        return Ok(Some(WindowTarget::Id {
            id: WindowSelector(id.clone()),
        }));
    }
    if let Some(index) = args.window_index {
        return Ok(Some(WindowTarget::Index { index }));
    }
    if let Some(title) = &args.window_title {
        return Ok(Some(WindowTarget::Title {
            title: title.clone(),
        }));
    }
    Ok(None)
}

fn tab_target(args: &TargetArgs) -> Result<Option<TabTarget>, ControlError> {
    if let Some(raw) = &args.tab {
        return parse_tab_target(raw).map(Some);
    }
    if let Some(id) = &args.tab_id {
        return Ok(Some(TabTarget::Id {
            id: TabSelector(id.clone()),
        }));
    }
    if let Some(index) = args.tab_index {
        return Ok(Some(TabTarget::Index { index }));
    }
    if let Some(title) = &args.tab_title {
        return Ok(Some(TabTarget::Title {
            title: title.clone(),
        }));
    }
    Ok(None)
}

fn parse_window_target(raw: &str) -> Result<WindowTarget, ControlError> {
    if raw == "active" {
        return Ok(WindowTarget::Active);
    }
    if let Some(id) = raw.strip_prefix("id:") {
        return Ok(WindowTarget::Id {
            id: WindowSelector(id.to_owned()),
        });
    }
    if let Some(index) = raw.strip_prefix("index:") {
        return Ok(WindowTarget::Index {
            index: parse_index(index, "window")?,
        });
    }
    if let Some(title) = raw.strip_prefix("title:") {
        return Ok(WindowTarget::Title {
            title: title.to_owned(),
        });
    }
    Err(invalid_selector("window", raw))
}

fn parse_tab_target(raw: &str) -> Result<TabTarget, ControlError> {
    if raw == "active" {
        return Ok(TabTarget::Active);
    }
    if let Some(id) = raw.strip_prefix("id:") {
        return Ok(TabTarget::Id {
            id: TabSelector(id.to_owned()),
        });
    }
    if let Some(index) = raw.strip_prefix("index:") {
        return Ok(TabTarget::Index {
            index: parse_index(index, "tab")?,
        });
    }
    if let Some(title) = raw.strip_prefix("title:") {
        return Ok(TabTarget::Title {
            title: title.to_owned(),
        });
    }
    Err(invalid_selector("tab", raw))
}

fn parse_index(raw: &str, family: &str) -> Result<u32, ControlError> {
    raw.parse::<u32>().map_err(|err| {
        ControlError::with_details(
            ErrorCode::InvalidSelector,
            format!("invalid {family} index selector"),
            err.to_string(),
        )
    })
}

fn invalid_selector(family: &str, raw: &str) -> ControlError {
    ControlError::new(
        ErrorCode::InvalidSelector,
        format!(
            "invalid {family} selector `{raw}`; expected active, id:<id>, index:<n>, or title:<title>"
        ),
    )
}
