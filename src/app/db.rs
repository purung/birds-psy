use async_trait::async_trait;
use chrono::{DateTime, Local};
use leptos::use_context;
use sqlx::PgPool;
use ulid::Ulid;
use web_push::{SubscriptionInfo, SubscriptionKeys};

use super::{errors::EyeError, please::Communicate, Contact, User};

#[derive(sqlx::FromRow, Debug)]
struct PgCard {
    uuid: String,
    name: String,
    tel: String,
    special: String,
    timestamp: DateTime<Local>,
}

#[async_trait]
impl Communicate<Contact, PgPool> for Contact {
    async fn power() -> Result<PgPool, EyeError> {
        let ctx = use_context::<PgPool>().ok_or(EyeError::ConfigError)?;
        Ok(ctx)
    }
    async fn birth(&self) -> Result<(), EyeError> {
        sqlx::query(
            r#"
            insert into contact_request (uuid, name, tel, special, timestamp)
            values ($1, $2, $3, $4, $5)
        "#,
        )
        .bind(&self.stamp.to_string())
        .bind(&self.name)
        .bind(&self.tel)
        .bind(&self.special)
        .bind(self.timestamp)
        .execute(&Self::power().await?)
        .await?;
        Ok(())
    }

    async fn destroy(id: String) -> Result<(), EyeError> {
        sqlx::query(
            r#"
            delete from contact_request where uuid = $1
        "#,
        )
        .bind(&id)
        .execute(&Self::power().await?)
        .await
        .unwrap();
        Ok(())
    }

    async fn all() -> Result<Vec<Contact>, EyeError> {
        log::info!("Grabbing all cards");
        let rows = sqlx::query_as::<_, PgCard>("select * from contact_request")
            .fetch_all(&Self::power().await?)
            .await?
            .into_iter()
            .map(Into::into)
            .collect::<Vec<Contact>>();
        log::info!("Got {} rows", rows.len());
        Ok(rows)
    }
}
impl From<PgCard> for Contact {
    fn from(val: PgCard) -> Self {
        Contact {
            stamp: Ulid::from_string(&val.uuid).unwrap_or_default(),
            name: val.name,
            tel: val.tel,
            special: val.special,
            timestamp: val.timestamp,
        }
    }
}

#[derive(sqlx::FromRow, Debug)]
struct PgSubInfo {
    user_name: String,
    endpoint: String,
    key_p256dh: String,
    key_auth: String,
}

#[async_trait]
impl Communicate<(SubscriptionInfo, User), PgPool> for (SubscriptionInfo, User) {
    async fn power() -> Result<PgPool, EyeError> {
        let ctx = use_context::<PgPool>().ok_or(EyeError::ConfigError)?;
        Ok(ctx)
    }
    async fn birth(&self) -> Result<(), EyeError> {
        sqlx::query(
            r#"
            insert into subscriptions (subscription_id, user_name, endpoint, key_p256dh, key_auth)
            values ($1, $2, $3, $4, $5)
            on conflict (endpoint)
            do update set key_p256dh = $4, key_auth = $5
        "#,
        )
        .bind(Ulid::new().to_string())
        .bind(self.1.as_ref())
        .bind(&self.0.endpoint)
        .bind(&self.0.keys.p256dh)
        .bind(&self.0.keys.auth)
        .execute(&Self::power().await?)
        .await?;
        Ok(())
    }

    async fn destroy(id: String) -> Result<(), EyeError> {
        sqlx::query(
            r#"
            delete from subscriptions where endpoint = $1
        "#,
        )
        .bind(&id)
        .execute(&Self::power().await?)
        .await
        .unwrap();
        Ok(())
    }

    async fn all() -> Result<Vec<(SubscriptionInfo, User)>, EyeError> {
        log::info!("Grabbing all subscriptions");
        let rows = sqlx::query_as::<_, PgSubInfo>("select * from subscriptions")
            .fetch_all(&Self::power().await?)
            .await?
            .into_iter()
            .map(Into::into)
            .collect::<Vec<(SubscriptionInfo, User)>>();
        log::info!("Got {} rows", rows.len());
        Ok(rows)
    }
}
impl From<PgSubInfo> for (SubscriptionInfo, User) {
    fn from(val: PgSubInfo) -> Self {
        (
            SubscriptionInfo {
                endpoint: val.endpoint,
                keys: SubscriptionKeys {
                    p256dh: val.key_p256dh,
                    auth: val.key_auth,
                },
            },
            User::try_from(val.user_name.as_str()).unwrap_or(User::Admin),
        )
    }
}
