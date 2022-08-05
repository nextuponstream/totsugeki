//! In-memory database
use super::BracketRequest;
use crate::matches::NextMatchGET;
use crate::persistence::{DBAccessor, Error};
use crate::{ApiServiceId, ApiServiceUser};
use serenity::model::id::{ChannelId, GuildId, UserId};
use serenity::model::misc::{ChannelIdParseError, UserIdParseError};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use totsugeki::bracket::{Format, Id as BracketId};
use totsugeki::join::POSTResponseBody;
use totsugeki::matches::{Opponent, ReportedResult};
use totsugeki::organiser::Id as OrganiserId;
use totsugeki::player::{Player, Players};
use totsugeki::seeding::get_balanced_round_matches_top_seed_favored;
use totsugeki::{
    bracket::{ActiveBrackets, Bracket, POSTResult},
    matches::{Error as MatchError, Id as MatchId},
    organiser::Organiser,
    seeding::Method as SeedingMethod,
};
use totsugeki::{player::Id as PlayerId, DiscussionChannelId};
use uuid::Uuid;

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
    /// Find quickly active brackets with discussion channel ID
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

    fn create_bracket<'a, 'b, 'c>(
        &'a self,
        r: BracketRequest<'b>,
    ) -> Result<POSTResult, Error<'c>> {
        let bracket_name = r.bracket_name;
        let bracket_format = r.bracket_format;
        let seeding_method = r.seeding_method;
        let organiser_name = r.organiser_name;
        let internal_organiser_id = r.organiser_internal_id;
        let internal_channel_id = r.internal_channel_id;
        let internal_id_type = r.service_type_id.parse::<Service>()?;
        let mut db = self.db.write().expect("database"); // FIXME bubble up error
        let format = bracket_format.parse::<Format>()?;
        let seeding_method = seeding_method.parse::<SeedingMethod>()?;
        let b = Bracket::new(bracket_name.to_string(), vec![], format, seeding_method);

        // find if global id exists already for channel
        let mut discussion_channel_id = Uuid::new_v4(); // if channel is not registered yet
        match internal_id_type {
            Service::Discord => {
                let internal_channel_id = match internal_channel_id.parse::<ChannelId>() {
                    Ok(v) => v,
                    Err(e) => {
                        let msg =
                            format!("could not parse channel id: {}\n{}", internal_channel_id, e);
                        return Err(Error::Parsing(msg));
                    }
                };
                if let Some(m) = db.discord_internal_channel.get(&internal_channel_id) {
                    discussion_channel_id = *m;
                } else {
                    db.discord_internal_channel
                        .insert(internal_channel_id, discussion_channel_id);
                }
            }
        }

        // find if global id already exists for organiser
        let mut organiser_id = Uuid::new_v4();
        match internal_id_type {
            Service::Discord => {
                let id = match internal_organiser_id.parse::<u64>() {
                    Ok(v) => v,
                    Err(e) => {
                        let msg =
                            format!("could not parse guild id: {}\n{}", internal_organiser_id, e);
                        return Err(Error::Parsing(msg));
                    }
                };
                let internal_organiser_id = GuildId::from(id);
                if let Some(id) = db.discord_internal_guilds.get(&internal_organiser_id) {
                    organiser_id = *id;
                } else {
                    db.discord_internal_guilds
                        .insert(internal_organiser_id, organiser_id);
                }
            }
        }

        // Update organiser
        let mut active_brackets = ActiveBrackets::new();
        if let Some(m) = db.organisers.iter_mut().find(|o| o.0 == &organiser_id) {
            let o = m.1;
            active_brackets = o.get_active_brackets();
            active_brackets.insert(discussion_channel_id, b.get_id());
            organiser_id = o.get_organiser_id();
        } else {
            active_brackets.insert(discussion_channel_id, b.get_id());
        }
        let o = Organiser::new(
            organiser_id,
            organiser_name.to_string(),
            Some(active_brackets),
        );
        db.organisers.insert(organiser_id, o);

        db.discussion_channel_active_brackets
            .insert(discussion_channel_id, b.get_id());
        db.bracket_organiser.insert(b.get_id(), organiser_id);

        // use uuid of service, then id of discussion_channel_id as key to get organiser id as val in kv map
        db.brackets.insert(b.get_id(), b.clone());
        Ok(POSTResult::from(
            b.get_id(),
            organiser_id,
            discussion_channel_id,
        ))

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
        let organiser_id = Uuid::new_v4();
        db.organisers.insert(
            organiser_id,
            Organiser::new(organiser_id, organiser_name.to_string(), None),
        );
        Ok(())
    }

    fn find_brackets<'a, 'b, 'c>(
        &'a self,
        bracket_name: &'b str,
        _offset: i64,
    ) -> Result<Vec<Bracket>, Error<'c>> {
        let db = self.db.read().expect("database");
        let mut brackets = vec![];
        for b in &db.brackets {
            if b.1.get_bracket_name() == bracket_name {
                brackets.push(b.1.clone());
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
        let service_type = service_type_id.parse::<Service>()?;
        let mut player_id = PlayerId::new_v4();

        let mut db = self.db.write().expect("database"); // FIXME bubble up error

        // map internal service id to global id
        match service_type {
            Service::Discord => {
                let player_internal_id = player_internal_id.parse::<UserId>()?;
                let channel_internal_id = channel_internal_id.parse::<ChannelId>()?;

                if let Some(id) = db.discord_internal_users.get(&player_internal_id) {
                    player_id = *id;
                }

                if let Some(id) = db.discord_internal_channel.get(&channel_internal_id) {
                    let discussion_channel_id = *id;
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
                        Error::Unknown(format!(
                            "No organiser is responsible for bracket {bracket_id}"
                        ))
                    })?;

                    let body = POSTResponseBody {
                        player_id,
                        bracket_id: *bracket_id,
                        organiser_id: *organiser_id,
                    };

                    let b = db.brackets.get(bracket_id).ok_or_else(|| {
                        Error::Unknown(format!("no bracket found for id: \"{bracket_id}\""))
                    })?;
                    let mut players = b.get_players();
                    if !players.iter().any(|p| p.get_id() == player_id) {
                        players.push(Player {
                            id: player_id,
                            name: player_name.to_string(),
                        });
                    }
                    let matches = if players.len() < 3 {
                        vec![]
                    } else {
                        get_balanced_round_matches_top_seed_favored(&Players::from(
                            players
                                .iter()
                                .map(totsugeki::player::Player::get_id)
                                .collect(),
                        )?)?
                    };
                    let b = Bracket::from(
                        b.get_id(),
                        b.get_bracket_name(),
                        players,
                        matches,
                        b.get_format(),
                        b.get_seeding_method(),
                    );
                    db.brackets.insert(b.get_id(), b);

                    Ok(body)
                } else {
                    return Err(Error::Parsing("Unknown discord guild".to_string()));
                }
            }
        }
    }

    fn list_brackets<'a, 'b>(&'a self, _offset: i64) -> Result<Vec<Bracket>, Error<'b>> {
        let db = self.db.read().expect("database"); // FIXME bubble up error
        let mut brackets = vec![];
        for b in &db.brackets {
            brackets.push(b.1.clone());
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

    fn get_bracket<'a, 'b, 'c>(&'a self, bracket_id: BracketId) -> Result<Bracket, Error<'c>> {
        let db = self.db.read().expect("database"); // FIXME bubble up error
        let bracket = match db.brackets.get(&bracket_id) {
            Some(b) => b,
            None => return Err(Error::BracketNotFound(bracket_id)),
        };
        Ok(bracket.clone())
    }

    fn find_next_match<'a, 'b, 'c>(
        &'a self,
        player_internal_id: &'b str,
        channel_internal_id: &'b str,
        service_type_id: &'b str,
    ) -> Result<crate::matches::NextMatchGET, Error<'c>> {
        let db = self.db.read().expect("database"); // FIXME bubble up error
        let service_type = service_type_id.parse::<Service>()?;
        let (bracket, player_id) = match service_type {
            Service::Discord => {
                let channel_internal_id = channel_internal_id.parse::<ChannelId>()?;
                let channel_id = match db.discord_internal_channel.get(&channel_internal_id) {
                    Some(id) => id,
                    None => return Err(Error::DiscussionChannelNotFound),
                };

                let active_bracket_id = match db.discussion_channel_active_brackets.get(channel_id)
                {
                    Some(id) => id,
                    None => return Err(Error::NoActiveBracketInDiscussionChannel),
                };

                let bracket = match db.brackets.iter().find(|b| b.0 == active_bracket_id) {
                    Some(b) => b.1.clone(),
                    None => return Err(Error::BracketNotFound(*active_bracket_id)),
                };

                let player_internal_id = player_internal_id.parse::<UserId>()?;
                match db.discord_internal_users.get(&player_internal_id) {
                    Some(player_id) => (bracket, *player_id),
                    None => return Err(Error::PlayerNotFound),
                }
            }
        };

        match bracket.get_format() {
            Format::SingleElimination => {
                // TODO save info about what round a player is playing for a given bracket
                // so you don't need to search the whole tree for it

                // search from first round, find match player is in, then the next one if he wins
                if bracket.get_matches().is_empty() {
                    return Err(Error::NoNextMatch);
                }

                for round in &bracket.get_matches() {
                    let is_last_round = round.len() == 1;
                    for m in round {
                        if let Opponent::Player(id) = m.get_players()[0] {
                            if id == player_id {
                                match m.get_winner() {
                                    Opponent::Player(winner) => {
                                        if winner == player_id {
                                            if is_last_round {
                                                return Err(Error::NoNextMatch);
                                            }
                                            continue; // search next round
                                        }
                                        return Err(Error::EliminatedFromBracket);
                                    }
                                    Opponent::Bye => todo!(), // TODO add some serious error
                                    Opponent::Unknown => {
                                        return Ok(NextMatchGET {
                                            opponent: m.get_players()[1].to_string(),
                                            match_id: m.get_id(),
                                            bracket_id: bracket.get_id(),
                                        })
                                    }
                                }
                            }
                        }
                        if let Opponent::Player(id) = m.get_players()[1] {
                            if id == player_id {
                                match m.get_winner() {
                                    Opponent::Player(winner) => {
                                        if winner == player_id {
                                            if is_last_round {
                                                return Err(Error::NoNextMatch);
                                            }
                                            continue; // search next round
                                        }
                                        return Err(Error::EliminatedFromBracket);
                                    }
                                    Opponent::Bye => todo!(), // TODO add some serious error
                                    Opponent::Unknown => {
                                        return Ok(NextMatchGET {
                                            opponent: m.get_players()[0].to_string(),
                                            match_id: m.get_id(),
                                            bracket_id: bracket.get_id(),
                                        })
                                    }
                                }
                            }
                        }
                    }
                }
                Err(Error::NextMatchNotFound)
            }
        }
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
        let (active_bracket_id, player_id) = match service_type {
            Service::Discord => {
                let channel_internal_id = channel_internal_id.parse::<ChannelId>()?;
                let channel_id = match db.discord_internal_channel.get(&channel_internal_id) {
                    Some(id) => id,
                    None => return Err(Error::DiscussionChannelNotFound),
                };

                let active_bracket_id = match db.discussion_channel_active_brackets.get(channel_id)
                {
                    Some(id) => *id,
                    None => return Err(Error::NoActiveBracketInDiscussionChannel),
                };

                let player_internal_id = player_internal_id.parse::<UserId>()?;
                match db.discord_internal_users.get(&player_internal_id) {
                    Some(player_id) => (active_bracket_id, *player_id),
                    None => return Err(Error::PlayerNotFound),
                }
            }
        };

        let bracket = db
            .brackets
            .iter_mut()
            .find(|b| b.0 == &active_bracket_id)
            .expect("bracket")
            .1;
        match bracket.get_format() {
            Format::SingleElimination => {
                // TODO save info about what round a player is playing for a given bracket
                // so you don't need to search the whole tree for it

                // search from first round, find match player is in, then the next one if he wins
                if bracket.get_matches().is_empty() {
                    return Err(Error::NoNextMatch);
                }

                for round in &mut bracket.matches {
                    for m in round {
                        if let Opponent::Player(id) = m.get_players()[0] {
                            if id == player_id {
                                match m.get_winner() {
                                    // the match where player is with no winner is where
                                    // you report the result
                                    Opponent::Unknown => {
                                        match m.get_players()[1] {
                                            Opponent::Player(_) => {
                                                m.reported_results[0] = result.0;
                                                return Ok(m.get_id());
                                            }
                                            Opponent::Bye => unreachable!(), // TODO better error
                                            Opponent::Unknown => {
                                                return Err(Error::NoOpponent);
                                            }
                                        }
                                    }
                                    Opponent::Player(_) | Opponent::Bye => {
                                        continue;
                                    }
                                }
                            }
                        }
                        if let Opponent::Player(id) = m.get_players()[1] {
                            if id == player_id {
                                match m.get_winner() {
                                    Opponent::Player(winner) => {
                                        if winner == player_id {
                                            continue; // search next round
                                        }
                                        return Err(Error::NoNextMatch);
                                    }
                                    Opponent::Bye => todo!(), // TODO add some serious error
                                    Opponent::Unknown => {
                                        m.reported_results[1] = result.0;
                                        return Ok(m.get_id());
                                    }
                                }
                            }
                        }
                    }
                }
                Err(Error::NextMatchNotFound)
            }
        }
    }

    fn validate_result<'a, 'b>(&'a self, match_id: MatchId) -> Result<(), Error<'b>> {
        // NOTE: this should be a hashset
        let mut db = self.db.write().expect("database");
        for bracket in &mut db.brackets {
            let mut winner = Opponent::Unknown;
            let mut seed = 0;
            for round in &mut (*bracket.1).matches {
                match winner {
                    // place winner in next round
                    Opponent::Player(winner_id) => {
                        for m in round {
                            if m.get_seeds().contains(&seed) {
                                if let Opponent::Unknown = m.players[0] {
                                    m.players[0] = Opponent::Player(winner_id);
                                    return Ok(());
                                } else if let Opponent::Unknown = m.players[1] {
                                    m.players[1] = Opponent::Player(winner_id);
                                    return Ok(());
                                }
                                return Err(Error::NoOpponent);
                            }
                        }
                    }
                    Opponent::Bye => todo!(),
                    // validate match result
                    Opponent::Unknown => {
                        let is_last_round = round.len() == 1;
                        for m in round {
                            if m.get_id() == match_id {
                                seed = m.set_outcome()?;
                                winner = m.get_winner();

                                if is_last_round {
                                    // then there is no next match to update
                                    return Ok(());
                                }
                            }
                        }
                    }
                }
            }
        }

        return Err(Error::Match(MatchError::NotFound));
    }
}

impl From<UserIdParseError> for Error<'_> {
    fn from(e: UserIdParseError) -> Self {
        Self::Parsing(e.to_string())
    }
}

impl From<ParseServiceInternalIdError> for Error<'_> {
    fn from(e: ParseServiceInternalIdError) -> Self {
        Self::Parsing(format!("{e:?}")) // FIXME bubble up error and implement display instead of using String
    }
}

impl From<ChannelIdParseError> for Error<'_> {
    fn from(e: ChannelIdParseError) -> Self {
        Self::Parsing(format!("{e:?}")) // FIXME bubble up error and implement display instead of using String
    }
}
