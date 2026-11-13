use super::*;

fn plan(header: u64) -> crosshair::Plan {
    crosshair::Plan {
        header,
        parts: [crosshair::empty_part(); crosshair::PARTS],
    }
}

/// What the layer holds when it has exactly `plan`.
fn sent_exactly(plan: &crosshair::Plan) -> Sent {
    (Some(plan.header), plan.parts.map(Some))
}

#[test]
fn a_crosshair_the_layer_already_has_sends_nothing() {
    let plan = plan(5);
    assert_eq!(next_step(&plan, &sent_exactly(&plan)), None);
}

#[test]
fn a_new_crosshair_with_no_parts_sends_only_its_header() {
    assert_eq!(next_step(&plan(5), &NOTHING_SENT), Some(None));
}

#[test]
fn a_changed_header_is_sent_again() {
    let old = plan(5);
    assert_eq!(next_step(&plan(6), &sent_exactly(&old)), Some(None));
}

#[test]
fn parts_go_before_the_header_so_a_new_crosshair_appears_whole() {
    let mut new = plan(6);
    new.parts[1] = 111;
    new.parts[3] = 333;
    let old = plan(5);
    assert_eq!(
        next_step(&new, &sent_exactly(&old)),
        Some(Some(1)),
        "the first changed part, though the header changed too"
    );
    let mut halfway = sent_exactly(&old);
    halfway.1[1] = Some(111);
    assert_eq!(
        next_step(&new, &halfway),
        Some(Some(3)),
        "then the next one"
    );
    halfway.1[3] = Some(333);
    assert_eq!(
        next_step(&new, &halfway),
        Some(None),
        "the header comes last"
    );
    halfway.0 = Some(6);
    assert_eq!(next_step(&new, &halfway), None, "and then it is all sent");
}

#[test]
fn a_part_that_has_become_empty_is_cleared() {
    let mut drawn = plan(5);
    drawn.parts[2] = 222;
    let now = plan(5);
    assert_eq!(next_step(&now, &sent_exactly(&drawn)), Some(Some(2)));
}

#[test]
fn a_part_never_sent_counts_as_empty() {
    // The HUD document starts with every part empty, so an unsent empty part needs no patch.
    let mut sent = sent_exactly(&plan(5));
    sent.1[0] = None;
    assert_eq!(next_step(&plan(5), &sent), None);
}

#[test]
fn hidden_hunger_and_crosshair_are_composed() {
    assert!(hidden_elements(false, false).is_empty());
    assert_eq!(
        hidden_elements(true, false),
        vec![hud::VanillaElement::Crosshair]
    );
    assert_eq!(
        hidden_elements(false, true),
        vec![hud::VanillaElement::Hunger]
    );
    assert_eq!(
        hidden_elements(true, true),
        vec![hud::VanillaElement::Crosshair, hud::VanillaElement::Hunger]
    );
}
