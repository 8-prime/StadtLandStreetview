use std::time::Duration;

use tokio::time::Instant;
use uuid::Uuid;

use super::{Phase, Position, ProximityBand, ProximityState, Result, Session, SessionError};

const PRESENCE_LIFETIME: Duration = Duration::from_secs(15);
const NEARBY_METERS: f64 = 5_000.0;
const CLOSE_METERS: f64 = 250.0;

#[derive(Debug)]
pub(super) struct Presence {
    pub(super) player_id: Uuid,
    position: Position,
    updated_at: Instant,
}

impl Session {
    pub(super) fn report_position(
        &mut self,
        player_id: Uuid,
        connection_id: Uuid,
        step_id: Uuid,
        position: Option<Position>,
        now: Instant,
    ) -> Result<()> {
        let player = self.active_player(player_id)?;
        if !player.connections.contains(&connection_id) {
            return Err(SessionError::Forbidden(
                "This connection is no longer active.",
            ));
        }
        self.require_phase(&[Phase::Searching])?;
        if !self.settings.proximity_enabled || self.step_id != step_id {
            return Err(SessionError::Conflict(
                "The neighbourhood radio is not active.",
            ));
        }
        if let Some(position) = position {
            if !position.lat.is_finite()
                || !position.lng.is_finite()
                || !(-90.0..=90.0).contains(&position.lat)
                || !(-180.0..=180.0).contains(&position.lng)
            {
                return Err(SessionError::InvalidInput("Invalid Street View position."));
            }
            self.presence.insert(
                connection_id,
                Presence {
                    player_id,
                    position,
                    updated_at: now,
                },
            );
        } else {
            self.presence.remove(&connection_id);
        }
        Ok(())
    }

    fn current_position(&self, player_id: Uuid, now: Instant) -> Option<Position> {
        self.presence
            .values()
            .filter(|presence| {
                presence.player_id == player_id
                    && now.duration_since(presence.updated_at) < PRESENCE_LIFETIME
            })
            .max_by_key(|presence| presence.updated_at)
            .map(|presence| presence.position)
    }

    pub(super) fn proximity_for(&self, player_id: Uuid, now: Instant) -> Option<ProximityState> {
        if !self.settings.proximity_enabled || self.phase != Phase::Searching {
            return None;
        }
        let band = if let Some(position) = self.current_position(player_id, now) {
            let distance = self
                .players
                .values()
                .filter(|player| {
                    !player.left && player.id != player_id && !player.connections.is_empty()
                })
                .filter_map(|player| self.current_position(player.id, now))
                .map(|other| distance_meters(position, other))
                .fold(f64::INFINITY, f64::min);
            if distance <= CLOSE_METERS {
                ProximityBand::Close
            } else if distance <= NEARBY_METERS {
                ProximityBand::Nearby
            } else {
                ProximityBand::Quiet
            }
        } else {
            ProximityBand::Inactive
        };
        Some(ProximityState {
            step_id: self.step_id,
            band,
        })
    }
}

fn distance_meters(a: Position, b: Position) -> f64 {
    let lat = ((b.lat - a.lat).to_radians() / 2.0).sin();
    let lng = ((b.lng - a.lng).to_radians() / 2.0).sin();
    let haversine = lat * lat + a.lat.to_radians().cos() * b.lat.to_radians().cos() * lng * lng;
    2.0 * 6_371_000.0 * haversine.clamp(0.0, 1.0).sqrt().asin()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn radio_room() -> (Session, [Uuid; 2], [Uuid; 2]) {
        let mut session = Session::new(60, Utc::now()).unwrap();
        let players = [
            session.join("Alex".into(), None).unwrap(),
            session.join("Mika".into(), None).unwrap(),
        ];
        session.settings.proximity_enabled = true;
        session.new_step(Phase::Searching);
        let connections = [
            session.connect(players[0]).unwrap(),
            session.connect(players[1]).unwrap(),
        ];
        (session, players, connections)
    }

    fn north(meters: f64) -> Position {
        Position {
            lat: (meters / 6_371_000.0).to_degrees(),
            lng: 0.0,
        }
    }

    #[test]
    fn distance_handles_date_line_poles_and_antipodes() {
        assert!(
            (distance_meters(
                Position {
                    lat: 0.0,
                    lng: 179.99
                },
                Position {
                    lat: 0.0,
                    lng: -179.99
                }
            ) - 2223.9)
                .abs()
                < 1.0
        );
        assert!(
            distance_meters(
                Position {
                    lat: 90.0,
                    lng: 0.0
                },
                Position {
                    lat: 90.0,
                    lng: 180.0
                }
            ) < 0.01
        );
        assert!(
            (distance_meters(
                north(0.0),
                Position {
                    lat: 0.0,
                    lng: 180.0
                }
            ) - std::f64::consts::PI * 6_371_000.0)
                .abs()
                < 1.0
        );
    }

    #[test]
    fn bands_use_five_kilometers_and_do_not_expose_coordinates_or_mutate_the_game() {
        let (mut session, players, connections) = radio_room();
        let now = Instant::now();
        assert_eq!(
            session.proximity_for(players[0], now).unwrap().band,
            ProximityBand::Inactive
        );
        session
            .report_position(
                players[0],
                connections[0],
                session.step_id,
                Some(north(0.0)),
                now,
            )
            .unwrap();
        assert_eq!(
            session.proximity_for(players[0], now).unwrap().band,
            ProximityBand::Quiet
        );
        for (meters, expected) in [
            (5001.0, ProximityBand::Quiet),
            (4999.0, ProximityBand::Nearby),
            (251.0, ProximityBand::Nearby),
            (249.0, ProximityBand::Close),
        ] {
            session
                .report_position(
                    players[1],
                    connections[1],
                    session.step_id,
                    Some(north(meters)),
                    now,
                )
                .unwrap();
            assert_eq!(
                session.proximity_for(players[0], now).unwrap().band,
                expected
            );
            assert_eq!(
                session.proximity_for(players[1], now).unwrap().band,
                expected
            );
        }
        assert_eq!(session.revision, 0);
        assert!(session.players.values().all(|player| !player.is_ready));
        let snapshot =
            serde_json::to_string(&session.snapshot(players[0], Utc::now()).unwrap()).unwrap();
        assert!(!snapshot.contains("position"));
        let state = serde_json::to_value(session.proximity_for(players[0], now).unwrap()).unwrap();
        assert_eq!(state.as_object().unwrap().len(), 2);
    }

    #[test]
    fn presence_expires_clears_and_is_specific_to_a_connection_and_search_step() {
        let (mut session, players, connections) = radio_room();
        let now = Instant::now();
        let step = session.step_id;
        for i in 0..2 {
            session
                .report_position(players[i], connections[i], step, Some(north(0.0)), now)
                .unwrap();
        }
        let replacement = session.connect(players[1]).unwrap();
        session
            .report_position(
                players[1],
                replacement,
                step,
                Some(north(1000.0)),
                now + Duration::from_secs(1),
            )
            .unwrap();
        session
            .disconnect(players[1], connections[1], Utc::now())
            .unwrap();
        assert_eq!(
            session
                .proximity_for(players[0], now + Duration::from_secs(1))
                .unwrap()
                .band,
            ProximityBand::Nearby
        );
        session
            .report_position(
                players[1],
                replacement,
                step,
                None,
                now + Duration::from_secs(2),
            )
            .unwrap();
        assert_eq!(
            session
                .proximity_for(players[0], now + Duration::from_secs(2))
                .unwrap()
                .band,
            ProximityBand::Quiet
        );
        session
            .report_position(
                players[1],
                replacement,
                step,
                Some(north(0.0)),
                now + Duration::from_secs(2),
            )
            .unwrap();
        session
            .report_position(
                players[0],
                connections[0],
                step,
                Some(north(0.0)),
                now + Duration::from_secs(3),
            )
            .unwrap();
        assert_eq!(
            session
                .proximity_for(players[0], now + Duration::from_secs(17))
                .unwrap()
                .band,
            ProximityBand::Quiet
        );
        assert_eq!(
            session
                .proximity_for(players[0], now + Duration::from_secs(18))
                .unwrap()
                .band,
            ProximityBand::Inactive
        );
        session.new_step(Phase::Selection);
        assert!(session.presence.is_empty());
        assert!(session.proximity_for(players[0], now).is_none());
    }

    #[test]
    fn disabled_stale_foreign_and_invalid_reports_are_rejected() {
        let (mut session, players, connections) = radio_room();
        let now = Instant::now();
        let step = session.step_id;
        assert!(
            session
                .report_position(players[0], connections[1], step, Some(north(0.0)), now)
                .is_err()
        );
        assert!(
            session
                .report_position(
                    players[0],
                    connections[0],
                    Uuid::new_v4(),
                    Some(north(0.0)),
                    now
                )
                .is_err()
        );
        for position in [
            Position {
                lat: 91.0,
                lng: 0.0,
            },
            Position {
                lat: 0.0,
                lng: f64::NAN,
            },
            Position {
                lat: 0.0,
                lng: 181.0,
            },
        ] {
            assert!(
                session
                    .report_position(players[0], connections[0], step, Some(position), now)
                    .is_err()
            );
        }
        session.settings.proximity_enabled = false;
        assert!(
            session
                .report_position(players[0], connections[0], step, Some(north(0.0)), now)
                .is_err()
        );
        assert!(session.proximity_for(players[0], now).is_none());
        assert!(session.presence.is_empty());
    }
}
