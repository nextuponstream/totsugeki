//! In-memory database
use crate::{
    persistence::{DBAccessor, Error},
    ApiServiceId, ApiServiceUser,
};
use serenity::model::id::{ChannelId, GuildId, UserId};
use serenity::model::misc::{ChannelIdParseError, UserIdParseError};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use totsugeki::{
    bracket::{ActiveBrackets, Bracket, CreateRequest, Id as BracketId, POSTResult, Raw},
    join::POSTResponseBody,
    matches::ReportedResult,
    matches::{Id as MatchId, NextMatchGETResponseRaw},
    organiser::Id as OrganiserId,
    organiser::Organiser,
    player::{Id as PlayerId, Player, Players, GET as PlayersGET},
    DiscussionChannelId,
};
use tracing::{debug, info};

use super::{ParseServiceInternalIdError, Service};

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

    fn create_bracket<'a, 'b, 'c>(&'a self, r: CreateRequest<'b>) -> Result<POSTResult, Error<'c>> {
        let organiser_name = r.organiser_name;
        let organiser_internal_id = r.organiser_internal_id;
        let internal_channel_id = r.internal_channel_id;
        let service = r.service_type_id.parse::<Service>()?;
        let b = Bracket::try_from(r)?;

        let mut db = self.db.write().expect("database"); // FIXME bubble up error

        match service {
            Service::Discord => {
                let internal_channel_id = match internal_channel_id.parse::<ChannelId>() {
                    Ok(v) => v,
                    Err(e) => {
                        let msg =
                            format!("could not parse channel id: {}\n{}", internal_channel_id, e);
                        return Err(Error::Parsing(msg));
                    }
                };
                let (discussion_channel_id, is_new_channel) =
                    if let Some(m) = db.discord_internal_channel.get(&internal_channel_id) {
                        (*m, false)
                    } else {
                        (DiscussionChannelId::new_v4(), true)
                    };
                let id = parse_dicord_id(organiser_internal_id)?;
                let internal_organiser_id = GuildId::from(id);
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

    fn init(&self) -> Result<(), Error> {
        Ok(())
    }

    fn join_bracket<'a, 'b, 'c>(
        &'a self,
        player_internal_id: &'b str,
        player_name: &'b str,
        channel_internal_id: &'b str,
        service_type_id: &'b str,
    ) -> Result<totsugeki::join::POSTResponseBody, Error<'c>> {
        let mut db = self.db.write().expect("database"); // FIXME bubble up error
        let (_, discussion_channel_id, service) =
            find_active_bracket_id(&db, channel_internal_id, service_type_id)?;

        let mut player_id = PlayerId::new_v4();

        match service {
            Service::Discord => {
                let player_internal_id = player_internal_id.parse::<UserId>()?;

                if let Some(id) = db.discord_internal_users.get(&player_internal_id) {
                    player_id = *id;
                }

                db.discord_internal_users
                    .insert(player_internal_id, player_id);
                let bracket_id = db
                    .discussion_channel_active_brackets
                    .get(&discussion_channel_id)
                    .ok_or_else(|| {
                        Error::Denied(
                            "There is no active bracket in this discussion channel".to_string(),
                        )
                    })?;

                let organiser_id = db.bracket_organiser.get(bracket_id).ok_or_else(|| {
                    Error::Corrupted(format!(
                        "No organiser is responsible for bracket {bracket_id}"
                    ))
                })?;

                let body = POSTResponseBody {
                    player_id,
                    bracket_id: *bracket_id,
                    organiser_id: *organiser_id,
                };

                let b = db.brackets.get(bracket_id).ok_or_else(|| {
                    Error::Corrupted(format!("No bracket found for id: \"{bracket_id}\""))
                })?;
                let b = b.clone();
                let id = b.get_id();
                let updated_bracket = b.join(Player {
                    id: player_id,
                    name: player_name.to_string(),
                })?;
                db.brackets.insert(id, updated_bracket);
                info!("New player joined: {player_id}");

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

    fn get_bracket<'a, 'b, 'c>(&'a self, bracket_id: BracketId) -> Result<Raw, Error<'c>> {
        let db = self.db.read().expect("database"); // FIXME bubble up error
        let bracket = match db.brackets.get(&bracket_id) {
            Some(b) => b,
            None => return Err(Error::UnregisteredBracket(bracket_id)),
        };
        Ok(bracket.clone().into())
    }

    fn find_next_match<'a, 'b, 'c>(
        &'a self,
        player_internal_id: &'b str,
        channel_internal_id: &'b str,
        service_type_id: &'b str,
    ) -> Result<NextMatchGETResponseRaw, Error<'c>> {
        let db = self.db.read().expect("database"); // FIXME bubble up error
        let service_type = service_type_id.parse::<Service>()?;
        let (bracket, player_id) =
            get_bracket_info(&db, service_type, channel_internal_id, player_internal_id)?;

        let (opponent, match_id, opponent_name) = bracket.next_opponent(player_id)?;
        info!(
            "\"{player_id}\" searched for his next opponent (\"{opponent}\") in bracket \"{}\"",
            bracket.get_id()
        );

        let m = bracket.get_matches();
        let relevant_match = m.iter().find(|m| m.get_id() == match_id).expect("match");
        debug!("{match_id}: {relevant_match}");

        Ok(NextMatchGETResponseRaw {
            opponent: opponent.to_string(),
            match_id,
            bracket_id: bracket.get_id(),
            player_name: opponent_name,
        })
    }

    fn report_result<'a, 'b, 'c>(
        &'a self,
        player_internal_id: &'b str,
        channel_internal_id: &'b str,
        service_type_id: &'b str,
        result: &'b str,
    ) -> Result<MatchId, Error<'c>> {
        let result = result.parse::<ReportedResult>()?;
        let mut db = self.db.write().expect("database"); // FIXME not good

        let service_type = service_type_id.parse::<Service>()?;
        let (bracket, player_id) =
            get_bracket_info(&db, service_type, channel_internal_id, player_internal_id)?;

        let (updated_bracket, affected_match_id) = bracket.report_result(player_id, result)?;
        db.brackets
            .insert(updated_bracket.get_id(), updated_bracket);

        Ok(affected_match_id)
    }

    fn validate_result<'a, 'b>(&'a self, match_id: MatchId) -> Result<(), Error<'b>> {
        let mut db = self.db.write().expect("database");

        let bracket_to_update = db
            .brackets
            .iter_mut()
            .find(|b| (*b.1).get_matches().iter().any(|m| m.get_id() == match_id));

        let updated_bracket = match bracket_to_update {
            Some(b) => b.1.clone().validate_match_result(match_id)?,
            None => return Err(Error::UnknownMatch(match_id)),
        };

        let bracket_id = updated_bracket.get_id();
        db.brackets
            .insert(updated_bracket.get_id(), updated_bracket);
        info!(
            "Match \"{match_id}\" validated in bracket \"{}\"",
            bracket_id
        );
        Ok(())
    }

    fn start_bracket<'a, 'b, 'c>(
        &'a self,
        internal_channel_id: &'b str,
        service: &'b str,
    ) -> Result<BracketId, Error<'c>> {
        let mut db = self.db.write().expect("database"); // FIXME bubble up error
        let active_bracket = find_active_bracket(&mut db, internal_channel_id, service)?;

        active_bracket.start();
        Ok(active_bracket.get_id())
    }

    fn bar_from_entering_bracket<'a, 'b, 'c>(
        &'a self,
        internal_channel_id: &'b str,
        service: &'b str,
    ) -> Result<BracketId, Error<'c>> {
        let mut db = self.db.write().expect("database"); // FIXME bubble up error
        let active_bracket = find_active_bracket(&mut db, internal_channel_id, service)?;

        active_bracket.bar_from_entering();
        Ok(active_bracket.get_id())
    }

    fn seed_bracket<'a, 'b, 'c>(
        &'a self,
        internal_channel_id: &'b str,
        service: &'b str,
        players: Vec<String>,
    ) -> Result<BracketId, Error<'c>> {
        let mut db = self.db.write().expect("database"); // FIXME bubble up error
        let active_bracket = find_active_bracket(&mut db, internal_channel_id, service)?;

        let players: Result<Vec<PlayerId>, _> =
            players.iter().map(|p| PlayerId::parse_str(p)).collect();
        if let Err(e) = active_bracket.update_seeding(&players?) {
            return Err(Error::UpdateBracket(e));
        }

        Ok(active_bracket.get_id())
    }

    fn list_players<'a, 'b>(&'a self, r: &PlayersGET) -> Result<(BracketId, Players), Error<'b>> {
        let db = self.db.read().expect("database"); // FIXME bubble up error
        let (active_bracket_id, _, _) = find_active_bracket_id(
            &db,
            r.internal_discussion_channel_id.as_str(),
            r.service.as_str(),
        )?;
        let players = match db.brackets.get(&active_bracket_id) {
            Some(b) => b.get_players(),
            None => return Err(Error::UnregisteredBracket(active_bracket_id)),
        };

        Ok((active_bracket_id, players))
    }
}

/// Return active bracket id in discussion channel, parsed discussion channel id and service
///
/// # Errors
/// Returns an error if there is no active channel
fn find_active_bracket<'a, 'b, 'c>(
    db: &'a mut InMemoryDatabase,
    internal_channel_id: &'b str,
    service: &'b str,
) -> Result<&'a mut Bracket, Error<'c>> {
    let (active_bracket_id, _, _) = find_active_bracket_id(db, internal_channel_id, service)?;
    match db.brackets.get_mut(&active_bracket_id) {
        Some(b) => Ok(b),
        None => Err(Error::UnregisteredBracket(active_bracket_id)),
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
    let service = service.parse::<Service>()?;
    match service {
        Service::Discord => {
            let id = parse_dicord_id(internal_channel_id)?;
            let channel_id = ChannelId::from(id);
            let channel_id = match db.discord_internal_channel.get(&channel_id) {
                Some(id) => *id,
                None => {
                    return Err(Error::UnregisteredDiscussionChannel(
                        service,
                        internal_channel_id.to_string(),
                    ))
                }
            };
            let active_bracket = match db.discussion_channel_active_brackets.get(&channel_id) {
                Some(id) => *id,
                None => return Err(Error::NoActiveBracketInDiscussionChannel(channel_id)),
            };

            Ok((active_bracket, channel_id, service))
        }
    }
}

/// Parse discord Id from string
///
/// # Errors
/// Returns a parsing error if discord id cannot be parsed
fn parse_dicord_id<'a, 'b>(id: &'a str) -> Result<u64, Error<'b>> {
    match id.parse::<u64>() {
        Ok(v) => Ok(v),
        Err(e) => {
            let msg = format!("could not parse discord id: {}\n{}", id, e);
            Err(Error::Parsing(msg))
        }
    }
}

impl From<UserIdParseError> for Error<'_> {
    fn from(e: UserIdParseError) -> Self {
        Self::Parsing(format!("User id: {e}"))
    }
}

impl From<ParseServiceInternalIdError> for Error<'_> {
    fn from(e: ParseServiceInternalIdError) -> Self {
        Self::Parsing(format!("Internal service: {e:?}")) // FIXME bubble up error and implement display instead of using String
    }
}

impl From<ChannelIdParseError> for Error<'_> {
    fn from(e: ChannelIdParseError) -> Self {
        Self::Parsing(format!("Channel id: {e:?}")) // FIXME bubble up error and implement display instead of using String
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
            let channel_id = channel_internal_id.parse::<ChannelId>()?;
            let channel_id = match db.discord_internal_channel.get(&channel_id) {
                Some(id) => id,
                None => {
                    return Err(Error::UnregisteredDiscussionChannel(
                        service,
                        channel_internal_id.to_string(),
                    ))
                }
            };

            let active_bracket_id = match db.discussion_channel_active_brackets.get(channel_id) {
                Some(id) => id,
                None => return Err(Error::NoActiveBracketInDiscussionChannel(*channel_id)),
            };

            let bracket = match db.brackets.iter().find(|b| b.0 == active_bracket_id) {
                Some(b) => b.1.clone(),
                None => return Err(Error::UnregisteredBracket(*active_bracket_id)),
            };

            let player_internal_id = player_internal_id.parse::<UserId>()?;
            match db.discord_internal_users.get(&player_internal_id) {
                Some(player_id) => Ok((bracket, *player_id)),
                None => Err(Error::UnregisteredPlayer),
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
