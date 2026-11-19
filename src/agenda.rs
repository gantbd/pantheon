//! Host-owned, read-only projection of owner-native operator attention facts.
//!
//! Entries retain their source identity and revision. Presence in this view is
//! never authority, approval, assignment, or evidence that an action settled.

use serde::{Deserialize, Serialize};

pub const MAX_AGENDA_ENTRIES: usize = 128;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgendaKind {
    DependencyBlocked,
    OutcomeUnknown,
    GovernedPermission,
    HumanGate,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgendaUrgency {
    Attention,
    Blocking,
}

/// Whether Pantheon can honestly route an owner-native action from the facts
/// currently projected. This is presentation readiness, never authority.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgendaActionability {
    Actionable,
    InspectOnly,
    Blocked,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgendaEntry {
    pub source_peer: String,
    pub source_instance: Option<String>,
    pub source_revision: String,
    pub resource_identity: String,
    pub kind: AgendaKind,
    pub urgency: AgendaUrgency,
    pub actionability: AgendaActionability,
    pub blocker: Option<String>,
    pub title: String,
    pub why_operator: String,
    pub authority: String,
    pub requested_action: String,
    pub consequence_of_waiting: String,
    pub next_owner_action: String,
    pub route: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct OperatingReviewReadiness {
    pub owner_configured: bool,
    pub status: String,
    pub required_owner: String,
    pub next_action: String,
}

impl Default for OperatingReviewReadiness {
    fn default() -> Self {
        Self {
            owner_configured: false,
            status: "Unavailable · no sovereign work-mode instance owner configured".into(),
            required_owner:
                "revisioned BusinessReviewV1/ProjectProposalV1 registry and instance owner".into(),
            next_action: "select and qualify the owner before scheduling sponsor sign-off".into(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct OperatorAgendaView {
    pub entries: Vec<AgendaEntry>,
    pub truncated_sources: Vec<String>,
    pub unavailable_sources: Vec<String>,
    pub operating_reviews: OperatingReviewReadiness,
}

impl OperatorAgendaView {
    pub fn push(&mut self, entry: AgendaEntry) {
        if self.entries.len() < MAX_AGENDA_ENTRIES {
            self.entries.push(entry);
        } else if !self.truncated_sources.iter().any(|source| source == "host") {
            self.truncated_sources.push("host".into());
        }
    }

    pub fn sort(&mut self) {
        self.entries.sort_by(|left, right| {
            right
                .urgency
                .cmp(&left.urgency)
                .then_with(|| right.actionability.cmp(&left.actionability))
                .then_with(|| left.source_peer.cmp(&right.source_peer))
                .then_with(|| left.resource_identity.cmp(&right.resource_identity))
        });
    }
}

/// Maps one bounded owner-published Chronos fact into host attention semantics.
/// Unknown phases are not agenda items. This never changes task state or
/// infers authority from priority, dependencies, or agenda presence.
pub fn chronos_attention_entry(
    workstream: &str,
    revision: &str,
    task: &str,
    incarnation: &str,
    phase: &str,
    priority: i64,
    dependency_count: u64,
) -> Option<AgendaEntry> {
    let blocked = match phase {
        "blocked" => true,
        "uncertain" => false,
        _ => return None,
    };
    Some(AgendaEntry {
        source_peer: "chronos".into(),
        source_instance: Some(workstream.into()),
        source_revision: revision.into(),
        resource_identity: format!("{task}#{incarnation}"),
        kind: if blocked {
            AgendaKind::DependencyBlocked
        } else {
            AgendaKind::OutcomeUnknown
        },
        urgency: AgendaUrgency::Blocking,
        actionability: if blocked {
            AgendaActionability::InspectOnly
        } else {
            AgendaActionability::Actionable
        },
        blocker: blocked.then(|| {
            format!("{dependency_count} owner-published prerequisite(s) remain incomplete")
        }),
        title: if blocked {
            format!("Blocked Chronos work · {task} · P{priority}")
        } else {
            format!("Uncertain Chronos task · {task}")
        },
        why_operator: if blocked {
            "the owner reports admitted work cannot become eligible yet"
        } else {
            "the owner fenced progress after an uncertain outcome"
        }
        .into(),
        authority: "agenda presence grants none; Chronos owns task state".into(),
        requested_action: if blocked {
            "inspect the exact dependency graph and align the executable next slice"
        } else {
            "inspect owner evidence and choose a permitted recovery"
        }
        .into(),
        consequence_of_waiting: if blocked {
            "the admitted outcome remains blocked and may drift from current priority"
        } else {
            "the task remains fenced; no blind retry is safe"
        }
        .into(),
        next_owner_action: if blocked {
            "open Chronos Dependencies; follow an incomplete prerequisite"
        } else {
            "open Chronos and inspect the exact task revision"
        }
        .into(),
        route: "navigate.chronos".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(identity: &str, urgency: AgendaUrgency) -> AgendaEntry {
        AgendaEntry {
            source_peer: "chronos".into(),
            source_instance: Some("delivery".into()),
            source_revision: "4".into(),
            resource_identity: identity.into(),
            kind: AgendaKind::OutcomeUnknown,
            urgency,
            actionability: AgendaActionability::Actionable,
            blocker: None,
            title: "Uncertain task".into(),
            why_operator: "owner requires reconciliation".into(),
            authority: "none inferred".into(),
            requested_action: "inspect".into(),
            consequence_of_waiting: "work remains fenced".into(),
            next_owner_action: "open Chronos".into(),
            route: "navigate.chronos".into(),
        }
    }

    #[test]
    fn agenda_orders_blocking_items_and_never_defaults_review_owner() {
        let mut view = OperatorAgendaView::default();
        view.push(entry("later", AgendaUrgency::Attention));
        view.push(entry("first", AgendaUrgency::Blocking));
        view.sort();
        assert_eq!(view.entries[0].resource_identity, "first");
        assert!(!view.operating_reviews.owner_configured);
        assert!(view.operating_reviews.status.contains("Unavailable"));
    }

    #[test]
    fn agenda_is_bounded_and_discloses_truncation() {
        let mut view = OperatorAgendaView::default();
        for index in 0..=MAX_AGENDA_ENTRIES {
            view.push(entry(&index.to_string(), AgendaUrgency::Attention));
        }
        assert_eq!(view.entries.len(), MAX_AGENDA_ENTRIES);
        assert_eq!(view.truncated_sources, ["host"]);
    }

    #[test]
    fn chronos_attention_distinguishes_blocked_from_uncertain() {
        let blocked =
            chronos_attention_entry("delivery", "7", "align", "2", "blocked", 10, 6).unwrap();
        assert_eq!(blocked.kind, AgendaKind::DependencyBlocked);
        assert_eq!(blocked.actionability, AgendaActionability::InspectOnly);
        assert!(blocked.title.contains("P10"));
        assert!(
            blocked
                .blocker
                .as_deref()
                .is_some_and(|value| value.contains('6'))
        );

        let uncertain =
            chronos_attention_entry("delivery", "8", "effect", "3", "uncertain", 0, 0).unwrap();
        assert_eq!(uncertain.kind, AgendaKind::OutcomeUnknown);
        assert_eq!(uncertain.actionability, AgendaActionability::Actionable);
        assert!(uncertain.blocker.is_none());
        assert!(chronos_attention_entry("delivery", "9", "ready", "4", "ready", 0, 0).is_none());
    }
}
