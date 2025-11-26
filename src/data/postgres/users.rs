use sqlx::PgPool;
use sqlx::types::Json;
use uuid::Uuid;
use crate::api::Request;
use crate::data::postgres::types::{CreateUser, User};

pub async fn init_database(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(r#"CREATE EXTENSION IF NOT EXISTS pgcrypto;"#)
        .execute(pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id                     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            email                  VARCHAR(255) UNIQUE NOT NULL,

            tags                   TEXT[],
            inbounds               TEXT[],

            traffic_limit          BIGINT NOT NULL DEFAULT 0,
            traffic_used           BIGINT NOT NULL DEFAULT 0,

            reset_traffic_every    BIGINT,
            last_traffic_reset_at  TIMESTAMPTZ,

            expire_at              TIMESTAMPTZ,

            ip_limit               BIGINT NOT NULL DEFAULT 0,
            ip_list                JSONB,
            ip_limit_punishment    JSONB,
            ip_expire_after        BIGINT NOT NULL DEFAULT 0,

            is_active              BOOLEAN NOT NULL DEFAULT true,
            created_at             TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at             TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        "#
    ).execute(pool).await?;

    sqlx::query(
        r#"
        CREATE OR REPLACE FUNCTION set_updated_at()
        RETURNS TRIGGER AS $$
        BEGIN
            NEW.updated_at := NOW();
            RETURN NEW;
        END;
        $$ LANGUAGE plpgsql;
        "#
    ).execute(pool).await?;

    // Trigger
    sqlx::query(
        r#"
        DO $$
        BEGIN
            IF NOT EXISTS (
                SELECT 1
                FROM pg_trigger
                WHERE tgname = 'set_timestamp'
            ) THEN
                CREATE TRIGGER set_timestamp
                BEFORE UPDATE ON users
                FOR EACH ROW
                EXECUTE FUNCTION set_updated_at();
            END IF;
        END;
        $$;
        "#
    ).execute(pool).await?;

    println!("Database initialized successfully");
    Ok(())
}

pub async fn create_user(
    pool: &PgPool,
    data: CreateUser,
) -> Result<User, sqlx::Error> {
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (
            email,
            tags,
            inbounds,
            traffic_limit,
            reset_traffic_every,
            expire_at,
            ip_limit,
            ip_limit_punishment,
            ip_expire_after,
            is_active
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING *;
        "#
    )
        .bind(data.email)
        .bind(data.tags)
        .bind(data.inbounds)
        .bind(data.traffic_limit)
        .bind(data.reset_traffic_every)
        .bind(data.expire_at)
        .bind(data.ip_limit)
        .bind(data.ip_limit_punishment)
        .bind(data.ip_expire_after)
        .bind(data.is_active)
        .fetch_one(pool)
        .await?;

    Ok(user)
}

pub async fn get_all_user_emails(
    pool: &PgPool
) -> Result<Vec<String>, sqlx::Error> {
    let emails = sqlx::query_scalar::<_, String>(
        r#"
        SELECT email FROM users;
        "#
    )
        .fetch_all(pool)
        .await?;

    Ok(emails)
}


pub async fn get_all_users(
    pool: &PgPool
) -> Result<Vec<User>, sqlx::Error> {
    let users = sqlx::query_as::<_, User>(
        r#"
        SELECT * FROM users ORDER BY created_at DESC;
        "#
    )
        .fetch_all(pool)
        .await?;

    Ok(users)
}

pub async fn get_user_by_id(
    pool: &PgPool,
    id: Uuid
) -> Result<Option<User>, sqlx::Error> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT * FROM users WHERE id = $1;
        "#
    )
        .bind(id)
        .fetch_optional(pool)
        .await?;

    Ok(user)
}

pub async fn get_user_by_email(
    pool: &PgPool,
    email: &str
) -> Result<Option<User>, sqlx::Error> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT * FROM users WHERE email = $1;
        "#
    )
        .bind(email)
        .fetch_optional(pool)
        .await?;

    Ok(user)
}

pub async fn delete_user_by_id(
    pool: &PgPool,
    id: Uuid
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM users WHERE id = $1;
        "#
    )
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn delete_user_by_email(
    pool: &PgPool,
    email: &str
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM users WHERE email = $1;
        "#
    )
        .bind(email)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn query_users_by_tags(
    pool: &PgPool,
    tag: Vec<String>
) -> Result<Vec<User>, sqlx::Error> {
    let users = sqlx::query_as::<_, User>(
        r#"
        SELECT * FROM users WHERE tags @> $1;
        "#
    )
        .bind(tag)
        .fetch_all(pool)
        .await?;

    Ok(users)
}

pub async fn query_users_by_inbounds(
    pool: &PgPool,
    inbounds: Vec<String>
) -> Result<Vec<User>, sqlx::Error> {
    let users = sqlx::query_as::<_, User>(
        r#"
        SELECT * FROM users WHERE inbounds @> $1;
        "#
    )
        .bind(inbounds)
        .fetch_all(pool)
        .await?;

    Ok(users)
}

pub async fn update_user(
    pool: &PgPool,
    req: Request,
) -> Result<User, sqlx::Error> {
    let (email, tags, inbounds, traffic_limit, reset_traffic_every, expire_at, ip_limit,
        ip_limit_punishment, ip_expire_after, is_active) = match req {
            Request::UpdateUser { email, tags, inbounds,
                traffic_limit, reset_traffic_every, expire_at,
                ip_limit, ip_limit_punishment, ip_expire_after,
                is_active } => {
                (email, tags, inbounds, traffic_limit, reset_traffic_every, expire_at, ip_limit,
                 ip_limit_punishment, ip_expire_after, is_active)
            },
            _ => {
                return Err(sqlx::Error::InvalidArgument("Invalid request".to_string()));
            },
        };

    let user = sqlx::query_as::<_, User>(
        r#"
        UPDATE users
        SET
            tags                = COALESCE($2, tags),
            inbounds            = COALESCE($3, inbounds),
            traffic_limit       = COALESCE($4, traffic_limit),
            reset_traffic_every = COALESCE($5, reset_traffic_every),
            expire_at           = COALESCE($6, expire_at),
            ip_limit            = COALESCE($7, ip_limit),
            ip_limit_punishment = COALESCE($8, ip_limit_punishment),
            ip_expire_after     = COALESCE($9, ip_expire_after),
            is_active           = COALESCE($10, is_active)
        WHERE email = $1
        RETURNING *;
        "#
    )
    .bind(&email)
    .bind(tags)                           // Option<Vec<String>>
    .bind(inbounds)                       // Option<Vec<String>>
    .bind(traffic_limit)                  // Option<i64>
    .bind(reset_traffic_every)            // Option<i64>
    .bind(expire_at)                      // Option<DateTime<Utc>>
    .bind(ip_limit)                       // Option<i64>
    .bind(ip_limit_punishment.map(Json))  // Option<IpLimitPunishment> -> Option<Json<_>>
    .bind(ip_expire_after)                // Option<i64>
    .bind(is_active)                      // Option<bool>
    .fetch_one(pool)
    .await?;

    Ok(user)
}