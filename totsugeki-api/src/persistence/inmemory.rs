//! In-memory database

use super::Service;
use crate::{
    critical::Error as CriticalError,
    parsing::{
        parse_bracket, parse_discord_channel_id, parse_discord_guild_id, parse_discord_user_id,
        parse_match_result, parse_player, parse_players, parse_service,
    },
    persistence::{DBAccessor, Error},
    resource::Error as ResourceError,
    ApiServiceId, ApiServiceUser,
};
use serenity::model::id::{ChannelId, GuildId, UserId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use totsugeki::{
    bracket::{ActiveBrackets, Bracket, CreateRequest, Id as BracketId, POSTResult, Raw},
    join::POSTResponse,
    matches::{Error as MatchError, Id as MatchId, NextMatchGETResponseRaw, ReportResultPOST},
    organiser::Id as OrganiserId,
    organiser::Organiser,
    player::{Id as PlayerId, Participants, Player, GET as PlayersGET},
    DiscussionChannelId,
};
use tracing::info;

/// In-memory database
#[derive(Default)]
pub struct InMemoryDBAccessor {
    /// Lock over in-memory database
    db: Arc<RwLock<InMemoryDatabase>>,
}

/// Link internal id of discord channel to global id
type DiscordInternalChannels = HashMap<ChannelId, DiscussionChannelId>;

/// Link internal discord guild id to organiser id
type DiscordInternalGuilds = HashMap<GuildId, OrganiserId>;

/// Link discord users to Totsugeki users
type DiscordInternalUsers = HashMap<UserId, PlayerId>;

/// Active bracket in discussion channel
type DiscussionChannelActiveBrackets = HashMap<DiscussionChannelId, BracketId>;

/// Organiser of bracket
type BracketOrganiser = HashMap<BracketId, OrganiserId>;

/// In memory database
#[derive(Default)]
pub struct InMemoryDatabase {
    // TODO find out if you need internal mapping per discord Api service
    // my guess is you don't need
    /// All registered brackets
    brackets: HashMap<BracketId, Bracket>,
    /// All registered organisers
    organisers: HashMap<OrganiserId, Organiser>,
    /// Alll registered api services users
    api_services: HashMap<ApiServiceId, (String, String)>,
    /// Maps to discussion channels id
    discord_internal_channel: DiscordInternalChannels,
    /// Maps to organiser id
    discord_internal_guilds: DiscordInternalGuilds,
    /// All registered discord users who are Totsugeki users
    discord_internal_users: DiscordInternalUsers,
    /// Get active bracket in discussion channel
    discussion_channel_active_brackets: DiscussionChannelActiveBrackets,
    /// All organisers of brackets
    bracket_organiser: BracketOrganiser,
}

impl DBAccessor for InMemoryDBAccessor {
    fn clean<'a, 'b>(&'a self) -> Result<(), Error<'b>> {
        let mut db = self.db.write().expect("database"); // FIXME bubble up error
        db.brackets = HashMap::new();
        db.organisers = HashMap::new();
        Ok(())
    }

    fn close_bracket<'a, 'b, 'c>(
        &'a self,
        internal_channel_id: &'b str,
        service: &'b str,
    ) -> Result<BracketId, Error<'c>> {
        let mut db = self.db.write().expect("database"); // FIXME bubble up error
        let (active_bracket_id, _, _) = find_active_bracket_id(&db, internal_channel_id, service)?;
        let active_bracket = db.brackets.get(&active_bracket_id);
        match active_bracket {
            Some(b) => {
                let updated_bracket = b.clone().close();
                db.brackets
                    .insert(updated_bracket.get_id(), updated_bracket);

                Ok(active_bracket_id)
            }
            None => {
                return Err(CriticalError::Corrupted(format!(
                    "No bracket found for id: \"{active_bracket_id}\""
                ))
                .into())
            }
        }
    }

    fn create_bracket<'a, 'b, 'c>(&'a self, r: CreateRequest<'b>) -> Result<POSTResult, Error<'c>> {
        let organiser_name = r.organiser_name;
        let organiser_internal_id = r.organiser_internal_id;
        let internal_channel_id = r.internal_channel_id;
        let service = parse_service(r.service_type_id)?;
        let b = parse_bracket(r)?;

        let mut db = self.db.write().expect("database"); // FIXME bubble up error

        match service {
            Service::Discord => {
                let internal_channel_id = parse_discord_channel_id(internal_channel_id)?;
                let (discussion_channel_id, is_new_channel) =
                    if let Some(m) = db.discord_internal_channel.get(&internal_channel_id) {
                        (*m, false)
                    } else {
                        (DiscussionChannelId::new_v4(), true)
                    };
                let internal_organiser_id = parse_discord_guild_id(organiser_internal_id)?;
                let organiser_id = db.discord_internal_guilds.get(&internal_organiser_id);

                let updated_organiser = add_bracket_to_organiser_active_brackets(
                    &db,
                    organiser_id,
                    organiser_name.to_string(),
                    b.get_id(),
                    discussion_channel_id,
                );

                // save service specific data
                let organiser_id = updated_organiser.get_id();
                db.discord_internal_channel
                    .insert(internal_channel_id, discussion_channel_id);
                db.discord_internal_guilds
                    .insert(internal_organiser_id, organiser_id);

                // save all other data
                db.organisers.insert(organiser_id, updated_organiser);
                db.discussion_channel_active_brackets
                    .insert(discussion_channel_id, b.get_id());
                db.bracket_organiser.insert(b.get_id(), organiser_id);
                db.brackets.insert(b.get_id(), b.clone());

                info!("Created new bracket {b}");
                if is_new_channel {
                    info!("Registered new discussion channel ({service}): {discussion_channel_id}");
                }

                Ok(POSTResult {
                    bracket_id: b.get_id(),
                    organiser_id,
                    discussion_channel_id,
                })
            }
        }

        // NOTE: to synchronise id of organiser across services, you would need to a sync endpoint otherwise
        // it will create many uuid for the same "real" organiser
        // example: Fancy bar uses discord bot and telegram bot to create bracket. If he creates smth with discord, he
        // would need to sync his telegram user with his discord account before creating a bracket with telegram under
        // the same user account.
        // Idea to implement that is the following:
        // do an operation with social_media_1 (using social media 1) that creates an account
        // do an operation with social_media_2 (using social media 2) that creates an account
        // as user of social_media_1 (...), list your global_id
        // as user of social_media_2, request your global id to be linked to global_id
        // as user of social_media_1, list sync request and accept request
        // now, user of social_media_2 can use social_media_2 to create and list under social_media_1
    }

    fn create_organiser<'a, 'b, 'c>(&'a self, organiser_name: &'b str) -> Result<(), Error<'c>> {
        let mut db = self.db.write().expect("database"); // FIXME bubble up error
        let organiser = Organiser::new(organiser_name.to_string(), None);
        db.organisers.insert(organiser.get_id(), organiser);
        Ok(())
    }

    fn disqualify_player<'a, 'b, 'c>(
        &'a self,
        internal_channel_id: &'b str,
        service: &'b str,
        player_id: &'b str,
    ) -> Result<BracketId, Error<'c>> {
        let mut db = self.db.write().expect("database"); // FIXME bubble up error
        let (active_bracket_id, _, _) = find_active_bracket_id(&db, internal_channel_id, service)?;
        match db.brackets.get(&active_bracket_id) {
            Some(b) => {
                let updated_bracket = b.clone().disqualify_participant(parse_player(player_id)?)?;
                db.brackets
                    .insert(updated_bracket.get_id(), updated_bracket);

                Ok(active_bracket_id)
            }
            None => {
                return Err(CriticalError::Corrupted(format!(
                    "No bracket found for id: \"{active_bracket_id}\""
                ))
                .into())
            }
        }
    }

    fn find_brackets<'a, 'b, 'c>(
        &'a self,
        bracket_name: &'b str,
        _offset: i64,
    ) -> Result<Vec<Raw>, Error<'c>> {
        let db = self.db.read().expect("database");
        let mut brackets = vec![];
        for b in &db.brackets {
            if b.1.get_name() == bracket_name {
                brackets.push(b.1.clone().into());
            }
        }
        Ok(brackets)
    }

    fn find_next_match<'a, 'b, 'c>(
        &'a self,
        player_internal_id: &'b str,
        channel_internal_id: &'b str,
        service: &'b str,
    ) -> Result<NextMatchGETResponseRaw, Error<'c>> {
        let db = self.db.read().expect("database"); // FIXME bubble up error
        let service = parse_service(service)?;
        let (bracket, player_id) =
            get_bracket_info(&db, service, channel_internal_id, player_internal_id)?;

        let (opponent, match_id, opponent_name) = bracket.next_opponent(player_id)?;
        info!(
            "\"{player_id}\" searched for his next opponent (\"{opponent}\") in bracket \"{}\"",
            bracket.get_id()
        );

        Ok(NextMatchGETResponseRaw {
            opponent: opponent.to_string(),
            match_id,
            bracket_id: bracket.get_id(),
            player_name: opponent_name,
        })
    }

    fn find_organisers<'a, 'b, 'c>(
        &'a self,
        organiser_name: &'b str,
        _offset: i64,
    ) -> Result<Vec<Organiser>, Error<'c>> {
        Ok(self
            .db
            .read()
            .expect("database") // FIXME bubble up error
            .organisers
            .clone()
            .into_iter()
            .filter(|o| o.1.get_organiser_name() == organiser_name)
            .map(|o| o.1)
            .collect::<Vec<Organiser>>())
    }

    fn get_bracket<'a, 'b, 'c>(&'a self, bracket_id: BracketId) -> Result<Raw, Error<'c>> {
        let db = self.db.read().expect("database"); // FIXME bubble up error
        let bracket = match db.brackets.get(&bracket_id) {
            Some(b) => b,
            None => return Err(ResourceError::UnknownResource(bracket_id).into()),
        };
        Ok(bracket.clone().into())
    }

    fn init(&self) -> Result<(), Error> {
        Ok(())
    }

    fn join_bracket<'a, 'b, 'c>(
        &'a self,
        player_internal_id: &'b str,
        player_name: &'b str,
        channel_internal_id: &'b str,
        service_type_id: &'b str,
    ) -> Result<totsugeki::join::POSTResponse, Error<'c>> {
        let mut db = self.db.write().expect("database"); // FIXME bubble up error
        let (bracket_id, _, service) =
            find_active_bracket_id(&db, channel_internal_id, service_type_id)?;

        match service {
            Service::Discord => {
                let player_internal_id = parse_discord_user_id(player_internal_id)?;
                let (player_id, is_new_player) =
                    if let Some(id) = db.discord_internal_users.get(&player_internal_id) {
                        (*id, false)
                    } else {
                        (PlayerId::new_v4(), true)
                    };

                db.discord_internal_users
                    .insert(player_internal_id, player_id);

                let organiser_id = db.bracket_organiser.get(&bracket_id).ok_or_else(|| {
                    CriticalError::Corrupted(format!(
                        "No organiser is responsible for bracket {bracket_id}"
                    ))
                })?;

                let body = POSTResponse {
                    player_id,
                    bracket_id,
                    organiser_id: *organiser_id,
                };

                let b = db.brackets.get(&bracket_id).ok_or_else(|| {
                    CriticalError::Corrupted(format!("No bracket found for id: \"{bracket_id}\""))
                })?;
                let b = b.clone();
                let id = b.get_id();
                let updated_bracket = b.join(Player {
                    id: player_id,
                    name: player_name.to_string(),
                })?;
                db.brackets.insert(id, updated_bracket);
                if is_new_player {
                    info!("New player {player_id} registered");
                }
                info!("Player {player_id} joined bracket {id}");

                Ok(body)
            }
        }
    }

    fn list_brackets<'a, 'b>(&'a self, _offset: i64) -> Result<Vec<Raw>, Error<'b>> {
        let db = self.db.read().expect("database"); // FIXME bubble up error
        let mut brackets = vec![];
        for b in &db.brackets {
            brackets.push(b.1.clone().into());
        }
        Ok(brackets)
    }

    fn list_organisers<'a, 'b>(&'a self, _offset: i64) -> Result<Vec<Organiser>, Error<'b>> {
        Ok(self
            .db
            .read()
            .expect("database") // FIXME bubble up error
            .organisers
            .clone()
            .into_iter()
            .map(|o| o.1)
            .collect())
    }

    fn list_players<'a, 'b>(
        &'a self,
        r: &PlayersGET,
    ) -> Result<(BracketId, Participants), Error<'b>> {
        let db = self.db.read().expect("database"); // FIXME bubble up error
        let (active_bracket_id, _, _) = find_active_bracket_id(
            &db,
            r.internal_discussion_channel_id.as_str(),
            r.service.as_str(),
        )?;
        let players = match db.brackets.get(&active_bracket_id) {
            Some(b) => b.get_participants(),
            None => return Err(ResourceError::UnknownResource(active_bracket_id).into()),
        };

        Ok((active_bracket_id, players))
    }

    fn list_service_api_user<'a, 'b>(
        &'a self,
        _offset: i64,
    ) -> Result<Vec<crate::ApiServiceUser>, Error<'b>> {
        let db = self.db.read().expect("database"); // FIXME bubble up error
        let api_service_users = db
            .api_services
            .clone()
            .into_iter()
            .map(|u| ApiServiceUser {
                id: u.0,
                name: u.1 .0,
                description: u.1 .1,
            })
            .collect();
        Ok(api_service_users)
    }

    fn quit_bracket<'a, 'b, 'c>(
        &'a self,
        internal_channel_id: &'b str,
        service: &'b str,
        internal_player_id: &'b str,
    ) -> Result<BracketId, Error<'c>> {
        let mut db = self.db.write().expect("database"); // FIXME bubble up error
        let (active_bracket_id, _, service) =
            find_active_bracket_id(&db, internal_channel_id, service)?;
        let player_id = match service {
            Service::Discord => {
                let user_id = parse_discord_user_id(internal_player_id)?;
                match db.discord_internal_users.get(&user_id) {
                    Some(player_id) => *player_id,
                    None => return Err(ResourceError::UnknownPlayer.into()),
                }
            }
        };
        match db.brackets.get(&active_bracket_id) {
            Some(b) => {
                let updated_bracket = b.clone().remove_participant(player_id)?;
                db.brackets
                    .insert(updated_bracket.get_id(), updated_bracket);

                Ok(active_bracket_id)
            }
            None => {
                return Err(CriticalError::Corrupted(format!(
                    "No bracket found for id: \"{active_bracket_id}\""
                ))
                .into())
            }
        }
    }

    fn register_service_api_user<'a, 'b, 'c>(
        &'a self,
        service_name: &'b str,
        service_description: &'b str,
    ) -> Result<ApiServiceId, Error<'c>> {
        let mut db = self.db.write().expect("database"); // FIXME bubble up error
        let id = ApiServiceId::new_v4();
        db.api_services.insert(
            id,
            (service_name.to_string(), service_description.to_string()),
        );
        Ok(id)
    }

    fn remove_player<'a, 'b, 'c>(
        &'a self,
        internal_channel_id: &'b str,
        service: &'b str,
        player_id: &'b str,
    ) -> Result<BracketId, Error<'c>> {
        let mut db = self.db.write().expect("database"); // FIXME bubble up error
        let (active_bracket_id, _, _) = find_active_bracket_id(&db, internal_channel_id, service)?;
        match db.brackets.get(&active_bracket_id) {
            Some(b) => {
                let updated_bracket = b.clone().remove_participant(parse_player(player_id)?)?;
                db.brackets
                    .insert(updated_bracket.get_id(), updated_bracket);

                Ok(active_bracket_id)
            }
            None => {
                return Err(CriticalError::Corrupted(format!(
                    "No bracket found for id: \"{active_bracket_id}\""
                ))
                .into())
            }
        }
    }

    fn report_result<'a, 'b, 'c>(
        &'a self,
        player_internal_id: &'b str,
        channel_internal_id: &'b str,
        service: &'b str,
        result: &'b str,
    ) -> Result<ReportResultPOST, Error<'c>> {
        let result = parse_match_result(result)?;
        let mut db = self.db.write().expect("database"); // FIXME not good
        let service = parse_service(service)?;
        let (bracket, player_id) =
            get_bracket_info(&db, service, channel_internal_id, player_internal_id)?;

        let (bracket, affected_match_id) = bracket.report_result(player_id, result)?;
        let bracket_with_reported_result = bracket.clone();
        if bracket.is_validating_matches_automatically() {
            match bracket.validate_match_result(affected_match_id) {
                Ok(b) => {
                    db.brackets.insert(b.get_id(), b);
                    return Ok(ReportResultPOST {
                        affected_match_id,
                        message: "Result reported and match validated".into(),
                    });
                }
                Err(e) => match e {
                    totsugeki::bracket::Error::Match(
                        MatchError::PlayersReportedDifferentMatchOutcome(_),
                    ) => {
                        // Even though match can't be validated, we still
                        // update the bracket but add a warning
                        db.brackets.insert(
                            bracket_with_reported_result.get_id(),
                            bracket_with_reported_result,
                        );
                        return Ok(ReportResultPOST {
                            affected_match_id,
                            message: "Result reported".into(),
                        });
                    }
                    _ => {
                        return Err(e.into());
                    }
                },
            }
        }
        db.brackets.insert(bracket.get_id(), bracket);

        Ok(ReportResultPOST {
            affected_match_id,
            message: "Result reported".into(),
        })
    }

    fn seed_bracket<'a, 'b, 'c>(
        &'a self,
        internal_channel_id: &'b str,
        service: &'b str,
        players: Vec<String>,
    ) -> Result<BracketId, Error<'c>> {
        let players = parse_players(&players)?;

        let mut db = self.db.write().expect("database"); // FIXME bubble up error
        let (active_bracket_id, _, _) = find_active_bracket_id(&db, internal_channel_id, service)?;
        let bracket = db.brackets.get(&active_bracket_id).ok_or_else(|| {
            CriticalError::Corrupted(format!("No bracket found for id: \"{active_bracket_id}\""))
        })?;
        let updated_bracket = bracket.clone().update_seeding(&players)?;

        db.brackets
            .insert(updated_bracket.get_id(), updated_bracket);

        Ok(active_bracket_id)
    }

    fn start_bracket<'a, 'b, 'c>(
        &'a self,
        internal_channel_id: &'b str,
        service: &'b str,
    ) -> Result<BracketId, Error<'c>> {
        let mut db = self.db.write().expect("database"); // FIXME bubble up error
        let (active_bracket_id, _, _) = find_active_bracket_id(&db, internal_channel_id, service)?;
        match db.brackets.get(&active_bracket_id) {
            Some(b) => {
                let updated_bracket = b.clone().start();
                db.brackets
                    .insert(updated_bracket.get_id(), updated_bracket);

                Ok(active_bracket_id)
            }
            None => {
                return Err(CriticalError::Corrupted(format!(
                    "No bracket found for id: \"{active_bracket_id}\""
                ))
                .into())
            }
        }
    }

    fn validate_result<'a, 'b>(&'a self, match_id: MatchId) -> Result<(), Error<'b>> {
        let mut db = self.db.write().expect("database");

        let bracket = db
            .brackets
            .iter()
            .find(|b| (*b.1).get_matches().iter().any(|m| m.get_id() == match_id));
        let bracket = match bracket {
            Some(b) => b.1.clone().validate_match_result(match_id)?,
            None => return Err(ResourceError::UnknownResource(match_id).into()),
        };

        let bracket_id = bracket.get_id();
        db.brackets.insert(bracket.get_id(), bracket);
        info!(
            "Match \"{match_id}\" validated in bracket \"{}\"",
            bracket_id
        );
        Ok(())
    }

    fn forfeit<'a, 'b, 'c>(
        &'a self,
        internal_channel_id: &'b str,
        service: &'b str,
        internal_player_id: &'b str,
    ) -> Result<BracketId, Error<'c>> {
        let mut db = self.db.write().expect("database"); // FIXME bubble up error
        let (active_bracket_id, _, service) =
            find_active_bracket_id(&db, internal_channel_id, service)?;
        let player_id = match service {
            Service::Discord => {
                let user_id = parse_discord_user_id(internal_player_id)?;
                match db.discord_internal_users.get(&user_id) {
                    Some(player_id) => *player_id,
                    None => return Err(ResourceError::UnknownPlayer.into()),
                }
            }
        };
        match db.brackets.get(&active_bracket_id) {
            Some(b) => {
                let updated_bracket = b.clone().disqualify_participant(player_id)?;
                db.brackets
                    .insert(updated_bracket.get_id(), updated_bracket);

                Ok(active_bracket_id)
            }
            None => {
                return Err(CriticalError::Corrupted(format!(
                    "No bracket found for id: \"{active_bracket_id}\""
                ))
                .into())
            }
        }
    }
}

/// Return active bracket id in discussion channel, parsed discussion channel id and service
///
/// # Errors
/// Returns an error if there is no active channel
fn find_active_bracket_id<'a, 'b, 'c>(
    db: &'a InMemoryDatabase,
    internal_channel_id: &'b str,
    service: &'b str,
) -> Result<(BracketId, DiscussionChannelId, Service), Error<'c>> {
    let service = parse_service(service)?;
    match service {
        Service::Discord => {
            let channel_id = parse_discord_channel_id(internal_channel_id)?;
            let channel_id = match db.discord_internal_channel.get(&channel_id) {
                Some(id) => *id,
                None => return Err(ResourceError::UnknownDiscussionChannel(service).into()),
            };
            let active_bracket = match db.discussion_channel_active_brackets.get(&channel_id) {
                Some(id) => *id,
                None => {
                    return Err(
                        ResourceError::UnknownActiveBracketForDiscussionChannel(channel_id).into(),
                    )
                }
            };

            Ok((active_bracket, channel_id, service))
        }
    }
}

/// Get player and bracket from discussion channel
fn get_bracket_info<'a, 'b, 'c>(
    db: &'a InMemoryDatabase,
    service: Service,
    channel_internal_id: &'b str,
    player_internal_id: &'b str,
) -> Result<(Bracket, PlayerId), Error<'c>> {
    match service {
        Service::Discord => {
            let channel_id = parse_discord_channel_id(channel_internal_id)?;
            let channel_id = match db.discord_internal_channel.get(&channel_id) {
                Some(id) => id,
                None => return Err(ResourceError::UnknownDiscussionChannel(service).into()),
            };

            let active_bracket_id = match db.discussion_channel_active_brackets.get(channel_id) {
                Some(id) => id,
                None => {
                    return Err(ResourceError::UnknownActiveBracketForDiscussionChannel(
                        *channel_id,
                    )
                    .into())
                }
            };

            let bracket = match db.brackets.iter().find(|b| b.0 == active_bracket_id) {
                Some(b) => b.1.clone(),
                None => return Err(ResourceError::UnknownResource(*active_bracket_id).into()),
            };

            let player_internal_id = parse_discord_user_id(player_internal_id)?;
            match db.discord_internal_users.get(&player_internal_id) {
                Some(player_id) => Ok((bracket, *player_id)),
                None => Err(ResourceError::UnknownPlayer.into()),
            }
        }
    }
}

/// Add bracket to organiser's active bracket and return updated organiser
fn add_bracket_to_organiser_active_brackets(
    db: &InMemoryDatabase,
    organiser_id: Option<&OrganiserId>,
    organiser_name: String,
    bracket_id: BracketId,
    discussion_channel_id: DiscussionChannelId,
) -> Organiser {
    match organiser_id {
        Some(id) => match db.organisers.get(id) {
            Some(o) => o
                .clone()
                .add_active_bracket(discussion_channel_id, bracket_id),
            None => {
                let mut active_bracket = ActiveBrackets::default();
                active_bracket.insert(discussion_channel_id, bracket_id);
                Organiser::new(organiser_name, Some(active_bracket))
            }
        },
        None => {
            let mut active_bracket = ActiveBrackets::default();
            active_bracket.insert(discussion_channel_id, bracket_id);
            Organiser::new(organiser_name, Some(active_bracket))
        }
    }
}
