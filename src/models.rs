use crate::schema::property;
use crate::SharedPooledConnection;
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
    serialize::{self, Output, ToSql},
    sql_types::{self, Date, Integer, Jsonb, Text},
    Associations, Identifiable, Insertable, Queryable, Selectable,
};
use leptos::{IntoView, View};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use time::Time;
use uuid::Uuid;
type DB = diesel::pg::Pg;

#[repr(i32)]
#[derive(Debug, Serialize, Deserialize, Clone, AsExpression)]
#[diesel(sql_type = Integer)]
pub enum PaymentMode {
    NotPaid = 0,
    Cash = 1,
    Card = 2,
    Gpay = 3,
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
// String type is stored as Text, NaiveDate type is stored as Date.
#[derive(AsExpression, Queryable, Debug, Clone, Serialize, Deserialize)]
#[diesel(sql_type = Jsonb)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PaymentMethod {
    mode_of_payment: PaymentMode,
    payment_transaction_id: Option<String>,
    payment_receiver: Option<String>,
    payment_received_date: Option<NaiveDate>,
}

impl FromSqlRow<Jsonb, diesel::pg::Pg> for PaymentMethod {
    fn build_from_row<'a>(
        row: &impl diesel::row::Row<'a, diesel::pg::Pg>,
    ) -> deserialize::Result<Self> {
        Ok(PaymentMethod {
            mode_of_payment: row.get_value(0)?,
            // This needs a type annotation
            // Text(datatype stored in db), String(datatype we want), usize(datatype for index)
            payment_transaction_id: Some(row.get_value::<Text, String, usize>(1)?),
            payment_receiver: Some(row.get_value::<Text, String, usize>(2)?),
            payment_received_date: Some(row.get_value::<Date, NaiveDate, usize>(3)?),
        })
    }
}

impl PaymentMethod {
    pub fn new(
        mode_of_payment: PaymentMode,
        payment_transaction_id: Option<String>,
        payment_receiver: Option<String>,
        payment_received_date: Option<NaiveDate>,
    ) -> Self {
        Self {
            mode_of_payment,
            payment_transaction_id,
            payment_receiver,
            payment_received_date,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Insertable, Debug)]
#[diesel(table_name = crate::schema::reservation)]
pub struct NewResv {
    name: String,
    contact: String,
    seating: String,
    specific_seating_requested: bool,
    advance: bool,
    advance_method: serde_json::Value,
    advance_amount: Option<i32>,
    confirmed: bool,
    reservation_date: NaiveDate,
    reservation_time: Time,
    property_id: Uuid,
}

impl NewResv {
    pub fn new(
        name: String,
        contact: String,
        seating: String,
        specific_seating_requested: bool,
        advance: bool,
        advance_method: serde_json::Value,
        advance_amount: Option<i32>,
        confirmed: bool,
        reservation_date: NaiveDate,
        reservation_time: Time,
        property_id: Uuid,
    ) -> Self {
        Self {
            name,
            contact,
            seating,
            specific_seating_requested,
            advance,
            advance_method,
            advance_amount,
            confirmed,
            reservation_date,
            reservation_time,
            property_id,
        }
    }
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

impl IntoView for Reservation {
    fn into_view(self) -> View {
        View::Transparent(leptos::leptos_dom::Transparent::new(self))
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

#[derive(Clone, Identifiable, Selectable, Debug, Serialize, Deserialize)]
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

impl IntoView for Property {
    fn into_view(self) -> View {
        View::Transparent(leptos::leptos_dom::Transparent::new(self))
    }
}

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
#[derive(Clone, Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::property)]
pub struct NewProperty {
    property_id: uuid::Uuid,
    property_name: String,
    //#[serde(skip_serializing)]
    property_password: String,
    property_email: String,
    property_phone: String,
}

impl NewProperty {
    pub fn new(
        property_id: uuid::Uuid,
        property_name: String,
        property_password: String,
        property_email: String,
        property_phone: String,
    ) -> Self {
        Self {
            property_id,
            property_name,
            property_password,
            property_email,
            property_phone,
        }
    }
}

#[derive(
    Queryable, Selectable, Identifiable, Associations, Debug, PartialEq, Serialize, Deserialize,
)]
#[diesel(primary_key(user_id))]
#[diesel(table_name = crate::schema::propertyusers)]
#[diesel(belongs_to(Property, foreign_key = property_id))]
pub struct PropertyUsers {
    user_id: i32,
    user_name: String,
    #[serde(skip_serializing)]
    user_password: String,
    user_role: i32,
    property_id: Uuid,
}

impl PropertyUsers {
    pub fn new(
        user_id: i32,
        user_name: String,
        user_password: String,
        user_role: i32,
        property_id: Uuid,
    ) -> Self {
        Self {
            user_id,
            user_name,
            user_password,
            user_role,
            property_id,
        }
    }
}

//TODO: this struct should only be accessable after property login.
#[derive(Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::propertyusers)]
pub struct NewPropertyUser {
    user_name: String,
    #[serde(skip_serializing)]
    user_password: String,
    user_role: i32,
    property_id: Uuid,
}

impl NewPropertyUser {
    pub fn new(
        user_name: String,
        user_password: String,
        user_role: i32,
        property_id: Uuid,
    ) -> Self {
        Self {
            user_name,
            user_password,
            user_role,
            property_id,
        }
    }
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
    role_id: i32,
    role_name: String,
}

impl Roles {
    pub fn new(role_id: i32, role_name: String) -> Self {
        Self { role_id, role_name }
    }

    /// Pass users id it will return role name for that id.
    pub fn get_role(user_role_id: i32, conn: SharedPooledConnection) -> anyhow::Result<i32> {
        use crate::schema::roles::dsl::{role_id, role_name, roles};
        Ok(roles
            .filter(role_id.eq(user_role_id))
            .select(role_id)
            .get_result(&mut conn.try_get().unwrap())
            .unwrap())
    }
}
