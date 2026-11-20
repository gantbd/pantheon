use super::*;
use std::{collections::VecDeque, sync::Arc};
use vellum_sdk_client::protocol::{
    ClientId, CommandV1, OutcomeV1, RefusalV1, ReplyV1, RequestV1, ServiceAccessErrorV1,
    ServiceControlErrorV1, SnapshotV1, VersionWitness, WorkspaceDocumentIdV1, WorkspaceIdV1,
    WorkspaceInventoryV1,
};

#[allow(dead_code)]
mod forged {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../design/generated/vellum_coarse_navigation_tests.rs"
    ));
}

use forged::*;

struct ScriptedPort {
    replies: VecDeque<Result<ReplyV1, ServiceAccessErrorV1>>,
    commands: Vec<CommandV1>,
}

impl ServicePort for ScriptedPort {
    fn connect(&mut self) -> Result<ClientId, ServiceControlErrorV1> {
        Ok(ClientId::new(1).unwrap())
    }

    fn execute(&mut self, request: RequestV1) -> Result<ReplyV1, ServiceAccessErrorV1> {
        self.commands.push(request.command);
        self.replies.pop_front().expect("one scripted owner reply")
    }

    fn disconnect(&mut self, _: ClientId) -> Result<(), ServiceControlErrorV1> {
        Ok(())
    }
}

struct Fixture {
    editor: Option<ServiceEditor<ScriptedPort>>,
    source: Arc<str>,
    document: WorkspaceDocumentIdV1,
    requests: u64,
    persistence: vellum_sdk_client::workspace::PersistenceProjectionV1,
    selected: usize,
}

impl Fixture {
    fn new() -> Self {
        Self::with_source(Arc::from("abcdef\nx\nabcdef\nx\nabcdef"))
    }

    fn with_source(source: Arc<str>) -> Self {
        let workspace = WorkspaceIdV1::new(7).unwrap();
        let document = WorkspaceDocumentIdV1::new(workspace, 9).unwrap();
        let port = ScriptedPort {
            replies: [
                Ok(ReplyV1::Accepted(OutcomeV1::WorkspaceInventory(
                    WorkspaceInventoryV1 {
                        workspace,
                        documents: vec![document],
                        roots: Vec::new(),
                    },
                ))),
                Ok(ReplyV1::Accepted(OutcomeV1::ViewOpened(
                    vellum_sdk_client::protocol::ViewOpenReceiptV1 {
                        document,
                        status: vellum_sdk_client::protocol::ViewOpenStatusV1::Opened,
                    },
                ))),
                Ok(ReplyV1::Accepted(OutcomeV1::Snapshot(SnapshotV1 {
                    witness: VersionWitness::new(3).unwrap(),
                    revision: 4,
                    text: source.clone(),
                }))),
            ]
            .into_iter()
            .collect(),
            commands: Vec::new(),
        };
        let workspace = WorkspaceClientV1::connect(port).unwrap();
        let editor = ServiceEditor::open(workspace).unwrap();
        let requests = editor.issued_requests();
        let persistence = editor.aggregate_persistence();
        let selected = editor.selected_document_count();
        Self {
            editor: Some(editor),
            source,
            document,
            requests,
            persistence,
            selected,
        }
    }

    fn editor(&mut self) -> &mut ServiceEditor<ScriptedPort> {
        self.editor.as_mut().expect("owner fixture remains live")
    }
}

struct Adapter {
    fixture: Fixture,
    translation: bool,
    movement: bool,
    boundaries: bool,
    protected: bool,
    composition: bool,
    evidence: bool,
    evaluated: Vec<&'static str>,
}

impl Adapter {
    fn new() -> Self {
        Self {
            fixture: Fixture::new(),
            translation: false,
            movement: false,
            boundaries: false,
            protected: false,
            composition: false,
            evidence: false,
            evaluated: Vec::new(),
        }
    }

    fn fixture_ready(&mut self) -> bool {
        let fixture = &mut self.fixture;
        let document = fixture.document;
        let source = fixture.source.clone();
        let requests = fixture.requests;
        let persistence = fixture.persistence;
        let selected = fixture.selected;
        let editor = fixture.editor();
        editor.current_document() == Some(document)
            && editor.workspace().projection().snapshot().unwrap().text == source
            && editor.issued_requests() == requests
            && editor.aggregate_persistence() == persistence
            && editor.selected_document_count() == selected
    }

    fn prove_translation(&mut self) -> bool {
        let translated = |code, modifiers, profile| {
            translate_service_input(translate_vellum_key(
                KeyEvent::new(code, modifiers),
                profile,
            ))
        };
        self.translation = translated(
            KeyCode::PageUp,
            KeyModifiers::NONE,
            KeybindingProfile::Pantheon,
        ) == ServiceInput::Apply(ServiceAction::PageUp)
            && translated(
                KeyCode::PageDown,
                KeyModifiers::NONE,
                KeybindingProfile::Pantheon,
            ) == ServiceInput::Apply(ServiceAction::PageDown)
            && translated(
                KeyCode::Char('v'),
                KeyModifiers::ALT,
                KeybindingProfile::Emacs,
            ) == ServiceInput::Apply(ServiceAction::PageUp)
            && translated(
                KeyCode::Char('v'),
                KeyModifiers::CONTROL,
                KeybindingProfile::Emacs,
            ) == ServiceInput::Apply(ServiceAction::PageDown);
        self.translation
    }

    fn prove_movement(&mut self) -> bool {
        let editor = self.fixture.editor();
        for _ in 0..4 {
            editor.apply(ServiceAction::Right).unwrap();
        }
        editor.observe_viewport_lines(2);
        let first = editor.apply_observed(ServiceAction::PageDown).unwrap();
        let short = editor.frame(2).unwrap().cursor;
        let second = editor.apply_observed(ServiceAction::PageDown).unwrap();
        let recovered = editor.frame(2).unwrap().cursor;
        editor.observe_viewport_lines(0);
        let zero = editor.apply_observed(ServiceAction::PageDown).unwrap();
        editor.observe_viewport_lines(u16::MAX);
        let bounded = editor.apply_observed(ServiceAction::PageDown).unwrap();
        let retained = editor.frame(2).unwrap().cursor;
        let mut oversized_fixture = Fixture::with_source(Arc::from("x\n".repeat(600)));
        let oversized_editor = oversized_fixture.editor();
        oversized_editor.observe_viewport_lines(u16::MAX);
        let oversized = oversized_editor
            .apply_observed(ServiceAction::PageDown)
            .unwrap();
        let oversized_cursor = oversized_editor.frame(512).unwrap().cursor;
        self.movement = first == vellum_tui::ServiceActionOutcome::Moved
            && short == Some((1, 1))
            && second == vellum_tui::ServiceActionOutcome::Moved
            && recovered == Some((4, 1))
            && zero == vellum_tui::ServiceActionOutcome::Boundary
            && bounded == vellum_tui::ServiceActionOutcome::Moved
            && retained == Some((4, 1))
            && oversized == vellum_tui::ServiceActionOutcome::Moved
            && oversized_cursor == Some((0, 511));
        self.movement
    }

    fn prove_boundaries(&mut self) -> bool {
        let editor = self.fixture.editor();
        let lower = editor.apply_observed(ServiceAction::PageDown).unwrap();
        let upward = editor.apply_observed(ServiceAction::PageUp).unwrap();
        let upper = editor.apply_observed(ServiceAction::PageUp).unwrap();
        self.boundaries = lower == vellum_tui::ServiceActionOutcome::Boundary
            && upward == vellum_tui::ServiceActionOutcome::Moved
            && upper == vellum_tui::ServiceActionOutcome::Boundary;
        self.boundaries
    }

    fn prove_protected_effects(&mut self) -> bool {
        let source = self.fixture.source.clone();
        let document = self.fixture.document;
        let requests = self.fixture.requests;
        let persistence = self.fixture.persistence;
        let selected = self.fixture.selected;
        let editor = self.fixture.editor.take().expect("live owner fixture");
        let unchanged = editor.workspace().projection().snapshot().unwrap().text == source
            && editor.current_document() == Some(document)
            && editor.issued_requests() == requests
            && editor.aggregate_persistence() == persistence
            && editor.selected_document_count() == selected
            && !editor.frame(2).unwrap().dirty;
        let port = editor.disconnect().unwrap();
        self.protected = unchanged && port.commands.len() == requests as usize;
        self.protected
    }

    fn prove_composition(&mut self) -> bool {
        let page_down = KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE);
        let mut facade = BehaviorFacade::new(KeybindingProfile::Pantheon);
        let active = facade.key_event(page_down, WorkbenchOverlay::None, Surface::Vellum, true)
            == KeyBehavior::SurfaceLocal;
        let modal = facade.key_event(page_down, WorkbenchOverlay::Commands, Surface::Vellum, true)
            == KeyBehavior::CommandPaletteInput;
        let unfocused = !vellum_surface_active(Surface::Overview, Focus::Vellum)
            && matches!(
                route_surface_local_key(
                    page_down,
                    Surface::Overview,
                    Focus::Vellum,
                    KeybindingProfile::Pantheon,
                ),
                RoutedInput::Suppress
            );
        let actions = [translate_service_input(translate_vellum_key(
            page_down,
            KeybindingProfile::Pantheon,
        ))];
        self.composition = active
            && modal
            && unfocused
            && vellum_viewport_lines(32, Surface::Vellum) == 24
            && vellum_viewport_lines(32, Surface::Workspace) == 22
            && actions == [ServiceInput::Apply(ServiceAction::PageDown)];
        self.composition
    }

    fn prove_evidence(&mut self) -> bool {
        let cases = [
            (Ok(vellum_tui::ServiceActionOutcome::Moved), "owner_moved"),
            (
                Ok(vellum_tui::ServiceActionOutcome::Boundary),
                "owner_boundary",
            ),
            (
                Err(vellum_tui::ServiceEditorError(
                    vellum_sdk_client::workspace::WorkspaceClientErrorV1::Refused(
                        RefusalV1::AuthorityDenied,
                    ),
                )),
                "owner_refused",
            ),
            (
                Err(vellum_tui::ServiceEditorError(
                    vellum_sdk_client::workspace::WorkspaceClientErrorV1::Access(
                        ServiceAccessErrorV1::Unavailable,
                    ),
                )),
                "owner_failed",
            ),
        ];
        self.evidence = cases.iter().all(|(result, expected_settlement)| {
            let mut instrumentation = Instrumentation::default();
            observe_vellum_action(
                &mut instrumentation,
                "vellum.cursor.page-down",
                Instant::now(),
                result,
            );
            let [trace] = instrumentation.traces() else {
                return false;
            };
            let stages: Vec<_> = trace.stages.iter().map(|stage| stage.stage).collect();
            let rendered = format!("{trace:?}");
            trace.complete
                && trace.action == "vellum.cursor.page-down"
                && trace.peer == "vellum"
                && stages
                    == [
                        "shell_dispatch",
                        "peer_pending",
                        *expected_settlement,
                        "terminal_disposition",
                    ]
                && [
                    "path:",
                    "cursor:",
                    "cursor_",
                    "document:",
                    "text:",
                    "payload:",
                    "key:",
                ]
                .iter()
                .all(|forbidden| !rendered.contains(forbidden))
        });
        self.evidence
    }
}

impl VellumCoarseNavigationTestsEvaluator for Adapter {
    fn evaluate(&mut self, obligation: &VellumCoarseNavigationTestsObligation) -> bool {
        self.evaluated.push(obligation.digest);
        match obligation.digest {
            "sha256:e9eeaab1dee482d60251be8ac2307a9469e1aebd04d3dde80255ceacb8672f5d" => {
                self.fixture_ready()
            }
            "sha256:98dcd95b1efe160cceca547b7c09d1a4ffea565cb583ecd60a51c506a7410a23" => {
                self.prove_translation()
            }
            "sha256:2ea7b2e0d31c02ecf03bee83ee13ff4c9bb3ec516f58c6f8a3b2b8b246f6dd78" => {
                self.translation
            }
            "sha256:cd7fd57a1d9ad13a104448a19636cfb0c0b521ed4b8f0ac34a495e7a381ee5c8" => {
                self.prove_movement()
            }
            "sha256:86b5933b4fdfc7b0615062e4bf9f125d0f5f3ef101740faf891be8f608fb22db" => {
                self.movement
            }
            "sha256:fa4b8bfac7326c1485cf7152fa1cdb34db282f6aab62974d560c2afaa03d0f8f" => {
                self.prove_boundaries()
            }
            "sha256:b1470aafc23df33180db3ce0e9ac762df80f2107a9ead99fb3215c1fd8c1f1bb" => {
                self.boundaries
            }
            "sha256:17471801b4b2beb60e6959b131201fd8032f747937d0426c99ec92355ba95bed" => {
                self.prove_protected_effects()
            }
            "sha256:a7eec2042984dcb015a818ca01a886d9be19de2f47edcc2fa05c7dc9fe3f3874" => {
                self.protected
            }
            "sha256:a57a6c68d93330fd3a03f3aa43137f8eee5920d3e2812e99c3babcc08602eefa" => {
                self.prove_composition()
            }
            "sha256:e82c7f624d90029a29e7eaa58ea8214986be7a687f7b28ff5b802ec6de8807a5" => {
                self.composition
            }
            "sha256:f22d64f0f1353301ec7e87097ea95df0624540cc04a5ed4c6121c3f7cda7b09a" => {
                self.prove_evidence()
            }
            "sha256:9ea85424dafddfd6050eb043c1fa505a357786ed506b12dca5dd80074bb44032" => {
                self.evidence
            }
            "sha256:890cef1697b4ca859076346d073de47b79cc7687a4060daee573a105df949b64" => {
                self.evaluated.len() == 14
            }
            _ => false,
        }
    }
}

struct NoChoice;

impl VellumCoarseNavigationTestsChooser for NoChoice {
    fn choose(
        &mut self,
        _: VellumCoarseNavigationTestsState,
        _: &[VellumCoarseNavigationTestsCandidate],
    ) -> Option<usize> {
        panic!("the forged regression machine is compelled")
    }
}

#[derive(Default)]
struct Observer(Vec<VellumCoarseNavigationTestsObservation>);

impl VellumCoarseNavigationTestsObserver for Observer {
    fn observe(&mut self, observation: &VellumCoarseNavigationTestsObservation) {
        self.0.push(*observation);
    }
}

#[test]
fn forged_coarse_navigation_contract_reaches_complete_only_on_actual_evidence() {
    let mut adapter = Adapter::new();
    let mut chooser = NoChoice;
    let mut observer = Observer::default();
    let mut state = VellumCoarseNavigationTestsState::INITIAL;
    while state != VellumCoarseNavigationTestsState::Complete {
        state = state
            .advance(&mut adapter, &mut chooser, &mut observer)
            .expect("every forged obligation must be proven")
            .state;
    }
    assert_eq!(adapter.evaluated.len(), 14);
    assert_eq!(observer.0.len(), 7);
    assert!(observer.0.iter().all(|observation| {
        observation.lineage == &VELLUMCOARSENAVIGATIONTESTS_LINEAGE
            && observation.outcome == VellumCoarseNavigationTestsObservationOutcome::Advanced
            && !observation.resource_error
            && !observation.saturation
    }));
}
