use ai::agent::action::{AskUserQuestionItem, AskUserQuestionOption, AskUserQuestionType};
use ai::agent::action_result::AskUserQuestionAnswerItem;
use pathfinder_geometry::rect::RectF;
use pathfinder_geometry::vector::{vec2f, Vector2F};
use warp_core::ui::appearance::Appearance;
use warpui::elements::{ParentElement, SavePosition, Stack};
use warpui::platform::WindowStyle;
use warpui::{
    App, AppContext, Element, Entity, Presenter, SingletonEntity, TypedActionView, View,
    ViewContext, ViewHandle, WindowId, WindowInvalidation,
};

use super::{
    ask_user_question_view_state, AskUserQuestionAction, AskUserQuestionEffect,
    AskUserQuestionPhase, AskUserQuestionSession, AskUserQuestionView, AskUserQuestionViewAction,
    AskUserQuestionViewState,
};
use crate::ai::agent::conversation::AIConversationId;
use crate::ai::agent::AIAgentActionId;
use crate::ai::blocklist::BlocklistAIActionModel;
use crate::test_util::terminal::{add_window_with_terminal, initialize_app_for_terminal_view};

const ACTIVE_QUESTION_CARD_POSITION_ID: &str = "active-question-card";
const PREVIOUS_ACTIVE_QUESTION_MAX_HEIGHT: f32 = 320.;

fn build_question(
    question_id: &str,
    question: &str,
    is_multiselect: bool,
    supports_other: bool,
    options: &[&str],
) -> AskUserQuestionItem {
    AskUserQuestionItem {
        question_id: question_id.to_string(),
        question: question.to_string(),
        question_type: AskUserQuestionType::MultipleChoice {
            is_multiselect,
            options: options
                .iter()
                .map(|label| AskUserQuestionOption {
                    label: (*label).to_string(),
                    recommended: false,
                })
                .collect(),
            supports_other,
        },
    }
}

fn build_session(questions: Vec<AskUserQuestionItem>) -> AskUserQuestionSession {
    AskUserQuestionSession::new(questions)
}

fn view_state_for(session: &AskUserQuestionSession) -> AskUserQuestionViewState {
    ask_user_question_view_state(session.current())
}

fn current_draft(session: &AskUserQuestionSession) -> Option<&super::QuestionDraft> {
    session.current().and_then(|current| current.draft)
}
struct ActiveQuestionTestView {
    question_view: ViewHandle<AskUserQuestionView>,
}

impl ActiveQuestionTestView {
    fn new(
        action_model: warpui::ModelHandle<BlocklistAIActionModel>,
        question: AskUserQuestionItem,
        ctx: &mut ViewContext<Self>,
    ) -> Self {
        let question_view = ctx.add_typed_action_view(move |ctx| {
            AskUserQuestionView::new(
                action_model,
                AIConversationId::new(),
                AIAgentActionId::from("layout-test".to_string()),
                vec![question],
                ctx,
            )
        });
        Self { question_view }
    }

    fn last_option_position_id(&self, app: &AppContext) -> String {
        let buttons_id = self.question_view.as_ref(app).buttons.id();
        let last_option_index = self
            .question_view
            .as_ref(app)
            .session
            .current()
            .expect("test question should exist")
            .question
            .numbered_option_count()
            - 1;
        format!("number_shortcut_buttons_{buttons_id}_{last_option_index}")
    }
}

impl Entity for ActiveQuestionTestView {
    type Event = ();
}

impl View for ActiveQuestionTestView {
    fn ui_name() -> &'static str {
        "ActiveQuestionTestView"
    }

    fn render(&self, app: &AppContext) -> Box<dyn Element> {
        let card = self
            .question_view
            .as_ref(app)
            .render_active(Appearance::as_ref(app), app)
            .expect("test question should render");
        Stack::new()
            .with_child(SavePosition::new(card, ACTIVE_QUESTION_CARD_POSITION_ID).finish())
            .finish()
    }
}

impl TypedActionView for ActiveQuestionTestView {
    type Action = AskUserQuestionViewAction;

    fn handle_action(&mut self, _action: &Self::Action, _ctx: &mut ViewContext<Self>) {}
}
fn add_active_question_window(
    app: &mut App,
    question: AskUserQuestionItem,
) -> (WindowId, ViewHandle<ActiveQuestionTestView>) {
    let terminal = add_window_with_terminal(app, None);
    let action_model = terminal.read(app, |terminal, _| {
        terminal.ai_action_model_for_test().clone()
    });
    app.add_window(WindowStyle::NotStealFocus, move |ctx| {
        ActiveQuestionTestView::new(action_model, question, ctx)
    })
}

fn active_question_positions(
    app: &mut App,
    window_id: WindowId,
    view: &ViewHandle<ActiveQuestionTestView>,
    window_size: Vector2F,
) -> (RectF, RectF) {
    let last_option_position_id = view.read(app, |view, app| view.last_option_position_id(app));
    let updated = app.read(|ctx| ctx.view_ids_for_window(window_id).into_iter().collect());
    let invalidation = WindowInvalidation {
        updated,
        ..Default::default()
    };
    let mut presenter = Presenter::new(window_id);
    app.update(move |ctx| {
        presenter.invalidate(invalidation, ctx);
        presenter.build_scene(window_size, 1., None, ctx);
        let card = presenter
            .position_cache()
            .get_position(ACTIVE_QUESTION_CARD_POSITION_ID)
            .expect("active question card should be laid out");
        let last_option = presenter
            .position_cache()
            .get_position(last_option_position_id)
            .expect("last option should be laid out");
        (card, last_option)
    })
}

#[test]
fn active_question_card_grows_beyond_previous_height_cap_for_many_options() {
    App::test((), |mut app| async move {
        initialize_app_for_terminal_view(&mut app);
        let option_labels = (1..=12)
            .map(|index| format!("Option {index}"))
            .collect::<Vec<_>>();
        let options = option_labels.iter().map(String::as_str).collect::<Vec<_>>();
        let question = build_question("q1", "Choose one", false, false, &options);
        let (window_id, view) = add_active_question_window(&mut app, question);

        let (card, last_option) =
            active_question_positions(&mut app, window_id, &view, vec2f(900., 2400.));

        assert!(
            card.size().y() > PREVIOUS_ACTIVE_QUESTION_MAX_HEIGHT,
            "expected card to grow beyond the previous height cap, got {}",
            card.size().y()
        );
        assert!(
            last_option.max_y() <= card.max_y(),
            "expected the last option to fit inside the card: option={last_option:?}, card={card:?}"
        );
    });
}

#[test]
fn active_question_card_grows_to_contain_wrapped_option_text() {
    App::test((), |mut app| async move {
        initialize_app_for_terminal_view(&mut app);
        let wrapped_option = "This is a deliberately long option label that should wrap across \
                              several lines in a narrow Agent questions card instead of being \
                              clipped inside an internally scrollable region.";
        let question = build_question("q1", "Choose one", false, false, &[wrapped_option]);
        let (window_id, view) = add_active_question_window(&mut app, question);

        let (card, last_option) =
            active_question_positions(&mut app, window_id, &view, vec2f(360., 2400.));

        assert!(
            last_option.max_y() <= card.max_y(),
            "expected wrapped option text to fit inside the card: option={last_option:?}, \
             card={card:?}"
        );
    });
}

#[test]
fn enter_on_other_row_focuses_the_other_input() {
    let mut session = build_session(vec![build_question("q1", "Only", false, true, &["Stable"])]);

    assert_eq!(
        session.apply(AskUserQuestionAction::PressEnter {
            highlighted_index: Some(1),
            active_other_text: None,
        }),
        AskUserQuestionEffect::FocusOtherInput
    );
}

#[test]
fn enter_without_an_answer_submits_the_last_question_immediately() {
    let mut session = build_session(vec![build_question(
        "q1",
        "Only",
        false,
        true,
        &["Stable", "Nightly"],
    )]);

    assert_eq!(
        session.apply(AskUserQuestionAction::PressEnter {
            highlighted_index: None,
            active_other_text: None,
        }),
        AskUserQuestionEffect::Submit(vec![AskUserQuestionAnswerItem::Skipped {
            question_id: "q1".to_string(),
        }])
    );
}

#[test]
fn enter_after_a_selected_answer_schedules_auto_advance() {
    let mut session = build_session(vec![build_question(
        "q1",
        "Only",
        false,
        false,
        &["Stable", "Nightly"],
    )]);

    assert_eq!(
        session.apply(AskUserQuestionAction::ToggleOption { option_index: 1 }),
        AskUserQuestionEffect::ScheduleAutoAdvance
    );

    assert_eq!(
        session.apply(AskUserQuestionAction::PressEnter {
            highlighted_index: None,
            active_other_text: None,
        }),
        AskUserQuestionEffect::ScheduleAutoAdvance
    );
}

#[test]
fn enter_with_blank_other_input_submits_the_last_question_immediately() {
    let mut session = build_session(vec![build_question("q1", "Only", false, true, &["Stable"])]);

    assert_eq!(
        session.apply(AskUserQuestionAction::OpenOtherInput),
        AskUserQuestionEffect::FocusOtherInput
    );

    assert_eq!(
        session.apply(AskUserQuestionAction::PressEnter {
            highlighted_index: None,
            active_other_text: None,
        }),
        AskUserQuestionEffect::Submit(vec![AskUserQuestionAnswerItem::Skipped {
            question_id: "q1".to_string(),
        }])
    );
}

#[test]
fn enter_with_active_other_text_schedules_auto_advance() {
    let mut session = build_session(vec![build_question("q1", "Only", false, true, &["Stable"])]);

    assert_eq!(
        session.apply(AskUserQuestionAction::OpenOtherInput),
        AskUserQuestionEffect::FocusOtherInput
    );
    assert_eq!(
        session.apply(AskUserQuestionAction::PressEnter {
            highlighted_index: None,
            active_other_text: Some("nightly".to_string()),
        }),
        AskUserQuestionEffect::ScheduleAutoAdvance
    );
    assert_eq!(
        current_draft(&session).and_then(|draft| draft.other_text.as_deref()),
        Some("nightly")
    );
}

#[test]
fn enter_on_single_select_option_toggles_it_and_schedules_auto_advance() {
    let mut session = build_session(vec![build_question(
        "q1",
        "Only",
        false,
        true,
        &["Stable", "Nightly"],
    )]);

    assert_eq!(
        session.apply(AskUserQuestionAction::PressEnter {
            highlighted_index: Some(1),
            active_other_text: None,
        }),
        AskUserQuestionEffect::ScheduleAutoAdvance
    );
    assert!(current_draft(&session).is_some_and(|draft| draft.selected_option_indices.contains(&1)));
}

#[test]
fn enter_on_non_last_multi_select_option_toggles_it_and_schedules_auto_advance() {
    let mut session = build_session(vec![
        build_question("q1", "First", true, true, &["Stable", "Nightly"]),
        build_question("q2", "Second", false, false, &["CLI"]),
    ]);

    assert_eq!(
        session.apply(AskUserQuestionAction::PressEnter {
            highlighted_index: Some(1),
            active_other_text: None,
        }),
        AskUserQuestionEffect::ScheduleAutoAdvance
    );
    assert!(current_draft(&session).is_some_and(|draft| draft.selected_option_indices.contains(&1)));
}

#[test]
fn enter_on_answered_non_last_multi_select_option_keeps_existing_selection_and_advances() {
    let mut session = build_session(vec![
        build_question("q1", "First", true, true, &["Stable", "Nightly"]),
        build_question("q2", "Second", false, false, &["CLI"]),
    ]);

    assert_eq!(
        session.apply(AskUserQuestionAction::ToggleOption { option_index: 0 }),
        AskUserQuestionEffect::RefreshCurrent
    );
    assert_eq!(
        session.apply(AskUserQuestionAction::PressEnter {
            highlighted_index: Some(1),
            active_other_text: None,
        }),
        AskUserQuestionEffect::ScheduleAutoAdvance
    );
    assert!(current_draft(&session).is_some_and(|draft| {
        draft.selected_option_indices.contains(&0) && draft.selected_option_indices.contains(&1)
    }));
}

#[test]
fn single_select_non_last_toggle_schedules_auto_advance() {
    let mut session = build_session(vec![
        build_question("q1", "First", false, false, &["Rust", "Go"]),
        build_question("q2", "Second", false, false, &["CLI", "GUI"]),
    ]);

    let effect = session.apply(AskUserQuestionAction::ToggleOption { option_index: 1 });

    assert_eq!(effect, AskUserQuestionEffect::ScheduleAutoAdvance);
    assert_eq!(session.current_question_index(), 0);
    assert!(current_draft(&session).is_some_and(|draft| draft.selected_option_indices.contains(&1)));
    assert!(matches!(session.phase(), AskUserQuestionPhase::Editing));
}

#[test]
fn multi_select_non_last_toggle_does_not_auto_advance() {
    let mut session = build_session(vec![
        build_question("q1", "First", true, false, &["Rust", "Go"]),
        build_question("q2", "Second", false, false, &["CLI", "GUI"]),
    ]);

    let effect = session.apply(AskUserQuestionAction::ToggleOption { option_index: 1 });

    assert_eq!(effect, AskUserQuestionEffect::RefreshCurrent);
    assert_eq!(session.current_question_index(), 0);
    assert!(current_draft(&session).is_some_and(|draft| draft.selected_option_indices.contains(&1)));
    assert!(matches!(session.phase(), AskUserQuestionPhase::Editing));
}

#[test]
fn last_multi_select_toggle_schedules_auto_advance() {
    let mut session = build_session(vec![build_question(
        "q1",
        "Only",
        true,
        false,
        &["Rust", "Go"],
    )]);

    let effect = session.apply(AskUserQuestionAction::ToggleOption { option_index: 0 });

    assert_eq!(effect, AskUserQuestionEffect::ScheduleAutoAdvance);
    assert_eq!(session.current_question_index(), 0);
    assert!(current_draft(&session).is_some_and(|draft| draft.selected_option_indices.contains(&0)));
}

#[test]
fn single_select_clicking_selected_option_clears_the_draft() {
    let mut session = build_session(vec![build_question(
        "q1",
        "Only",
        false,
        false,
        &["Rust", "Go"],
    )]);

    assert_eq!(
        session.apply(AskUserQuestionAction::ToggleOption { option_index: 0 }),
        AskUserQuestionEffect::ScheduleAutoAdvance
    );
    assert_eq!(
        session.apply(AskUserQuestionAction::ToggleOption { option_index: 0 }),
        AskUserQuestionEffect::RefreshCurrent
    );
    assert!(current_draft(&session).is_none());
    assert_eq!(session.current_question_index(), 0);
}

#[test]
fn drafts_survive_navigation_and_submit_skips_only_unanswered_questions() {
    let mut session = build_session(vec![
        build_question("q1", "First", true, false, &["Rust", "Go"]),
        build_question("q2", "Second", true, false, &["CLI", "GUI"]),
        build_question("q3", "Third", false, true, &["Stable"]),
    ]);

    assert_eq!(
        session.apply(AskUserQuestionAction::ToggleOption { option_index: 0 }),
        AskUserQuestionEffect::RefreshCurrent
    );
    assert_eq!(
        session.apply(AskUserQuestionAction::NavigateNext),
        AskUserQuestionEffect::ShowQuestion
    );
    assert_eq!(
        session.apply(AskUserQuestionAction::NavigatePrev),
        AskUserQuestionEffect::ShowQuestion
    );
    assert_eq!(
        current_draft(&session).map(|draft| draft.selected_option_indices.len()),
        Some(1)
    );
    assert_eq!(
        session.apply(AskUserQuestionAction::NavigateNext),
        AskUserQuestionEffect::ShowQuestion
    );
    assert_eq!(
        session.apply(AskUserQuestionAction::NavigateNext),
        AskUserQuestionEffect::ShowQuestion
    );
    assert_eq!(
        session.apply(AskUserQuestionAction::OpenOtherInput),
        AskUserQuestionEffect::FocusOtherInput
    );
    assert_eq!(
        session.apply(AskUserQuestionAction::SaveOtherText {
            text: Some("nightly toolchain".to_string()),
        }),
        AskUserQuestionEffect::ScheduleAutoAdvance
    );

    let effect = session.apply(AskUserQuestionAction::Confirm);

    assert_eq!(
        effect,
        AskUserQuestionEffect::Submit(vec![
            AskUserQuestionAnswerItem::Answered {
                question_id: "q1".to_string(),
                selected_options: vec!["Rust".to_string()],
                other_text: String::new(),
            },
            AskUserQuestionAnswerItem::Skipped {
                question_id: "q2".to_string(),
            },
            AskUserQuestionAnswerItem::Answered {
                question_id: "q3".to_string(),
                selected_options: vec![],
                other_text: "nightly toolchain".to_string(),
            },
        ])
    );
    assert!(matches!(
        session.phase(),
        AskUserQuestionPhase::Completed { .. }
    ));
}

#[test]
fn multi_select_other_text_does_not_auto_advance_before_last_question() {
    let mut session = build_session(vec![
        build_question("q1", "First", true, true, &["Rust"]),
        build_question("q2", "Second", false, false, &["CLI"]),
    ]);

    assert_eq!(
        session.apply(AskUserQuestionAction::OpenOtherInput),
        AskUserQuestionEffect::FocusOtherInput
    );
    assert_eq!(
        session.apply(AskUserQuestionAction::SaveOtherText {
            text: Some("nightly".to_string()),
        }),
        AskUserQuestionEffect::RefreshCurrent
    );
    assert_eq!(session.current_question_index(), 0);
    assert_eq!(
        current_draft(&session).and_then(|draft| draft.other_text.as_deref()),
        Some("nightly")
    );
}

#[test]
fn skip_all_moves_session_to_completed_with_skipped_answers() {
    let mut session = build_session(vec![
        build_question("q1", "First", true, false, &["Rust"]),
        build_question("q2", "Second", false, true, &["Stable"]),
    ]);

    session.apply(AskUserQuestionAction::ToggleOption { option_index: 0 });
    session.apply(AskUserQuestionAction::NavigateNext);
    session.apply(AskUserQuestionAction::OpenOtherInput);
    session.apply(AskUserQuestionAction::SaveOtherText {
        text: Some("nightly".to_string()),
    });

    let effect = session.apply(AskUserQuestionAction::SkipAll);

    assert_eq!(
        effect,
        AskUserQuestionEffect::Submit(vec![
            AskUserQuestionAnswerItem::Skipped {
                question_id: "q1".to_string(),
            },
            AskUserQuestionAnswerItem::Skipped {
                question_id: "q2".to_string(),
            },
        ])
    );
    assert!(matches!(
        session.phase(),
        AskUserQuestionPhase::Completed { .. }
    ));
}

#[test]
fn other_text_submission_exits_input_and_submits_last_question() {
    let mut session = build_session(vec![build_question("q1", "Only", false, true, &["Stable"])]);

    assert_eq!(
        session.apply(AskUserQuestionAction::OpenOtherInput),
        AskUserQuestionEffect::FocusOtherInput
    );
    assert!(view_state_for(&session).show_other_input);

    assert_eq!(
        session.apply(AskUserQuestionAction::SaveOtherText {
            text: Some("nightly".to_string()),
        }),
        AskUserQuestionEffect::ScheduleAutoAdvance
    );

    let draft = current_draft(&session).expect("draft should exist");
    assert_eq!(draft.other_text.as_deref(), Some("nightly"));
    assert!(!draft.is_other_input_active);
    assert!(!view_state_for(&session).show_other_input);

    let effect = session.apply(AskUserQuestionAction::Confirm);

    assert_eq!(
        effect,
        AskUserQuestionEffect::Submit(vec![AskUserQuestionAnswerItem::Answered {
            question_id: "q1".to_string(),
            selected_options: vec![],
            other_text: "nightly".to_string(),
        }])
    );
}

#[test]
fn navigating_next_on_last_question_is_a_noop() {
    let mut session = build_session(vec![build_question(
        "q1",
        "Only",
        false,
        false,
        &["Rust", "Go"],
    )]);

    assert_eq!(
        session.apply(AskUserQuestionAction::ToggleOption { option_index: 0 }),
        AskUserQuestionEffect::ScheduleAutoAdvance
    );
    assert_eq!(
        session.apply(AskUserQuestionAction::NavigateNext),
        AskUserQuestionEffect::Noop
    );
    assert!(matches!(session.phase(), AskUserQuestionPhase::Editing));
    assert!(current_draft(&session).is_some_and(|draft| draft.selected_option_indices.contains(&0)));
}

#[test]
fn view_state_shows_other_input() {
    let mut session = build_session(vec![
        build_question("q1", "First", false, true, &["Stable"]),
        build_question("q2", "Second", false, false, &["CLI"]),
    ]);

    assert_eq!(
        view_state_for(&session),
        AskUserQuestionViewState {
            show_other_input: false,
        }
    );

    assert_eq!(
        session.apply(AskUserQuestionAction::OpenOtherInput),
        AskUserQuestionEffect::FocusOtherInput
    );
    assert_eq!(
        view_state_for(&session),
        AskUserQuestionViewState {
            show_other_input: true,
        }
    );

    assert_eq!(
        session.apply(AskUserQuestionAction::SaveOtherText {
            text: Some("nightly".to_string()),
        }),
        AskUserQuestionEffect::ScheduleAutoAdvance
    );
    assert_eq!(
        session.apply(AskUserQuestionAction::NavigateNext),
        AskUserQuestionEffect::ShowQuestion
    );

    assert_eq!(
        view_state_for(&session),
        AskUserQuestionViewState {
            show_other_input: false,
        }
    );
}
