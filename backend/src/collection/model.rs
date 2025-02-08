use super::{Collection, CollectionDatabase, DefaultCollectionLink};
use crate::song::LinkDatabase as SongLinkDatabase;
use crate::AppError;
use fancy_surreal::Client;
use shared::song::Link as SongLink;
use std::sync::Arc;

pub struct Model;

impl Model {
    pub async fn get(
        db: Arc<Client<'_>>,
        owners: Vec<String>,
    ) -> Result<Vec<Collection>, AppError> {
        Ok(db
            .table("collections")
            .owners(owners)
            .select()?
            .query::<CollectionDatabase>()
            .await?
            .into_iter()
            .map(|collection| collection.into())
            .collect())
    }

    pub async fn get_one(
        db: Arc<Client<'_>>,
        owners: Vec<String>,
        id: &str,
    ) -> Result<Collection, AppError> {
        Ok(db
            .table("collections")
            .owners(owners)
            .select()?
            .id(id)
            .query_one::<CollectionDatabase>()
            .await?
            .into())
    }

    pub async fn put(
        db: Arc<Client<'_>>,
        owners: Vec<String>,
        collections: Vec<Collection>,
    ) -> Result<Vec<Collection>, AppError> {
        Ok(db
            .table("collections")
            .owners(owners)
            .update::<CollectionDatabase>(
                collections
                    .into_iter()
                    .map(|collection| collection.into())
                    .collect(),
            )
            .await?
            .into_iter()
            .map(|collection| collection.into())
            .collect())
    }

    pub async fn create(
        db: Arc<Client<'_>>,
        owner: &str,
        collections: Vec<Collection>,
    ) -> Result<Vec<Collection>, AppError> {
        Ok(db
            .table("collections")
            .owner(owner)
            .create::<CollectionDatabase>(
                collections
                    .into_iter()
                    .map(|collection| collection.into())
                    .collect(),
            )
            .await?
            .into_iter()
            .map(|collection| collection.into())
            .collect())
    }

    pub async fn delete(
        db: Arc<Client<'_>>,
        owners: Vec<String>,
        collections: Vec<Collection>,
    ) -> Result<Vec<Collection>, AppError> {
        Ok(db
            .table("collections")
            .owners(owners)
            .delete::<CollectionDatabase>(
                collections
                    .into_iter()
                    .map(|collection| collection.into())
                    .collect(),
            )
            .await?
            .into_iter()
            .map(|collection| collection.into())
            .collect())
    }

    pub async fn get_song_link_numbers(
        db: Arc<Client<'_>>,
        owners: Vec<String>,
        id: &str,
    ) -> Result<Vec<Option<String>>, AppError> {
        Ok(db
            .table("collections")
            .owners(owners)
            .select()?
            .id(id)
            .field("content.songs.nr")
            .wrapper_js_map_unpack("element.content.songs.nr")
            .query_direct::<Option<String>>()
            .await?)
    }

    pub async fn add_song_to_default_collection(
        db: Arc<Client<'_>>,
        owner: &str,
        song_link: SongLink,
    ) -> Result<(), AppError> {
        let links = db
            .table("default_collection_link")
            .owner(owner)
            .select()?
            .query::<DefaultCollectionLink>()
            .await?;

        if links.len() > 1 {
            return Err(AppError::Database(
                "Inconsistent state, to many default_collection_links for the same user found"
                    .into(),
            ));
        }

        if links.len() == 1 {
            let mut collection = Self::get_one(
                db.clone(),
                vec![owner.into()],
                &links.first().unwrap().collection_id,
            )
            .await?;
            collection.songs.push(song_link);
            Self::put(db.clone(), vec![owner.into()], vec![collection]).await?;
        } else {
            let collection = Collection {
                id: None,
                title: "default".into(),
                cover: "".into(),
                songs: vec![song_link],
            };
            let collections = Self::create(db.clone(), owner, vec![collection]).await?;
            let collection = collections.first().unwrap();
            let link = DefaultCollectionLink {
                id: None,
                collection_id: collection
                    .id
                    .clone()
                    .ok_or(AppError::Database("Created collection has no id".into()))?,
            };

            db.table("default_collection_link")
                .owner(owner)
                .create::<DefaultCollectionLink>(vec![link])
                .await?;
        }
        Ok(())
    }
}
