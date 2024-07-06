use super::Player;
use crate::collection::Model as CollectionModel;
use crate::error::AppError;
use crate::rest::parse_user_header;
use crate::song::Model as SongModel;
use crate::song::QueryParams;
use crate::user::Model as UserModel;

use fancy_surreal::Client;

use actix_web::{get, web::Data, web::Query, HttpRequest, HttpResponse};

#[get("/api/player")]
pub async fn get(
    req: HttpRequest,
    db: Data<Client<'_>>,
    q: Query<QueryParams>,
) -> Result<HttpResponse, AppError> {
    let db = db.into_inner();
    let filter = q.to_filter();

    let owners = UserModel::get_or_create(db.clone(), &parse_user_header(&req)?)
        .await?
        .read;

    let mut player = SongModel::get(db.clone(), owners.clone(), &filter)
        .await?
        .into_iter()
        .map(|song| Player::from(song))
        .try_fold(Player::default(), |acc, result| {
            Ok::<Player, AppError>(acc + result)
        })?;

    if let Some(collection) = filter.get_collection() {
        player.add_numbers(
            CollectionModel::get_song_link_numbers(db, owners, collection)
                .await?
                .into_iter()
                .filter(|nr| nr.len() > 0),
        )
    } else {
        player.add_numbers_range();
    }

    Ok(HttpResponse::Ok().json(player))
}
