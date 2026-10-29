use std::time::Duration;

use serde_json::json;
use tokio::sync::broadcast::error::{RecvError, TryRecvError};

use super::*;

fn at(millis: i64) -> DateTime<Utc> {
    DateTime::from_timestamp_millis(millis).unwrap()
}

fn view() -> Viewpoint {
    Viewpoint {
        panorama_id: "real-panorama".into(),
        position: Position {
            lat: 52.52,
            lng: 13.405,
        },
        heading: 145.2,
        pitch: -5.0,
        zoom: 1.0,
    }
}

fn room(count: usize) -> (Session, Vec<Uuid>) {
    let mut session = Session::new(60, at(1000)).unwrap();
    let players = (0..count)
        .map(|i| session.join(format!("Player {i}"), None).unwrap())
        .collect();
    (session, players)
}

fn ready_all(session: &mut Session) {
    let step_id = session.step_id;
    let players: Vec<_> = session
        .player_order
        .iter()
        .copied()
        .filter(|id| !session.players[id].left)
        .collect();
    for player in players {
        session.ready(player, step_id, true, at(1000)).unwrap();
    }
}

fn searching(count: usize, category_count: usize) -> (Session, Vec<Uuid>, Vec<Uuid>) {
    let (mut session, players) = room(count);
    ready_all(&mut session);
    let categories = (0..category_count)
        .map(|i| {
            session
                .add_category(players[0], format!("Category {i}"))
                .unwrap()
        })
        .collect();
    ready_all(&mut session);
    assert_eq!(session.phase, Phase::Searching);
    (session, players, categories)
}

#[test]
fn readiness_waits_for_every_member_and_rejects_stale_steps() {
    let (mut session, players) = room(2);
    let step = session.step_id;
    session.ready(players[0], step, true, at(1000)).unwrap();
    session.ready(players[0], step, true, at(1000)).unwrap();
    assert_eq!(session.phase, Phase::Lobby);
    session.ready(players[0], step, false, at(1000)).unwrap();
    assert!(!session.players[&players[0]].is_ready);
    ready_all(&mut session);
    assert_eq!(session.phase, Phase::Categories);
    assert!(session.players.values().all(|player| !player.is_ready));
    assert!(matches!(
        session.ready(players[0], step, true, at(1000)),
        Err(SessionError::Conflict(_))
    ));
    assert!(
        session
            .ready(players[0], session.step_id, true, at(1000))
            .is_err()
    );
    assert!(!session.players[&players[0]].is_ready);
}

#[test]
fn category_edits_and_setup_membership_changes_invalidate_agreement() {
    let (mut session, players) = room(2);
    assert!(session.add_category(players[0], "Door".into()).is_err());
    ready_all(&mut session);
    let category = session
        .add_category(players[0], "  A blue door  ".into())
        .unwrap();
    assert_eq!(session.categories[&category].text, "A blue door");
    let step = session.step_id;
    session.ready(players[0], step, true, at(1000)).unwrap();
    // Another member can edit the shared category, as specified by the API.
    session
        .update_category(players[1], category, "A red door".into())
        .unwrap();
    assert_ne!(session.step_id, step);
    assert!(session.players.values().all(|player| !player.is_ready));
    session
        .ready(players[0], session.step_id, true, at(1000))
        .unwrap();
    let joined = session.join("New peer".into(), None).unwrap();
    assert!(session.players.values().all(|player| !player.is_ready));
    let step = session.step_id;
    session.leave(joined, at(1000)).unwrap();
    assert_ne!(session.step_id, step);
    session.delete_category(players[1], category).unwrap();
    assert!(session.categories.is_empty());
    assert!(session.category_order.is_empty());
    assert!(session.delete_category(players[0], category).is_err());
}

#[test]
fn deadline_ends_searching_without_unanimous_readiness() {
    let (mut session, players, categories) = searching(2, 1);
    assert_eq!(session.search_ends_at, Some(61000));
    session
        .ready(players[0], session.step_id, true, at(2000))
        .unwrap();
    assert_eq!(session.phase, Phase::Searching);
    assert!(!session.tick(at(60999)));
    session
        .add_finding(players[0], categories[0], view(), at(60999))
        .unwrap();
    let step = session.step_id;
    assert!(session.tick(at(61000)));
    assert_eq!(session.phase, Phase::Selection);
    assert!(session.players.values().all(|player| !player.is_ready));
    assert_ne!(session.step_id, step);
    assert!(!session.tick(at(62000)));
    assert!(
        session
            .add_finding(players[0], categories[0], view(), at(61000))
            .is_err()
    );
    assert!(
        session
            .add_category(players[0], "Late category".into())
            .is_err()
    );
}

#[test]
fn unanimous_search_readiness_is_reversible_and_opens_selection_once() {
    let (mut session, players, categories) = searching(2, 1);
    let finding = session
        .add_finding(players[0], categories[0], view(), at(2000))
        .unwrap();
    let step = session.step_id;
    session.ready(players[0], step, true, at(3000)).unwrap();
    session.ready(players[0], step, true, at(3000)).unwrap();
    assert_eq!(session.phase, Phase::Searching);
    assert_eq!(session.search_ends_at, Some(61000));
    session.ready(players[0], step, false, at(4000)).unwrap();
    session.ready(players[1], step, true, at(5000)).unwrap();
    assert_eq!(session.phase, Phase::Searching);
    // A player with no findings may still finish searching.
    session.ready(players[0], step, true, at(6000)).unwrap();
    assert_eq!(session.phase, Phase::Selection);
    assert_ne!(session.step_id, step);
    assert!(session.players.values().all(|player| !player.is_ready));
    assert_eq!(
        session.players[&players[0]].selections[&categories[0]],
        finding
    );
    assert!(matches!(
        session.ready(players[0], step, true, at(6000)),
        Err(SessionError::Conflict(_))
    ));
    assert!(
        session
            .add_finding(players[0], categories[0], view(), at(7000))
            .is_err()
    );
    assert!(!session.tick(at(61000)));
    assert_eq!(session.phase, Phase::Selection);
}

#[test]
fn changing_search_evidence_clears_readiness_and_leaving_rechecks_completion() {
    let (mut session, players, categories) = searching(2, 1);
    let step = session.step_id;
    session.ready(players[0], step, true, at(2000)).unwrap();
    let finding = session
        .add_finding(players[0], categories[0], view(), at(3000))
        .unwrap();
    assert!(!session.players[&players[0]].is_ready);
    session.ready(players[0], step, true, at(4000)).unwrap();
    session.delete_finding(players[0], finding).unwrap();
    assert!(!session.players[&players[0]].is_ready);
    session.ready(players[0], step, true, at(5000)).unwrap();
    // An unready disconnected member still counts until they explicitly leave.
    assert!(session.players[&players[1]].connections.is_empty());
    assert_eq!(session.phase, Phase::Searching);
    session.leave(players[1], at(6000)).unwrap();
    assert_eq!(session.phase, Phase::Selection);
    assert!(session.players.values().all(|player| !player.is_ready));
}

#[test]
fn findings_are_distinct_and_selected_candidates_are_repaired_in_save_order() {
    let (mut session, players, categories) = searching(2, 2);
    // Equal timestamps must still preserve the order in which findings were saved.
    let first = session
        .add_finding(players[0], categories[0], view(), at(2000))
        .unwrap();
    let second = session
        .add_finding(players[0], categories[0], view(), at(2000))
        .unwrap();
    let third = session
        .add_finding(players[0], categories[0], view(), at(2000))
        .unwrap();
    assert_ne!(first, second);
    assert_eq!(
        session.players[&players[0]].findings[&categories[0]].len(),
        3
    );
    assert_eq!(
        session.players[&players[0]].selections[&categories[0]],
        first
    );
    assert!(
        !session.players[&players[0]]
            .selections
            .contains_key(&categories[1])
    );
    assert!(session.delete_finding(players[1], first).is_err());
    assert!(
        session
            .select_finding(players[1], categories[0], first)
            .is_err()
    );
    assert!(
        session
            .select_finding(players[0], categories[1], first)
            .is_err()
    );
    session.tick(at(61000));
    session
        .ready(players[0], session.step_id, true, at(62000))
        .unwrap();
    session
        .select_finding(players[0], categories[0], third)
        .unwrap();
    assert!(!session.players[&players[0]].is_ready);
    session
        .ready(players[0], session.step_id, true, at(62000))
        .unwrap();
    session
        .select_finding(players[0], categories[0], third)
        .unwrap();
    assert!(session.players[&players[0]].is_ready); // Retry is idempotent.
    session.delete_finding(players[0], third).unwrap();
    assert!(!session.players[&players[0]].is_ready);
    assert_eq!(
        session.players[&players[0]].selections[&categories[0]],
        first
    );
    session.delete_finding(players[0], first).unwrap();
    assert_eq!(
        session.players[&players[0]].selections[&categories[0]],
        second
    );
    session.delete_finding(players[0], second).unwrap();
    assert!(
        !session.players[&players[0]]
            .selections
            .contains_key(&categories[0])
    );
}

#[test]
fn review_order_is_category_then_player_and_keeps_departed_players_submissions() {
    let (mut session, players, categories) = searching(3, 2);
    let mut ids = HashMap::new();
    // Deliberately save in the opposite order to the review order.
    for category in categories.iter().rev() {
        for player in players.iter().rev() {
            if (*category, *player) != (categories[1], players[1]) {
                ids.insert(
                    (*category, *player),
                    session
                        .add_finding(*player, *category, view(), at(2000))
                        .unwrap(),
                );
            }
        }
    }
    session.leave(players[2], at(3000)).unwrap();
    session.tick(at(61000));
    ready_all(&mut session);
    assert_eq!(session.phase, Phase::Review);
    assert_eq!(
        session
            .review_queue
            .iter()
            .map(|finding| finding.id)
            .collect::<Vec<_>>(),
        vec![
            ids[&(categories[0], players[0])],
            ids[&(categories[0], players[1])],
            ids[&(categories[0], players[2])],
            ids[&(categories[1], players[0])],
            ids[&(categories[1], players[2])],
        ]
    );
    let current = session.review_queue[0].id;
    assert!(session.delete_finding(players[0], current).is_err());
    assert!(
        session
            .select_finding(players[0], categories[0], current)
            .is_err()
    );
}

#[test]
fn snapshots_only_expose_own_candidates_current_review_and_revealed_results() {
    let (mut session, players, categories) = searching(3, 1);
    session
        .add_finding(players[0], categories[0], view(), at(2000))
        .unwrap();
    let mut secret = view();
    secret.panorama_id = "future-private-panorama".into();
    session
        .add_finding(players[1], categories[0], secret, at(2000))
        .unwrap();
    let before = session.snapshot(players[0], at(2000)).unwrap();
    assert_eq!(before.own_findings.len(), 1);
    assert!(before.review.is_none());
    assert!(session.snapshot(Uuid::new_v4(), at(2000)).is_err());
    session.tick(at(61000));
    ready_all(&mut session);
    let item = session.review_queue[0].id;
    session
        .vote(players[1], item, Vote::Reject, at(62000))
        .unwrap();
    let state = session.snapshot(players[0], at(62000)).unwrap();
    let encoded = serde_json::to_value(&state).unwrap();
    assert_eq!(encoded["phase"], "review");
    assert_eq!(encoded["meId"], players[0].to_string());
    assert_eq!(encoded["durationSeconds"], 60);
    assert_eq!(encoded["serverTime"], 62000);
    assert_eq!(encoded["searchEndsAt"], 61000);
    assert_eq!(encoded["review"]["ownVote"], serde_json::Value::Null);
    assert_eq!(encoded["review"]["result"], serde_json::Value::Null);
    assert_eq!(encoded["review"]["votesCast"], 1);
    assert_eq!(encoded["review"]["eligibleVoters"], 2);
    assert!(encoded["results"].as_array().unwrap().is_empty());
    assert!(encoded.get("votes").is_none());
    assert!(encoded.get("reviewQueue").is_none());
    assert!(encoded.get("connections").is_none());
    assert!(!encoded.to_string().contains("future-private-panorama"));
    assert_eq!(
        session
            .snapshot(players[1], at(62000))
            .unwrap()
            .review
            .unwrap()
            .own_vote,
        Some(Vote::Reject)
    );
}

#[test]
fn votes_count_as_ready_and_the_last_vote_finishes_without_a_reveal_step() {
    let (mut session, players, categories) = searching(3, 1);
    let item = session
        .add_finding(players[0], categories[0], view(), at(2000))
        .unwrap();
    session.tick(at(61000));
    ready_all(&mut session);
    assert!(matches!(
        session.vote(players[0], item, Vote::Accept, at(62000)),
        Err(SessionError::Forbidden(_))
    ));
    assert!(
        session
            .vote(players[1], Uuid::new_v4(), Vote::Accept, at(62000))
            .is_err()
    );
    assert!(
        session
            .ready(players[1], session.step_id, true, at(62000))
            .is_err()
    );
    session
        .ready(players[0], session.step_id, true, at(62000))
        .unwrap();
    session
        .vote(players[1], item, Vote::Accept, at(62000))
        .unwrap();
    assert_eq!(session.phase, Phase::Review);
    assert!(session.results.is_empty());
    assert!(session.players[&players[1]].is_ready);
    session
        .vote(players[1], item, Vote::Accept, at(62000))
        .unwrap();
    assert!(session.players[&players[1]].is_ready);
    session
        .vote(players[1], item, Vote::Reject, at(62000))
        .unwrap();
    assert!(session.players[&players[1]].is_ready);
    assert!(
        session
            .ready(players[1], session.step_id, false, at(62000))
            .is_err()
    );
    assert!(session.players[&players[0]].is_ready);
    session
        .vote(players[2], item, Vote::Accept, at(62000))
        .unwrap();
    assert_eq!(session.phase, Phase::Finished);
    assert_eq!(session.results[0].accepts, 1);
    assert_eq!(session.results[0].rejects, 1);
    assert!(!session.results[0].accepted);
    assert_eq!(session.players[&players[0]].score, 0);
    assert!(
        session
            .vote(players[1], item, Vote::Accept, at(62000))
            .is_err()
    );
    assert_eq!(session.results.len(), 1);
}

#[test]
fn complete_round_awards_each_point_once_and_finishes_after_last_result() {
    let (mut session, players, categories) = searching(3, 2);
    for category in &categories {
        for player in &players {
            session
                .add_finding(*player, *category, view(), at(2000))
                .unwrap();
        }
    }
    session.tick(at(61000));
    ready_all(&mut session);
    for index in 0..6 {
        let finding = session.review_queue[index].clone();
        let voters: Vec<_> = players
            .iter()
            .copied()
            .filter(|id| *id != finding.player_id)
            .collect();
        session
            .vote(voters[0], finding.id, Vote::Accept, at(62000))
            .unwrap();
        session
            .vote(
                voters[1],
                finding.id,
                if index % 2 == 0 {
                    Vote::Accept
                } else {
                    Vote::Reject
                },
                at(62000),
            )
            .unwrap();
        let voting_step = session.step_id;
        // Votes alone wait for the owner, who is the only player needing a ready click.
        assert_eq!(session.review_index, index);
        session
            .ready(finding.player_id, voting_step, true, at(62000))
            .unwrap();
        assert_ne!(session.step_id, voting_step);
        assert_eq!(session.review_index, index + 1);
        assert!(session.players.values().all(|player| !player.is_ready));
        assert!(session.votes.is_empty());
        assert_eq!(session.results.len(), index + 1);
        assert_eq!(session.results[index].accepted, index % 2 == 0);
        assert!(
            session
                .ready(players[0], voting_step, true, at(62000))
                .is_err()
        );
        assert_eq!(session.results.len(), index + 1);
    }
    assert_eq!(session.phase, Phase::Finished);
    assert!(
        session
            .snapshot(players[0], at(62000))
            .unwrap()
            .review
            .is_none()
    );
    assert_eq!(session.results.len(), 6);
    assert!(players.iter().all(|id| session.players[id].score == 1));
    assert!(
        session
            .ready(players[0], session.step_id, true, at(62000))
            .is_err()
    );
}

#[test]
fn zero_votes_fail_and_empty_submission_rounds_finish() {
    let (mut session, players, categories) = searching(1, 1);
    session
        .add_finding(players[0], categories[0], view(), at(2000))
        .unwrap();
    session.tick(at(61000));
    ready_all(&mut session);
    ready_all(&mut session);
    assert!(!session.results[0].accepted);
    assert_eq!(session.results[0].accepts, 0);
    assert_eq!(session.results[0].rejects, 0);
    assert_eq!(session.phase, Phase::Finished);

    let (mut empty, _, _) = searching(2, 1);
    empty.tick(at(61000));
    ready_all(&mut empty);
    assert_eq!(empty.phase, Phase::Finished);
    assert!(empty.results.is_empty());
}

#[test]
fn leaving_removes_private_vote_and_rechecks_review_progression() {
    let (mut session, players, categories) = searching(3, 1);
    let item = session
        .add_finding(players[0], categories[0], view(), at(2000))
        .unwrap();
    session.tick(at(61000));
    ready_all(&mut session);
    session
        .vote(players[1], item, Vote::Accept, at(62000))
        .unwrap();
    session
        .ready(players[0], session.step_id, true, at(62000))
        .unwrap();
    assert_eq!(session.phase, Phase::Review);
    assert!(session.results.is_empty());
    session.leave(players[2], at(62000)).unwrap();
    assert_eq!(session.phase, Phase::Finished);
    assert!(session.results[0].accepted);
    assert_eq!(session.results[0].rejects, 0);
    assert_eq!(session.players[&players[0]].score, 1);
    assert!(!session.votes.contains_key(&players[2]));
    assert!(session.snapshot(players[2], at(62000)).is_err());
    assert!(session.connect(players[2]).is_err());
    assert!(session.join("Returned".into(), Some(players[2])).is_err());
    session.leave(players[2], at(62000)).unwrap();
    assert_eq!(session.players[&players[0]].score, 1);
}

#[test]
fn a_departed_owner_does_not_block_votes_and_departing_voters_lose_their_vote() {
    let (mut session, players, categories) = searching(4, 1);
    let item = session
        .add_finding(players[0], categories[0], view(), at(2000))
        .unwrap();
    session.tick(at(61000));
    ready_all(&mut session);
    session
        .vote(players[1], item, Vote::Reject, at(62000))
        .unwrap();
    session.leave(players[1], at(62000)).unwrap();
    assert!(!session.votes.contains_key(&players[1]));
    session.leave(players[0], at(62000)).unwrap();
    session
        .vote(players[2], item, Vote::Accept, at(62000))
        .unwrap();
    assert_eq!(session.phase, Phase::Review);
    session
        .vote(players[3], item, Vote::Accept, at(62000))
        .unwrap();
    assert_eq!(session.phase, Phase::Finished);
    assert_eq!(session.results[0].accepts, 2);
    assert_eq!(session.results[0].rejects, 0);
    assert_eq!(session.players[&players[0]].score, 1);
}

#[test]
fn connections_do_not_change_membership_or_readiness_and_inactivity_is_stable() {
    let (mut session, players) = room(2);
    let old = session.connect(players[0]).unwrap();
    let new = session.connect(players[0]).unwrap();
    assert!(session.inactive_since.is_none());
    session
        .ready(players[0], session.step_id, true, at(1000))
        .unwrap();
    let step = session.step_id;
    session.disconnect(players[0], old, at(2000)).unwrap();
    session.disconnect(players[0], old, at(3000)).unwrap();
    assert!(session.snapshot(players[0], at(3000)).unwrap().players[0].connected);
    assert!(session.players[&players[0]].is_ready);
    assert_eq!(session.step_id, step);
    session.disconnect(players[0], new, at(4000)).unwrap();
    session.disconnect(players[0], old, at(5000)).unwrap();
    assert_eq!(session.inactive_since, Some(at(4000)));
    assert_eq!(session.phase, Phase::Lobby);
    // The disconnected member still has to agree before progression.
    session.ready(players[0], step, false, at(5000)).unwrap();
    session.ready(players[1], step, true, at(5000)).unwrap();
    assert_eq!(session.phase, Phase::Lobby);
    assert_eq!(
        session.join("Player 0".into(), Some(players[0])).unwrap(),
        players[0]
    );
    assert_eq!(session.step_id, step);
    session.ready(players[0], step, true, at(5000)).unwrap();
    assert_eq!(session.phase, Phase::Categories);
}

#[test]
fn input_limits_are_enforced_without_accepting_invalid_members_or_viewpoints() {
    assert!(Session::new(0, at(1000)).is_err());
    assert!(Session::new(u64::MAX, at(1000)).is_err());
    let (mut session, players) = room(MAX_PLAYERS);
    assert!(session.join("Overflow".into(), None).is_err());
    session.leave(players[1], at(1000)).unwrap();
    session.join("Replacement".into(), None).unwrap();
    assert!(session.join(" ".into(), None).is_err());
    assert!(session.join("a".repeat(25), None).is_err());
    assert!(
        session
            .join("Unknown".into(), Some(Uuid::new_v4()))
            .is_err()
    );
    ready_all(&mut session);
    assert!(session.add_category(players[0], " ".into()).is_err());
    assert!(session.add_category(players[0], "a".repeat(101)).is_err());
    for index in 0..MAX_CATEGORIES {
        session
            .add_category(players[0], format!("Category {index}"))
            .unwrap();
    }
    assert!(session.add_category(players[0], "Overflow".into()).is_err());
    ready_all(&mut session);
    let category = session.category_order[0];
    for _ in 0..MAX_FINDINGS_PER_CATEGORY {
        session
            .add_finding(players[0], category, view(), at(2000))
            .unwrap();
    }
    assert!(
        session
            .add_finding(players[0], category, view(), at(2000))
            .is_err()
    );
    assert!(
        session
            .add_finding(players[0], Uuid::new_v4(), view(), at(2000))
            .is_err()
    );
    assert!(
        session
            .add_finding(Uuid::new_v4(), category, view(), at(2000))
            .is_err()
    );
    assert!(session.join("Late newcomer".into(), None).is_err());

    let invalid = [
        Viewpoint {
            panorama_id: " ".into(),
            ..view()
        },
        Viewpoint {
            position: Position {
                lat: 91.0,
                lng: 0.0,
            },
            ..view()
        },
        Viewpoint {
            position: Position {
                lat: 0.0,
                lng: f64::NAN,
            },
            ..view()
        },
        Viewpoint {
            heading: f64::INFINITY,
            ..view()
        },
        Viewpoint {
            heading: -1.0,
            ..view()
        },
        Viewpoint {
            pitch: 91.0,
            ..view()
        },
        Viewpoint {
            zoom: -1.0,
            ..view()
        },
    ];
    for viewpoint in invalid {
        assert!(matches!(
            validate_viewpoint(&viewpoint),
            Err(SessionError::InvalidInput(_))
        ));
    }
}

#[test]
fn wire_inputs_reject_demo_fields_and_use_camel_case() {
    let mut encoded = serde_json::to_value(view()).unwrap();
    assert!(encoded.get("panoramaId").is_some());
    encoded["demoScene"] = json!(1);
    assert!(serde_json::from_value::<Viewpoint>(encoded).is_err());
    assert_eq!(
        serde_json::from_value::<Vote>(json!("accept")).unwrap(),
        Vote::Accept
    );
    assert!(serde_json::from_value::<Vote>(json!("maybe")).is_err());
    let info: SessionStartInfo =
        serde_json::from_value(json!({ "name": "Alex", "durationSeconds": 300 })).unwrap();
    assert_eq!(info.duration_seconds, 300);
    assert_eq!(
        serde_json::to_value(SessionEvent::StateChanged { revision: 7 }).unwrap(),
        json!({
            "type": "state-changed", "revision": 7,
        })
    );
}

async fn manager_ready_all(manager: &SessionManager, id: &str, players: &[Uuid]) {
    let state = manager.get_session(id, players[0]).await.unwrap();
    for player in players {
        manager
            .ready(id, *player, state.step_id, true)
            .await
            .unwrap();
    }
}

fn drain(connection: &mut SessionConnection) {
    while connection.updates.try_recv().is_ok() {}
}

#[tokio::test]
async fn rooms_with_identical_creator_names_are_independent_and_codes_ignore_case() {
    let manager = SessionManager::new();
    let first = manager.start_session("  Alex  ".into(), 60).await.unwrap();
    let second = manager.start_session("Alex".into(), 60).await.unwrap();
    assert_ne!(first.session_id, second.session_id);
    assert_eq!(first.session_id.len(), 6);
    assert_eq!(first.session_id, first.session_id.to_ascii_uppercase());
    let state = manager
        .clone()
        .get_session(&first.session_id.to_ascii_lowercase(), first.player_id)
        .await
        .unwrap();
    assert_eq!(state.players[0].name, "Alex");
    assert!(!state.players[0].connected);
    assert!(matches!(
        manager
            .get_session(&second.session_id, first.player_id)
            .await,
        Err(SessionError::Forbidden(_))
    ));
    assert!(matches!(
        manager.get_session("MISSING", first.player_id).await,
        Err(SessionError::NotFound(_))
    ));
    assert!(manager.start_session(" ".into(), 60).await.is_err());
    assert!(manager.start_session("Alex".into(), 59).await.is_err());
}

#[tokio::test]
async fn concurrent_ready_requests_advance_once_without_losing_updates() {
    let manager = SessionManager::new();
    let created = manager.start_session("Alex".into(), 60).await.unwrap();
    let id = &created.session_id;
    let other = manager.join_session(id, "Mika".into(), None).await.unwrap();
    let before = manager.get_session(id, created.player_id).await.unwrap();
    let (first, second) = tokio::join!(
        manager.ready(id, created.player_id, before.step_id, true),
        manager.ready(id, other, before.step_id, true),
    );
    first.unwrap();
    second.unwrap();
    let after = manager.get_session(id, created.player_id).await.unwrap();
    assert_eq!(after.phase, Phase::Categories);
    assert_eq!(after.revision, before.revision + 2);
    assert!(after.players.iter().all(|player| !player.ready));
    let category = manager
        .add_category(id, created.player_id, "Door".into())
        .await
        .unwrap();
    manager
        .update_category(id, other, category, "Blue door".into())
        .await
        .unwrap();
    manager.delete_category(id, other, category).await.unwrap();
    assert!(
        manager
            .get_session(id, other)
            .await
            .unwrap()
            .categories
            .is_empty()
    );
}

#[tokio::test(start_paused = true)]
async fn concurrent_search_readiness_publishes_completion_and_the_old_timer_is_a_noop() {
    let manager = SessionManager::new();
    let created = manager.start_session("Alex".into(), 60).await.unwrap();
    let id = &created.session_id;
    let other = manager.join_session(id, "Mika".into(), None).await.unwrap();
    let players = [created.player_id, other];
    let mut connection = manager.connect(id, created.player_id).await.unwrap();
    manager_ready_all(&manager, id, &players).await;
    manager
        .add_category(id, created.player_id, "Door".into())
        .await
        .unwrap();
    manager_ready_all(&manager, id, &players).await;
    let before = manager.get_session(id, created.player_id).await.unwrap();
    assert_eq!(before.phase, Phase::Searching);
    drain(&mut connection);
    let (first, second) = tokio::join!(
        manager.ready(id, created.player_id, before.step_id, true),
        manager.ready(id, other, before.step_id, true),
    );
    first.unwrap();
    second.unwrap();
    assert_eq!(
        connection.updates.try_recv().unwrap(),
        SessionEvent::StateChanged {
            revision: before.revision + 1
        }
    );
    assert_eq!(
        connection.updates.try_recv().unwrap(),
        SessionEvent::StateChanged {
            revision: before.revision + 2
        }
    );
    let after = manager.get_session(id, created.player_id).await.unwrap();
    assert_eq!(after.phase, Phase::Selection);
    assert_ne!(after.step_id, before.step_id);
    assert!(after.players.iter().all(|player| !player.ready));
    assert!(
        manager
            .ready(id, other, before.step_id, true)
            .await
            .is_err()
    );
    tokio::time::advance(Duration::from_secs(61)).await;
    tokio::task::yield_now().await;
    assert!(matches!(
        connection.updates.try_recv(),
        Err(TryRecvError::Empty)
    ));
    let unchanged = manager.get_session(id, created.player_id).await.unwrap();
    assert_eq!(unchanged.phase, Phase::Selection);
    assert_eq!(unchanged.step_id, after.step_id);
    assert_eq!(unchanged.revision, after.revision);
}

#[tokio::test(start_paused = true)]
async fn deadline_fires_and_notifies_without_any_requests() {
    let manager = SessionManager::new();
    let created = manager.start_session("Alex".into(), 60).await.unwrap();
    let id = &created.session_id;
    let mut connection = manager.connect(id, created.player_id).await.unwrap();
    manager_ready_all(&manager, id, &[created.player_id]).await;
    manager
        .add_category(id, created.player_id, "Door".into())
        .await
        .unwrap();
    manager_ready_all(&manager, id, &[created.player_id]).await;
    let before = manager.get_session(id, created.player_id).await.unwrap();
    assert_eq!(before.phase, Phase::Searching);
    drain(&mut connection);
    tokio::time::advance(Duration::from_secs(59)).await;
    tokio::task::yield_now().await;
    assert!(matches!(
        connection.updates.try_recv(),
        Err(TryRecvError::Empty)
    ));
    tokio::time::advance(Duration::from_secs(2)).await;
    // Await the notification before making another manager request.
    let event = tokio::time::timeout(Duration::from_secs(1), connection.updates.recv())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        event,
        SessionEvent::StateChanged {
            revision: before.revision + 1
        }
    );
    let after = manager.get_session(id, created.player_id).await.unwrap();
    assert_eq!(after.phase, Phase::Selection);
    assert_ne!(after.step_id, before.step_id);
    assert!(
        manager
            .add_finding(id, created.player_id, after.categories[0].id, view())
            .await
            .is_err()
    );
    manager_ready_all(&manager, id, &[created.player_id]).await;
    assert_eq!(
        manager
            .get_session(id, created.player_id)
            .await
            .unwrap()
            .phase,
        Phase::Finished
    );
}

#[tokio::test]
async fn socket_cleanup_is_connection_specific_and_notifications_stay_within_the_room() {
    let manager = SessionManager::new();
    let created = manager.start_session("Alex".into(), 60).await.unwrap();
    let id = &created.session_id;
    let other_room = manager.start_session("Jules".into(), 60).await.unwrap();
    let mut unrelated = manager
        .connect(&other_room.session_id, other_room.player_id)
        .await
        .unwrap();
    let mut old = manager.connect(id, created.player_id).await.unwrap();
    let mut new = manager.connect(id, created.player_id).await.unwrap();
    drain(&mut old);
    drain(&mut new);
    drain(&mut unrelated);
    manager
        .disconnect(id, created.player_id, old.connection_id)
        .await
        .unwrap();
    manager
        .disconnect(id, created.player_id, old.connection_id)
        .await
        .unwrap();
    assert!(
        manager
            .get_session(id, created.player_id)
            .await
            .unwrap()
            .players[0]
            .connected
    );
    assert!(new.updates.try_recv().is_ok());
    assert!(matches!(
        unrelated.updates.try_recv(),
        Err(TryRecvError::Empty)
    ));
    manager
        .disconnect(id, created.player_id, new.connection_id)
        .await
        .unwrap();
    assert!(
        !manager
            .get_session(id, created.player_id)
            .await
            .unwrap()
            .players[0]
            .connected
    );
    manager.leave_session(id, created.player_id).await.unwrap();
    manager
        .disconnect(id, created.player_id, new.connection_id)
        .await
        .unwrap();
    assert!(manager.get_session(id, created.player_id).await.is_err());
}

#[tokio::test]
async fn lagging_subscribers_can_resynchronize_and_invalid_actions_do_not_publish() {
    let manager = SessionManager::new();
    let created = manager.start_session("Alex".into(), 60).await.unwrap();
    let id = &created.session_id;
    manager.join_session(id, "Mika".into(), None).await.unwrap();
    let mut connection = manager.connect(id, created.player_id).await.unwrap();
    drain(&mut connection);
    let before = manager.get_session(id, created.player_id).await.unwrap();
    assert!(
        manager
            .add_category(id, created.player_id, "Wrong phase".into())
            .await
            .is_err()
    );
    assert_eq!(
        manager
            .get_session(id, created.player_id)
            .await
            .unwrap()
            .revision,
        before.revision
    );
    assert!(matches!(
        connection.updates.try_recv(),
        Err(TryRecvError::Empty)
    ));
    for index in 0..40 {
        manager
            .ready(id, created.player_id, before.step_id, index % 2 == 0)
            .await
            .unwrap();
    }
    assert!(matches!(
        connection.updates.recv().await,
        Err(RecvError::Lagged(_))
    ));
    let latest = manager.get_session(id, created.player_id).await.unwrap();
    assert_eq!(latest.revision, before.revision + 40);
    assert!(!latest.players[0].ready);
}

#[tokio::test(start_paused = true)]
async fn manager_methods_support_the_full_api_action_sequence() {
    let manager = SessionManager::new();
    let created = manager.start_session("Alex".into(), 60).await.unwrap();
    let id = &created.session_id;
    let alex = created.player_id;
    let mika = manager.join_session(id, "Mika".into(), None).await.unwrap();
    manager_ready_all(&manager, id, &[alex, mika]).await;
    let category = manager.add_category(id, alex, "Door".into()).await.unwrap();
    manager_ready_all(&manager, id, &[alex, mika]).await;
    let first = manager
        .add_finding(id, alex, category, view())
        .await
        .unwrap();
    let second = manager
        .add_finding(id, alex, category, view())
        .await
        .unwrap();
    manager
        .select_finding(id, alex, category, second)
        .await
        .unwrap();
    manager.delete_finding(id, alex, first).await.unwrap();
    assert!(manager.delete_finding(id, mika, second).await.is_err());
    assert_eq!(
        manager
            .join_session(id, "Alex".into(), Some(alex))
            .await
            .unwrap(),
        alex
    );
    tokio::time::advance(Duration::from_secs(61)).await;
    tokio::task::yield_now().await;
    manager_ready_all(&manager, id, &[alex, mika]).await;
    let state = manager.get_session(id, mika).await.unwrap();
    assert_eq!(state.phase, Phase::Review);
    assert_eq!(state.review.unwrap().id, second);
    assert!(state.own_findings.is_empty());
    manager.vote(id, mika, second, Vote::Accept).await.unwrap();
    let waiting = manager.get_session(id, alex).await.unwrap();
    assert!(waiting.players[1].ready);
    manager
        .ready(id, alex, waiting.step_id, true)
        .await
        .unwrap();
    let finished = manager.get_session(id, alex).await.unwrap();
    assert!(finished.results[0].accepted);
    assert_eq!(finished.players[0].score, 1);
    assert_eq!(
        manager.get_session(id, alex).await.unwrap().phase,
        Phase::Finished
    );
    manager.leave_session(id, mika).await.unwrap();
    let finished = manager.get_session(id, alex).await.unwrap();
    assert!(finished.players[1].left);
    assert_eq!(finished.results.len(), 1);
}
