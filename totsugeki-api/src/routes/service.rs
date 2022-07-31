//! Service Api

use crate::{persistence::Error, ApiServiceId, SharedDb};
use crate::{ApiServiceUser, ServerKey};
use jwt::SignWithKey;
use poem::web::Data;
use poem::Result;
use poem_openapi::Object;
use poem_openapi::{param::Path, payload::Json, OpenApi};
use totsugeki::ServiceId;

/// Register response for new service to the api
#[derive(Object)]
struct ServiceRegisterPOST {
    /// Id of newly registered service
    id: ServiceId,
    /// Token for registered service
    token: String,
}

/// Service Api
pub struct Api;

#[OpenApi]
impl Api {
    /// Create new service api user
    #[oai(path = "/service/register/:name/:description", method = "post")]
    async fn register_api_service<'a>(
        &self,
        db: SharedDb<'a>,
        server_key: Data<&ServerKey>,
        name: Path<String>,
        description: Path<String>,
    ) -> Result<Json<ServiceRegisterPOST>> {
        let id = register_service(&db, name.as_str(), description.as_str())?;

        let token = ApiServiceUser::new(name.0, description.0)
            .sign_with_key(server_key.0)
            .expect("could not sign token");

        Ok(Json(ServiceRegisterPOST { id, token }))
    }

    /// List all registered services
    #[oai(path = "/service/register/:offset", method = "get")]
    async fn list_api_service<'a>(
        &self,
        db: SharedDb<'a>,
        offset: Path<i64>,
    ) -> Result<Json<Vec<ApiServiceUser>>> {
        let api_services = list_services(&db, offset.0)?;
        Ok(Json(api_services))
    }
}

/// Call to register service made to database
fn register_service<'a, 'b, 'c>(
    db: &'a SharedDb,
    service_name: &'b str,
    service_description: &'b str,
) -> Result<ApiServiceId, Error<'c>>
where
    'a: 'c,
    'b: 'c,
{
    let db = db.read()?;
    let id = db.register_service_api_user(service_name, service_description)?;
    Ok(id)
}

/// Cal to list services made to database
fn list_services<'a, 'b>(db: &'a SharedDb, offset: i64) -> Result<Vec<ApiServiceUser>, Error<'b>>
where
    'a: 'b,
{
    let db = db.read()?;
    let api_service_users = db.list_service_api_user(offset)?;
    Ok(api_service_users)
}
