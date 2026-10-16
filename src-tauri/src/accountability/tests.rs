use super::resources::*;
use super::routes::*;
use super::types::*;
use crate::case_profile::sample_appeal_profile;

#[test]
fn test_disciplinary_route_contains_mandatory_warning() {
    let profile = sample_appeal_profile();
    let routes = determine_routes_for_issue(&profile, RecordSupportStatus::UnsupportedInRecord, false);

    let acjc_route = routes.iter().find(|r| r.is_disciplinary);
    assert!(acjc_route.is_some(), "Must provide disciplinary route for awareness");
    let route = acjc_route.unwrap();
    assert!(
        route.warning_notice.contains("DOES NOT stay or overturn court orders"),
        "Disciplinary route must warn that it cannot overturn court orders"
    );
    assert!(
        route.warning_notice.contains("DOES NOT toll"),
        "Disciplinary route must warn that appeal deadlines are not tolled"
    );
}

#[test]
fn test_nj_interlocutory_route_applies_lawson_v_dewar() {
    let mut profile = sample_appeal_profile();
    profile.governing_state = "NJ".into();
    let routes = determine_routes_for_issue(&profile, RecordSupportStatus::UnsupportedInRecord, false);

    let interloc_route = routes.iter().find(|r| r.route_kind == RouteKind::ReconsiderationInterlocutory);
    assert!(interloc_route.is_some());
    let r = interloc_route.unwrap();
    assert!(r.rule_citation.contains("4:42-2"));
    assert!(r.governing_standard.contains("Lawson v. Dewar"));
}

#[test]
fn test_list_accountability_resources_filters() {
    let nj_resources = list_accountability_resources(Some("NJ"), Some("judicial"));
    assert!(nj_resources.iter().any(|r| r.id == "nj-acjc-11166"));
    let acjc = nj_resources.iter().find(|r| r.id == "nj-acjc-11166").unwrap();
    assert!(acjc.non_appeal_warning.contains("DOES NOT TOLL"));

    let records_resources = list_accountability_resources(Some("NJ"), Some("records"));
    assert_eq!(records_resources.len(), 1);
    assert_eq!(records_resources[0].id, "nj-court-records-10200");
    assert!(records_resources[0].fee_schedule.contains("5¢"));
}
