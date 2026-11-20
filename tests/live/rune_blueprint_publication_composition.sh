#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd -P)
workspace=$(CDPATH= cd -- "$root/.." && pwd -P)
scratch=$(mktemp -d /tmp/pantheon-rune-composition.XXXXXX)
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

machine="$root/design/runes/pantheon-rune-blueprint-publication.machine.rune"
generated="$root/design/generated/pantheon_rune_blueprint_publication.rs"
scry="$workspace/rune/bin/scry"
smith="$workspace/rune/src/rust/target/debug/smith"

"$scry" "$machine" >"$scratch/composition.ir"
"$smith" compatibility "$machine" --target rust --scry "$scry" \
  >"$scratch/compatibility.json"
jq -e '
  .status == "complete" and
  .unsupported == [] and
  (.preserved | index("invoked-machine-evaluation")) != null and
  (.preserved | index("transition-precondition-evaluation")) != null
' "$scratch/compatibility.json" >/dev/null

"$smith" compile "$machine" --target rust --scry "$scry" \
  >"$scratch/generated.rs"
cmp "$generated" "$scratch/generated.rs"

cat >"$scratch/harness.rs" <<'RUST'
include!("generated.rs");

#[derive(Clone, Copy)]
struct Evaluator { verdict: PantheonRuneBlueprintPublicationInvocationVerdict }
impl PantheonRuneBlueprintPublicationEvaluator for Evaluator {
    fn evaluate(&mut self, _: &PantheonRuneBlueprintPublicationObligation) -> bool { true }
    fn settle_invocation(
        &mut self,
        invocation: &PantheonRuneBlueprintPublicationInvocation,
    ) -> PantheonRuneBlueprintPublicationInvocationVerdict {
        assert_eq!(invocation.child, "pantheon-rune-packet-publication");
        self.verdict
    }
}
struct Chooser;
impl PantheonRuneBlueprintPublicationChooser for Chooser {
    fn choose(
        &mut self,
        _: PantheonRuneBlueprintPublicationState,
        _: &[PantheonRuneBlueprintPublicationCandidate],
    ) -> Option<usize> { None }
}
struct Observer;
impl PantheonRuneBlueprintPublicationObserver for Observer {
    fn observe(&mut self, _: &PantheonRuneBlueprintPublicationObservation) {}
}

fn advance(
    verdict: PantheonRuneBlueprintPublicationInvocationVerdict,
) -> Result<
    PantheonRuneBlueprintPublicationAdvanced,
    PantheonRuneBlueprintPublicationRefusal,
> {
    PantheonRuneBlueprintPublicationState::PublicationEligible.advance(
        &mut Evaluator { verdict },
        &mut Chooser,
        &mut Observer,
    )
}

fn main() {
    assert!(matches!(
        advance(PantheonRuneBlueprintPublicationInvocationVerdict::Unknown),
        Err(PantheonRuneBlueprintPublicationRefusal::InvokedMachineUnknown { .. })
    ));
    assert!(matches!(
        advance(PantheonRuneBlueprintPublicationInvocationVerdict::Refused),
        Err(PantheonRuneBlueprintPublicationRefusal::InvokedMachineRefused { .. })
    ));
    let advanced = advance(
        PantheonRuneBlueprintPublicationInvocationVerdict::Succeeded,
    ).expect("exact child success must permit parent advancement");
    assert_eq!(
        advanced.state,
        PantheonRuneBlueprintPublicationState::DurablePublished,
    );
    assert_eq!(
        advanced.signal,
        PantheonRuneBlueprintPublicationSignal::BlueprintPublished,
    );
}
RUST
cp "$generated" "$scratch/generated.rs"
rustc --edition=2021 "$scratch/harness.rs" -o "$scratch/harness"
"$scratch/harness"

echo 'PASS: canonical composition Rune -> Scry -> Smith Rust; exact child refused/unknown absorb and success alone advances'
