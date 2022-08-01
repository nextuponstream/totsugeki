//! Postgresql database

use super::DBAccessor;

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

    fn create_organiser<'a, 'b, 'c>(
        &'a self,
        organiser_name: &'b str,
    ) -> Result<(), super::Error<'c>> {
        todo!()
    }

    fn find_brackets<'a, 'b, 'c>(
        &'a self,
        bracket_name: &'b str,
        offset: i64,
    ) -> Result<Vec<totsugeki::bracket::Bracket>, super::Error<'c>> {
        todo!()
    }

    fn find_organisers<'a, 'b, 'c>(
        &'a self,
        organiser_name: &'b str,
        offset: i64,
    ) -> Result<Vec<totsugeki::organiser::Organiser>, super::Error<'c>> {
        todo!()
    }

    fn init(&self) -> Result<(), super::Error> {
        todo!()
    }

    fn list_brackets<'a, 'b>(
        &'a self,
        offset: i64,
    ) -> Result<Vec<totsugeki::bracket::Bracket>, super::Error<'b>> {
        todo!()
    }

    fn list_organisers<'a, 'b>(
        &'a self,
        offset: i64,
    ) -> Result<Vec<totsugeki::organiser::Organiser>, super::Error<'b>> {
        todo!()
    }

    fn list_service_api_user<'a, 'b, 'c>(
        &'a self,
        offset: i64,
    ) -> Result<Vec<crate::ApiServiceUser>, super::Error<'c>> {
        todo!()
    }

    fn register_service_api_user<'a, 'b, 'c>(
        &'a self,
        service_name: &'b str,
        service_description: &'b str,
    ) -> Result<crate::ApiServiceId, super::Error<'c>> {
        todo!()
    }

    fn join_bracket<'a, 'b, 'c>(
        &'a self,
        player_internal_id: &'b str,
        channel_internal_id: &'b str,
        service_type_id: &'b str,
    ) -> Result<totsugeki::join::POSTResponseBody, super::Error<'c>> {
        todo!()
    }

    fn get_bracket<'a, 'b>(
        &'a self,
        bracket_id: totsugeki::bracket::Id,
    ) -> Result<totsugeki::bracket::Bracket, super::Error<'b>> {
        todo!()
    }

    fn create_bracket<'a, 'b, 'c>(
        &'a self,
        r: super::BracketRequest<'b>,
    ) -> Result<totsugeki::bracket::POSTResult, super::Error<'c>> {
        todo!()
    }
}
