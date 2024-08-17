use super::Player;
use crate::collection::Model as CollectionModel;
use crate::error::AppError;
use crate::rest::parse_user_header;
use crate::song::Model as SongModel;
use crate::song::QueryParams;
use crate::user::Model as UserModel;
use crate::like::Model as LikeModel;

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
            CollectionModel::get_song_link_numbers(db.clone(), owners.clone(), collection)
                .await?
                .into_iter()
                .map(|nr| nr.unwrap_or("".into()))
                .filter(|nr| nr.len() > 0)
                .chain(std::iter::successors(Some(1), |&n| Some(n + 1)).map(|nr| nr.to_string())),
        )
    } else {
        player.add_numbers_range();
    }

    let ids = player.toc().iter()
        .filter(|toc| toc.id.is_some())
        .map(|toc| toc.id.clone().unwrap())
        .collect::<Vec<String>>();

    player = player.like_multi(&LikeModel::filter_liked(db, &ids, owners).await?);

    Ok(HttpResponse::Ok().json(player))
}
