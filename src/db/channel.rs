use std::borrow::Cow;

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::task;

use crate::error::Error;
use crate::schema::channels;
use crate::state::DbContext;

#[derive(Queryable, Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Channel {
    pub id: i32,
    pub twitch_room_id: Option<i32>,
    pub name: String,
    pub join_on_start: bool,
    pub command_prefix: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub silent: bool,
}

#[derive(Insertable, AsChangeset, Clone)]
#[table_name = "channels"]
pub struct UpdateChannel {
    pub twitch_room_id: Option<i32>,
    pub name: Cow<'static, str>,
}

pub async fn get_channel(ctx: &DbContext, channel_name: &str) -> Result<Option<Channel>, Error> {
    let channel_name = channel_name.to_owned();
    let ctx = ctx.clone();

    task::spawn_blocking(move || get_channel_blocking(&ctx, &channel_name)).await?
}

fn get_channel_blocking(ctx: &DbContext, channel_name: &str) -> Result<Option<Channel>, Error> {
    let pg = &*ctx.db_pool.get()?;
    let query_result = channels::table
        .filter(channels::name.eq(channel_name))
        .first::<Channel>(pg);

    match query_result {
        Ok(channel) => Ok(Some(channel)),
        Err(diesel::result::Error::NotFound) => Ok(None),
        Err(err) => Err(Error::Database(err)),
    }
}

pub async fn get_or_save_channel(
    ctx: &DbContext,
    channel_values: UpdateChannel,
) -> Result<Channel, Error> {
    let ctx = ctx.clone();
    task::spawn_blocking(move || {
        if let Some(channel) = get_channel_blocking(&ctx, &channel_values.name)? {
            Ok(channel)
        } else {
            let pg_conn = ctx.db_pool.get()?;
            let inserted_channel = diesel::insert_into(channels::table)
                .values(&channel_values)
                .get_result::<Channel>(&pg_conn)?;
            Ok(inserted_channel)
        }
    })
    .await?
}
