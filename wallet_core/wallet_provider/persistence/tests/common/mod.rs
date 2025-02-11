use std::convert::Infallible;

use chrono::{DateTime, Local};
use ctor::ctor;
use p256::ecdsa::{SigningKey, VerifyingKey};
use rand_core::OsRng;
use sea_orm::{
    sea_query::{Expr, Query},
    ColumnTrait, ConnectionTrait, EntityTrait, FromQueryResult, QueryFilter,
};
use uuid::Uuid;

use wallet_common::utils::random_bytes;
use wallet_provider_database_settings::Settings;
use wallet_provider_domain::{
    model::{
        encrypter::Encrypter,
        hsm::mock::MockPkcs11Client,
        wallet_user::{InstructionChallenge, WalletUserCreate},
    },
    repository::PersistenceError,
};
use wallet_provider_persistence::{
    database::Db,
    entity::{wallet_user, wallet_user_instruction_challenge},
    wallet_user::{create_wallet_user, update_instruction_challenge_and_sequence_number},
    PersistenceConnection,
};

#[ctor]
fn init_logging() {
    let _ = tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_test_writer()
            .finish(),
    );
}

pub async fn db_from_env() -> Result<Db, PersistenceError> {
    let settings = Settings::new().unwrap();
    Db::new(settings.database.connection_string(), Default::default()).await
}

pub async fn create_wallet_user_with_random_keys<S, T>(db: &T, id: Uuid, wallet_id: String)
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    create_wallet_user(
        db,
        WalletUserCreate {
            id,
            wallet_id,
            hw_pubkey: *SigningKey::random(&mut OsRng).verifying_key(),
            encrypted_pin_pubkey: Encrypter::<VerifyingKey>::encrypt(
                &MockPkcs11Client::<Infallible>::default(),
                "key1",
                *SigningKey::random(&mut OsRng).verifying_key(),
            )
            .await
            .unwrap(),
        },
    )
    .await
    .expect("Could not create wallet user");
}

pub async fn find_wallet_user<S, T>(db: &T, id: Uuid) -> Option<wallet_user::Model>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    wallet_user::Entity::find()
        .filter(wallet_user::Column::Id.eq(id))
        .one(db.connection())
        .await
        .expect("Could not fetch wallet user")
}

pub async fn create_instruction_challenge_with_random_data<S, T>(db: &T, wallet_id: String)
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    update_instruction_challenge_and_sequence_number(
        db,
        &wallet_id,
        InstructionChallenge {
            expiration_date_time: Local::now(), // irrelevant for these tests
            bytes: random_bytes(32),
        },
        0, // irrelevant for these tests
    )
    .await
    .expect("Could not create wallet user");
}

#[derive(FromQueryResult)]
pub struct InstructionChallengeResult {
    pub id: Uuid,
    pub wallet_user_id: Uuid,
    pub instruction_challenge: Vec<u8>,
    pub expiration_date_time: DateTime<Local>,
}

pub async fn find_instruction_challenges_by_wallet_id<S, T>(
    db: &T,
    wallet_id: String,
) -> Vec<InstructionChallengeResult>
where
    S: ConnectionTrait,
    T: PersistenceConnection<S>,
{
    let stmt = Query::select()
        .columns([
            wallet_user_instruction_challenge::Column::Id,
            wallet_user_instruction_challenge::Column::WalletUserId,
            wallet_user_instruction_challenge::Column::InstructionChallenge,
            wallet_user_instruction_challenge::Column::ExpirationDateTime,
        ])
        .from(wallet_user_instruction_challenge::Entity)
        .and_where(
            wallet_user_instruction_challenge::Column::WalletUserId.in_subquery(
                Query::select()
                    .column(wallet_user::Column::Id)
                    .from(wallet_user::Entity)
                    .and_where(Expr::col(wallet_user::Column::WalletId).eq(wallet_id))
                    .to_owned(),
            ),
        )
        .to_owned();

    let conn = db.connection();
    let builder = conn.get_database_backend();

    InstructionChallengeResult::find_by_statement(builder.build(&stmt))
        .all(conn)
        .await
        .expect("Could not fetch instruction challenges")
}
