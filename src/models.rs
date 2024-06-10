use axum::{
    extract::{self, Json, Path, Query, State},
    response::{Html, IntoResponse},
    routing::get,
};
use chrono::NaiveDate;
use diesel::*;
use diesel::{
    deserialize::{self, FromSql, FromSqlRow},
    expression::AsExpression,
    query_dsl::BelongingToDsl,
    serialize::{self, Output, ToSql},
    sql_types::{self, Integer, Jsonb},
    Associations, Identifiable, Insertable, Queryable, Selectable,
};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use time::Time;
use uuid::Uuid;
type DB = diesel::pg::Pg;

#[repr(i32)]
#[derive(Debug, Serialize, Deserialize, Clone, AsExpression)]
#[diesel(sql_type = Integer)]
pub enum PaymentMode {
    NotPaid = 1,
    Cash = 2,
    Card = 3,
    Gpay = 4,
}

impl<DB> ToSql<Integer, DB> for PaymentMode
where
    DB: diesel::backend::Backend,
    i32: ToSql<Integer, DB>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, DB>) -> serialize::Result {
        match self {
            PaymentMode::NotPaid => 0.to_sql(out),
            PaymentMode::Cash => 1.to_sql(out),
            PaymentMode::Card => 2.to_sql(out),
            PaymentMode::Gpay => 3.to_sql(out),
        }
    }
}

impl<DB> FromSql<Integer, DB> for PaymentMode
where
    DB: diesel::backend::Backend,
    i32: FromSql<Integer, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
        match i32::from_sql(bytes)? {
            0 => Ok(PaymentMode::NotPaid),
            1 => Ok(PaymentMode::Cash),
            2 => Ok(PaymentMode::Card),
            3 => Ok(PaymentMode::Gpay),
            x => Err(format!("Unrecognized variant {}", x).into()),
        }
    }
}

//NOTE: AsExpression is converting this struct to sql type Jsonb, which is the data type in our db.
#[derive(AsExpression, Queryable, Debug, Clone, Serialize, Deserialize)]
#[diesel(sql_type = Jsonb)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PaymentMethod {
    pub mode_of_payment: PaymentMode,
    pub payment_transaction_id: Option<String>,
    pub payment_receiver: Option<String>,
    pub payment_received_date: Option<NaiveDate>,
}

impl FromSqlRow<Jsonb, diesel::pg::Pg> for PaymentMethod {
    fn build_from_row<'a>(
        row: &impl diesel::row::Row<'a, diesel::pg::Pg>,
    ) -> deserialize::Result<Self> {
        Ok(PaymentMethod {
            mode_of_payment: row.get_value(0)?,
            payment_transaction_id: row.get_value(1)?,
            payment_receiver: row.get_value(2)?,
            payment_received_date: row.get_value(3)?,
        })
    }
}

#[derive(Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::reservation)]
pub struct NewResv {
    pub name: String,
    pub contact: String,
    pub seating: String,
    pub specific_seating_requested: bool,
    pub advance: bool,
    pub advance_method: serde_json::Value,
    pub advance_amount: Option<i32>,
    pub confirmed: bool,
    pub reservation_date: NaiveDate,
    pub reservation_time: Time,
    pub property_id: Uuid,
}

#[derive(Clone, Debug, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::reservation)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Reservation {
    pub id: i32,
    pub name: String,
    pub contact: String,
    pub seating: String,
    pub specific_seating_requested: bool,
    pub advance: bool,
    pub advance_method: serde_json::Value,
    pub advance_amount: Option<i32>,
    pub confirmed: bool,
    pub reservation_date: NaiveDate,
    pub reservation_time: Time,
    pub property_id: Uuid,
}
use crate::{get_connection_pool, SharedPooledConnection};
impl Reservation {
    #[axum_macros::debug_handler]
    pub async fn find_all() -> Json<Vec<Reservation>> {
        use crate::schema::reservation::dsl::{reservation, reservation_date};

        let results = reservation
            .filter(reservation_date.eq(chrono::offset::Local::now().date_naive()))
            .limit(5)
            .select(Reservation::as_select())
            .load(&mut get_connection_pool().try_get().unwrap())
            .expect("Error loading resv");

        Json(results)
    }
}

impl Display for Reservation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}, {}, {}, {}, {}, {},{}, {:?}, {:?}, {}, {}, {} )",
            self.id,
            self.name,
            self.contact,
            self.seating,
            self.specific_seating_requested,
            self.advance,
            self.advance_method,
            self.advance_amount,
            self.confirmed,
            self.reservation_date,
            self.reservation_time,
            self.property_id,
        )
    }
}

#[derive(Identifiable, Selectable, Debug, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::property)]
#[diesel(primary_key(property_id))]
pub struct Property {
    pub property_id: Uuid,
    pub property_name: String,
    #[serde(skip_serializing)]
    pub property_password: String,
    pub property_email: String,
    pub property_phone: String,
}

use crate::schema::property;
impl Queryable<property::SqlType, DB> for Property {
    type Row = (Uuid, String, String, String, String);

    fn build(row: Self::Row) -> deserialize::Result<Self> {
        Ok(Property {
            property_id: row.0,
            property_name: row.1,
            property_password: row.2,
            property_email: row.3,
            property_phone: row.4,
        })
    }
}
// impl FromSqlRow<property::SqlType, diesel::pg::Pg> for Property {
//     fn build_from_row<'a>(
//         row: &impl diesel::row::Row<'a, diesel::pg::Pg>,
//     ) -> deserialize::Result<Self> {
//         Ok(Property {
//             property_id: row.get_value(0)?,
//             property_name: row.get_value(1)?,
//             property_password: row.get_value(2)?,
//             property_email: row.get_value(3)?,
//             property_phone: row.get_value(4)?,
//         })
//     }
// }

//TODO: this should be the entry point for any new client.
// Add a field of phone number.
#[derive(Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::property)]
pub struct NewProperty {
    pub property_id: uuid::Uuid,
    pub property_name: String,
    #[serde(skip_serializing)]
    pub property_password: String,
    pub property_email: String,
    pub property_phone: String,
}

#[derive(
    Queryable, Selectable, Identifiable, Associations, Debug, PartialEq, Serialize, Deserialize,
)]
#[diesel(primary_key(user_id))]
#[diesel(table_name = crate::schema::propertyusers)]
#[diesel(belongs_to(Property, foreign_key = property_id))]
pub struct PropertyUsers {
    pub user_id: i32,
    pub user_name: String,
    #[serde(skip_serializing)]
    pub user_password: String,
    pub user_role: i32,
    pub property_id: Uuid,
}

//TODO: this struct should only be accessable after property login.
#[derive(Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::propertyusers)]
pub struct NewPropertyUser {
    pub user_name: String,
    #[serde(skip_serializing)]
    pub user_password: String,
    pub user_role: i32,
    pub property_id: Uuid,
}

#[repr(i32)]
#[derive(Debug, Serialize, Deserialize, Clone, AsExpression)]
#[diesel(sql_type = Integer)]
pub enum Role {
    Dev = 0,
    PropertyAdmin = 1,
    PropertyManager = 2,
    PropertyUser = 3,
}

#[derive(Queryable, Selectable, Insertable, Debug, PartialEq, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::roles)]
pub struct Roles {
    pub role_id: i32,
    pub role_name: String,
}

impl Roles {
    pub fn get_role(role_name: String, conn: SharedPooledConnection) -> anyhow::Result<i32> {
        use crate::schema::roles::dsl::{role_id, role_name, roles};
        Ok(roles
            .filter(role_name.eq(role_name))
            .select(role_id)
            .get_result(&mut conn.try_get().unwrap())
            .unwrap())
    }
}
