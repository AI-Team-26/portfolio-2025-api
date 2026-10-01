use crate::{
    entities::custodian::{Custodian, CustodianKind},
    repositories::{errors::DatabaseError, repository_traits::BaseRepository},
};
use sqlx::PgPool;

#[derive(Clone)]
pub struct CustodianRepository {
    db_pool: PgPool,
}

impl BaseRepository for CustodianRepository {}

impl CustodianRepository {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    pub async fn create(&self, custodian: Custodian) -> Result<i32, DatabaseError> {
        let result = sqlx::query(
            r#"
                INSERT INTO Custodians (user_id, name, custodian, account, kind, color_code, description)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                RETURNING id
            "#
        )
        .bind(&custodian.user_id)
        .bind(&custodian.name)
        .bind(&custodian.custodian)
        .bind(&custodian.account)
        .bind(&custodian.kind)
        .bind(&custodian.color_code)
        .bind(&custodian.description)
        .fetch_one(&self.db_pool)
        .await;

        self.check_result_for_new_id(result)
    }

    pub async fn update(&self, custodian: Custodian) -> Result<(), DatabaseError> {
        let result = sqlx::query!(
            r#"
                UPDATE Custodians
                SET name = $3, custodian = $4, account = $5, kind = $6, color_code = $7, description = $8
                where id = $1 and user_id = $2
            "#,
            custodian.id,
            custodian.user_id,
            custodian.name,
            custodian.custodian,
            custodian.account,
            custodian.kind as CustodianKind,
            custodian.color_code,
            custodian.description
        )
        .execute(&self.db_pool)
        .await
        .map_err(DatabaseError::from)?;

        self.check_result(result)
    }

    pub async fn delete(&self, id: i32, user_id: &str) -> Result<(), DatabaseError> {
        let result = sqlx::query!(
            r#"delete from Custodians where id = $1 and user_id = $2"#,
            id,
            user_id
        )
        .execute(&self.db_pool)
        .await
        .map_err(DatabaseError::from)?;

        self.check_result(result)
    }

    pub async fn single(&self, id: i32, user_id: &str) -> Result<Custodian, DatabaseError> {
        let item =
            sqlx::query_as!(
                Custodian,
                r#"SELECT id, user_id, name, custodian, account, kind as "kind!: CustodianKind", color_code, description
                FROM Custodians
                WHERE user_id=$1 AND id=$2 "#,
                user_id, id)
                    .fetch_one(&self.db_pool)
                    .await
                    .map_err(DatabaseError::from)?;
        Ok(item)
    }

    pub async fn list(&self) -> Result<Vec<Custodian>, DatabaseError> {
        let custodians = sqlx::query_as!(Custodian,
            r#"
                SELECT id, user_id, name, custodian, account, kind as "kind!: CustodianKind", color_code, description
                FROM Custodians
            "#
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(DatabaseError::from)?;

        Ok(custodians)
    }
}