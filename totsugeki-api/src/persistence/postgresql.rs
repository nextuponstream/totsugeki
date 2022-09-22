//! Postgresql database

use super::DBAccessor;
use totsugeki::{
    bracket::{CreateRequest, Id as BracketId},
    matches::{Id as MatchId, NextMatchGETResponseRaw, ReportResultPOST},
    player::{Participants, GET as PlayersGET},
};

/// Postgresql database
#[derive(Default)]
pub struct Accessor;

#[allow(
    unused_variables,
    // reason = "figure out business logic of bracket feature with inmemory before implementing"
)] // FIXME remove
impl DBAccessor for Accessor {
    fn clean<'a, 'b>(&'a self) -> Result<(), super::Error<'b>> {
        todo!()
    }

    fn close_bracket<'a, 'b, 'c>(
        &'a self,
        internal_channel_id: &'b str,
        service_type_id: &'b str,
    ) -> Result<BracketId, super::Error<'c>> {
        todo!()
    }

    fn create_bracket<'a, 'b, 'c>(
        &'a self,
        r: CreateRequest<'b>,
    ) -> Result<totsugeki::bracket::http_responses::POSTResult, super::Error<'c>> {
        todo!()
    }

    fn create_organiser<'a, 'b, 'c>(
        &'a self,
        organiser_name: &'b str,
    ) -> Result<(), super::Error<'c>> {
        todo!()
    }

    fn disqualify_player<'a, 'b, 'c>(
        &'a self,
        internal_channel_id: &'b str,
        service: &'b str,
        player_id: &'b str,
    ) -> Result<BracketId, super::Error<'c>> {
        todo!()
    }

    fn find_brackets<'a, 'b, 'c>(
        &'a self,
        bracket_name: &'b str,
        offset: i64,
    ) -> Result<Vec<totsugeki::bracket::raw::Raw>, super::Error<'c>> {
        todo!()
    }

    fn find_next_match<'a, 'b, 'c>(
        &'a self,
        player_internal_id: &'b str,
        channel_internal_id: &'b str,
        service_type_id: &'b str,
    ) -> Result<NextMatchGETResponseRaw, super::Error<'c>> {
        todo!()
    }

    fn find_organisers<'a, 'b, 'c>(
        &'a self,
        organiser_name: &'b str,
        offset: i64,
    ) -> Result<Vec<totsugeki::organiser::Organiser>, super::Error<'c>> {
        todo!()
    }

    fn forfeit<'a, 'b, 'c>(
        &'a self,
        internal_channel_id: &'b str,
        service: &'b str,
        player_internal_id: &'b str,
    ) -> Result<BracketId, super::Error<'c>> {
        todo!()
    }

    fn get_bracket<'a, 'b, 'c>(
        &'a self,
        bracket_id: BracketId,
    ) -> Result<totsugeki::bracket::raw::Raw, super::Error<'c>> {
        todo!()
    }

    fn init(&self) -> Result<(), super::Error> {
        todo!()
    }

    fn join_bracket<'a, 'b, 'c>(
        &'a self,
        player_internal_id: &'b str,
        player_name: &'b str,
        channel_internal_id: &'b str,
        service_type_id: &'b str,
    ) -> Result<totsugeki::join::POSTResponse, super::Error<'c>> {
        todo!()
    }

    fn list_brackets<'a, 'b>(
        &'a self,
        offset: i64,
    ) -> Result<Vec<totsugeki::bracket::raw::Raw>, super::Error<'b>> {
        todo!()
    }

    fn list_organisers<'a, 'b>(
        &'a self,
        offset: i64,
    ) -> Result<Vec<totsugeki::organiser::Organiser>, super::Error<'b>> {
        todo!()
    }

    fn list_players<'a, 'b>(
        &'a self,
        r: &PlayersGET,
    ) -> Result<(BracketId, Participants), super::Error<'b>> {
        todo!()
    }

    fn list_service_api_user<'a, 'b, 'c>(
        &'a self,
        offset: i64,
    ) -> Result<Vec<crate::ApiServiceUser>, super::Error<'c>> {
        todo!()
    }

    fn quit_bracket<'a, 'b, 'c>(
        &'a self,
        internal_channel_id: &'b str,
        service: &'b str,
        player_internal_id: &'b str,
    ) -> Result<BracketId, super::Error<'c>> {
        todo!()
    }

    fn register_service_api_user<'a, 'b, 'c>(
        &'a self,
        service_name: &'b str,
        service_description: &'b str,
    ) -> Result<crate::ApiServiceId, super::Error<'c>> {
        todo!()
    }

    fn remove_player<'a, 'b, 'c>(
        &'a self,
        internal_channel_id: &'b str,
        service: &'b str,
        player_id: &'b str,
    ) -> Result<BracketId, super::Error<'c>> {
        todo!()
    }

    fn report_result<'a, 'b, 'c>(
        &'a self,
        player_internal_id: &'b str,
        channel_internal_id: &'b str,
        service_type_id: &'b str,
        result: &'b str,
    ) -> Result<ReportResultPOST, super::Error<'c>> {
        todo!()
    }

    fn seed_bracket<'a, 'b, 'c>(
        &'a self,
        internal_channel_id: &'b str,
        service_type_id: &'b str,
        players: Vec<String>,
    ) -> Result<BracketId, super::Error<'c>> {
        todo!()
    }

    fn start_bracket<'a, 'b, 'c>(
        &'a self,
        internal_channel_id: &'b str,
        service_type_id: &'b str,
    ) -> Result<BracketId, super::Error<'c>> {
        todo!()
    }

    fn tournament_organiser_reports_result<'a, 'b, 'c>(
        &'a self,
        channel_internal_id: &'b str,
        service: &'b str,
        player1_id: &'b str,
        result: &'b str,
        player2_id: &'b str,
    ) -> Result<ReportResultPOST, super::Error<'c>> {
        todo!()
    }

    fn validate_result<'a, 'b>(&'a self, match_id: MatchId) -> Result<(), super::Error<'b>> {
        todo!()
    }
}
