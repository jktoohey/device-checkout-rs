use diesel;
use rocket;
use crate::utils;

use rocket::request::FromRequest;
use std::ops::Deref;
use rocket::outcome::Outcome;
use rocket::request;

/// Initializes a database pool.
pub fn init_pool(
    config: &utils::types::Settings,
) -> diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::sqlite::SqliteConnection>> {
    let database_url = config.database_url.clone();
    let manager =
        diesel::r2d2::ConnectionManager::<diesel::sqlite::SqliteConnection>::new(database_url);
    diesel::r2d2::Pool::builder()
        .max_size(1)
        .build(manager)
        .expect("Database pool")
}

// Connection request guard type: a wrapper around an r2d2 pooled connection.
pub struct DbConn(
    pub  diesel::r2d2::PooledConnection<
        diesel::r2d2::ConnectionManager<diesel::sqlite::SqliteConnection>,
    >,
);

/// Attempts to retrieve a single connection from the managed database pool. If
/// no pool is currently managed, fails with an `InternalServerError` status. If
/// no connections are available, fails with a `ServiceUnavailable` status.
#[rocket::async_trait]
impl<'r> FromRequest<'r> for DbConn {
    type Error = ();

    async fn from_request(
        request: &'r request::Request<'_>,
    ) -> request::Outcome<Self, ()> {
        let pool = request.guard::<&'r rocket::State<diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::sqlite::SqliteConnection>>>>().await;

        if let Outcome::Success(pool) = pool {
            if let Ok(conn) = pool.get() {
                return Outcome::Success(DbConn(conn));
            }
        }

        return Outcome::Failure((rocket::http::Status::ServiceUnavailable, ()));
    }
}

// For the convenience of using an &DbConn as an &SqliteConnection.
impl Deref for DbConn {
    type Target = diesel::sqlite::SqliteConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
