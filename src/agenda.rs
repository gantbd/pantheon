//! Host-owned, read-only projection of owner-native operator attention facts.
//!
//! Entries retain their source identity and revision. Presence in this view is
//! never authority, approval, assignment, or evidence that an action settled.

use serde::{Deserialize, Serialize};

pub const MAX_AGENDA_ENTRIES: usize = 128;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgendaKind {
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgendaEntry {
    pub source_peer: String,
    pub source_instance: Option<String>,
    pub source_revision: String,
    pub resource_identity: String,
    pub kind: AgendaKind,
    pub urgency: AgendaUrgency,
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
                .then_with(|| left.source_peer.cmp(&right.source_peer))
                .then_with(|| left.resource_identity.cmp(&right.resource_identity))
        });
    }
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
}
