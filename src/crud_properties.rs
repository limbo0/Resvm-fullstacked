use crate::models::{NewProperty, NewResv, PaymentMethod, PaymentMode, Property, Reservation};
use crate::salt_password;
use crate::AppState;
use chrono::NaiveDate;
use diesel::prelude::*;
use leptos::{expect_context, server, Params, ServerFnError};
use leptos_router::Params;
use time::Time;
use uuid::Uuid;

#[server(AllProperties, "/api", "GetJson")]
pub async fn get_all_properties() -> Result<Vec<Property>, ServerFnError> {
    use crate::schema::property::dsl::{property, property_id};
    let state = expect_context::<AppState>();

    match property
        .select(Property::as_select())
        .load(&mut state.pool.try_get().unwrap())
    {
        Ok(_result) => Ok(_result),
        Err(e) => Err(ServerFnError::ServerError(e.to_string())),
    }
}

#[derive(PartialEq, Params, Debug)]
pub struct PropertyIdParam {
    pub pid: Option<Uuid>,
}

#[server(PropertyReservations, "/api", "GetJson")]
pub async fn get_property_reservations(pid: Uuid) -> Result<Vec<Reservation>, ServerFnError> {
    use crate::schema::reservation::dsl::{property_id, reservation, reservation_date};

    let state = expect_context::<AppState>();
    // let conn: AppState = extract_with_state(&state).await.unwrap();

    match reservation
        .filter(property_id.eq(pid))
        // .filter(reservation_date.eq(chrono::offset::Local::now().date_naive()))
        .limit(5)
        .select(Reservation::as_select())
        .load(&mut state.pool.try_get().unwrap())
    {
        Ok(_result) => Ok(_result),
        Err(e) => Err(ServerFnError::ServerError(e.to_string())),
    }
}
#[server(name = AddProperty, prefix = "/api", endpoint = "add_property", input = Json, output = Json, encoding = "Url", impl_from = true)]
pub async fn add_property(
    property_name: String,
    property_password: String,
    property_email: String,
    property_phone: String,
) -> Result<Uuid, ServerFnError> {
    use crate::schema::property::dsl::{property, property_id};

    let state = expect_context::<AppState>();

    let new_property = NewProperty::new(
        Uuid::new_v4(),
        property_name,
        salt_password(property_password)
            .await
            .expect("failed to salt property password"),
        property_email,
        property_phone,
    );

    match diesel::insert_into(property)
        .values(&new_property)
        .returning(property_id)
        .get_result::<Uuid>(&mut state.pool.try_get().unwrap())
    {
        Ok(_pid) => Ok(_pid),
        Err(e) => Err(ServerFnError::ServerError(e.to_string())),
    }
}

#[server(name = AddResv, prefix = "/api", endpoint = "add_resv", input = Json, output = Json, encoding = "Url", impl_from = true)]
pub async fn add_reservation(
    name: String,
    contact: String,
    seating: String,
    specific_seating_requested: bool,
    advance: bool,
    mode_of_payment: String,
    payment_transaction_id: Option<String>,
    payment_receiver: Option<String>,
    payment_received_date: Option<NaiveDate>,
    advance_amount: Option<i32>,
    confirmed: bool,
    reservation_date: NaiveDate,
    reservation_time: String,
    property_id: Uuid,
) -> Result<i32, ServerFnError> {
    use crate::schema::reservation::dsl::{id, reservation};

    let state = expect_context::<AppState>();

    let advance_method = serde_json::to_value(PaymentMethod::new(
        match mode_of_payment {
            mode_of_payment if mode_of_payment == String::from("NotPaid") => PaymentMode::NotPaid,
            mode_of_payment if mode_of_payment == String::from("Cash") => PaymentMode::Cash,
            mode_of_payment if mode_of_payment == String::from("Card") => PaymentMode::Card,
            mode_of_payment if mode_of_payment == String::from("Gpay") => PaymentMode::Gpay,
            _ => PaymentMode::NotPaid,
        },
        payment_transaction_id,
        payment_receiver,
        payment_received_date,
    ))
    .expect("failed to convert struct into json");

    // Constructing time with only needed properties.
    // In the front end the max length of input should be 4.
    let resvt = reservation_time.trim();
    let hour: u8 = resvt[..2].parse::<u8>()?;
    let min: u8 = resvt[2..4].parse::<u8>()?;
    let reservation_time = Time::from_hms(hour, min, 0u8)?;

    let new_resv = NewResv::new(
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
    );

    match diesel::insert_into(reservation)
        .values(&new_resv)
        .returning(id)
        .get_result::<i32>(&mut state.pool.try_get().unwrap())
    {
        Ok(rid) => Ok(rid),
        Err(e) => Err(ServerFnError::ServerError(e.to_string())),
    }
}

#[server(TotalResv, "/api", "Url")]
pub async fn total_resv(pid: Uuid) -> Result<i64, ServerFnError> {
    use crate::schema::reservation::dsl::{property_id, reservation};

    let state = expect_context::<AppState>();

    match reservation
        .filter(property_id.eq(pid))
        .count()
        .get_result(&mut state.pool.try_get().unwrap())
    {
        Ok(count) => Ok(count),
        Err(e) => Err(ServerFnError::ServerError(e.to_string())),
    }
}

// async fn add_reservation(user_name: String, user_role: String) -> Result<(), ServerFnError> {
//     use crate::schema::myusers::dsl::{myusers, name};
//     use diesel::*;
//     use ssr::establish_connection;
//
//     let mut db = establish_connection();
//
//     let new_user = NewUser {
//         name: user_name,
//         role: user_role,
//     };
//
//     match diesel::insert_into(myusers)
//         .values(&new_user)
//         .execute(&mut db)
//     { Ok(_row) => Ok(()),
//         Err(e) => Err(ServerFnError::ServerError(e.to_string())),
//     }
// }

// #[server(DeleteReservation, "/api", "Url", "delete_reservation")]
// async fn delete_reservation(user_id: i32) -> Result<(), ServerFnError> {
//     use crate::schema::myusers::dsl::{id, myusers};
//     use diesel::*;
//     use ssr::establish_connection;
//
//     let mut db = establish_connection();
//     match diesel::delete(myusers.filter(id.eq(user_id))).execute(&mut db) {
//         Ok(_row) => Ok(()),
//         Err(e) => Err(ServerFnError::ServerError(e.to_string())),
//     }
// }
