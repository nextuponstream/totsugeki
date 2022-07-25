//! In-memory database
use crate::persistence::{DBAccessor, Error};
use crate::{ApiServiceId, ApiServiceUser, BracketPOSTResult};
use serenity::model::id::{ChannelId, GuildId, UserId};
use serenity::model::misc::{ChannelIdParseError, UserIdParseError};
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use totsugeki::bracket::BracketId;
use totsugeki::join::JoinPOSTResponseBody;
use totsugeki::organiser::OrganiserId;
use totsugeki::{
    bracket::{ActiveBrackets, Bracket},
    organiser::Organiser,
};
use totsugeki::{DiscussionChannelId, PlayerId};
use uuid::Uuid;

use super::{InternalIdType, ParseInternalIdTypeError};

/// In-memory database
#[derive(Default)]
pub struct InMemoryDBAccessor {
    db: Arc<RwLock<InMemoryDatabase>>,
}

/// Link internal id of discord channel to global id
type DiscordInternalChannels = HashMap<ChannelId, DiscussionChannelId>;

/// Link internal discord guild id to organiser id
type DiscordInternalGuilds = HashMap<GuildId, OrganiserId>;

type DiscordInternalUsers = HashMap<UserId, PlayerId>;

type DiscussionChannelActiveBrackets = HashMap<DiscussionChannelId, BracketId>;

type BracketOrganiser = HashMap<BracketId, OrganiserId>;

/// In memory database
#[derive(Default)]
pub struct InMemoryDatabase {
    // TODO find out if you need internal mapping per discord Api service
    // my guess is you don't need
    brackets: HashMap<BracketId, Bracket>,
    organisers: HashMap<OrganiserId, Organiser>,
    api_services: HashMap<ApiServiceId, (String, String)>,
    service_organisers: Vec<(ApiServiceId, Box<dyn Any + Send + Sync>, OrganiserId)>,
    /// Maps to discussion channels id
    discord_internal_channel: DiscordInternalChannels,
    /// Maps to organiser id
    discord_internal_guilds: DiscordInternalGuilds,
    discord_internal_users: DiscordInternalUsers,
    discussion_channel_active_brackets: DiscussionChannelActiveBrackets,
    bracket_organiser: BracketOrganiser,
}

impl DBAccessor for InMemoryDBAccessor {
    fn clean<'a, 'b>(&'a self) -> Result<(), Error<'b>> {
        let mut db = self.db.write().expect("database"); // FIXME bubble up error
        db.brackets = HashMap::new();
        db.organisers = HashMap::new();
        db.service_organisers = vec![];
        Ok(())
    }

    fn create_bracket<'a, 'b, 'c>(
        &'a self,
        bracket_name: &'b str,
        organiser_name: &'b str,
        internal_organiser_id: String,
        internal_channel_id: String,
        internal_id_type: InternalIdType,
    ) -> Result<BracketPOSTResult, Error<'c>> {
        let mut db = self.db.write().expect("database"); // FIXME bubble up error
        let b = Bracket::new(bracket_name.to_string(), vec![]);

        // find if global id exists already for channel
        let mut discussion_channel_id = Uuid::new_v4(); // if channel is not registered yet
        match internal_id_type {
            InternalIdType::Discord => {
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
            InternalIdType::Discord => {
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
        Ok(BracketPOSTResult::new(
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
        for b in db.brackets.iter() {
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
        channel_internal_id: &'b str,
        service_type_id: &'b str,
    ) -> Result<totsugeki::join::JoinPOSTResponseBody, Error<'c>> {
        let service_type = service_type_id.parse::<InternalIdType>()?;
        let mut player_id = PlayerId::new_v4();

        let mut db = self.db.write().expect("database"); // FIXME bubble up error

        // map internal service id to global id
        match service_type {
            InternalIdType::Discord => {
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
                        .ok_or(Error::Denied(
                            "There is no active bracket in this discussion channel".to_string(),
                        ))?;

                    let organiser_id =
                        db.bracket_organiser
                            .get(bracket_id)
                            .ok_or(Error::Unknown(format!(
                                "No organiser is responsible for bracket {bracket_id}"
                            )))?;

                    let body = JoinPOSTResponseBody {
                        player_id,
                        bracket_id: *bracket_id,
                        organiser_id: *organiser_id,
                    };

                    let b = db.brackets.get(bracket_id).ok_or(Error::Unknown(format!(
                        "no bracket found for id: \"{bracket_id}\""
                    )))?;
                    let mut players = b.get_players();
                    if !players.contains(&player_id) {
                        players.push(player_id);
                    }
                    let b = Bracket::from(b.get_id(), b.get_bracket_name(), players);
                    db.brackets.insert(b.get_id(), b.clone());

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
        for b in db.brackets.iter() {
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

    fn list_service_api_user<'a, 'b, 'c>(
        &'a self,
        _offset: i64,
    ) -> Result<Vec<crate::ApiServiceUser>, Error<'c>> {
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
}

impl From<UserIdParseError> for Error<'_> {
    fn from(e: UserIdParseError) -> Self {
        Self::Parsing(e.to_string())
    }
}

impl From<ParseInternalIdTypeError> for Error<'_> {
    fn from(e: ParseInternalIdTypeError) -> Self {
        Self::Parsing(format!("{e:?}")) // FIXME bubble up error and implement display instead of using String
    }
}

impl From<ChannelIdParseError> for Error<'_> {
    fn from(e: ChannelIdParseError) -> Self {
        Self::Parsing(format!("{e:?}")) // FIXME bubble up error and implement display instead of using String
    }
}
