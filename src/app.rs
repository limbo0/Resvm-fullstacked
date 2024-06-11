use crate::error_template::ErrorTemplate;
use crate::models::{NewResv, Property, Reservation};
use crate::AppState;
use crate::SharedPooledConnection;
use axum::{
    extract::{Json, Path, Query, State},
    response::{Html, IntoResponse},
};
use chrono::NaiveDate;
use diesel::prelude::*;
use leptos::*;
use leptos_axum::extract_with_state;
use leptos_router::{
    use_params, use_params_map, ActionForm, MultiActionForm, Outlet, Params, Route, Router, Routes,
    A,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod ssr {
    // use http::{header::SET_COOKIE, HeaderMap, HeaderValue, StatusCode};
    use diesel::pg::PgConnection;
    use diesel::prelude::*;
    use dotenvy::dotenv;
    use std::env;

    pub fn establish_connection() -> PgConnection {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        PgConnection::establish(&database_url)
            .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
    }
}

#[server(AllProperties, "/api", "GetJson", "all_properties")]
async fn get_all_properties() -> Result<Vec<Property>, ServerFnError> {
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
    pid: Option<Uuid>,
}
#[server(PropertyReservations, "/api", "GetJson", "pid")]
async fn get_property_reservations(pid: Uuid) -> Result<Vec<Reservation>, ServerFnError> {
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

// async fn get_users() -> Result<Vec<MyUsers>, ServerFnError> {
//     use crate::schema::myusers::dsl::myusers;
//     use diesel::*;
//     use http::request::Parts;
//     use ssr::establish_connection;
//
//     // this is just an example of how to access server context injected in the handlers
//     let req_parts = use_context::<Parts>();
//     if let Some(req_parts) = req_parts {
//         println!("getting Uri = {:?}", req_parts.uri);
//     }
//     let mut db = establish_connection();
//
//     match myusers.select(MyUsers::as_select()).load(&mut db) {
//         Ok(_result) => Ok(_result),
//         Err(e) => Err(ServerFnError::ServerError(e.to_string())),
//     }
// }

// #[server(AddReservation, "/api", "Url", "add_reservation")]
// async fn add_reservation() -> Result<Vec<i32>, ServerFnError> {
//     //TODO: Send the necessary data to the provided contacts.
//     use crate::schema::reservation::dsl::{id, reservation};
//     let new_resv = NewResv {
//         name: payload.name,
//         contact: payload.contact,
//         seating: payload.seating,
//         specific_seating_requested: payload.specific_seating_requested,
//         advance: payload.advance,
//         advance_method: payload.advance_method,
//         advance_amount: payload.advance_amount,
//         confirmed: payload.confirmed,
//         reservation_date: payload.reservation_date,
//         reservation_time: payload.reservation_time,
//         property_id: payload.property_id,
//     };
//
//     match diesel::insert_into(reservation)
//         .values(&new_resv)
//         .returning(id)
//         .get_results::<i32>(&mut conn.try_get().unwrap())
//     {
//         Ok(_id) => Ok(_id),
//         Err(e) => Err(ServerFnError::ServerError(e.to_string())),
//     }
// }

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
//     {
//         Ok(_row) => Ok(()),
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

#[component]
pub fn ResvmApp() -> impl IntoView {
    view! {
        <Router>
            <header>
                <h1>"This is the resv manager"</h1>
            </header>
            <nav></nav>
            <a href="/Property">"Properties"</a>
            <main>
                <Routes>
                    <Route path="/" view=|| view! { <h1>"The first page"</h1> }/>
                    <Route path="/Property" view=Properties>
                        <Route path=":pid" view=PropertyReservations/>
                        // if no pid specified, fall back
                        <Route
                            path=""
                            view=|| {
                                view! {
                                    <div class="select-property">
                                        "Select a property to view resv info."
                                    </div>
                                }
                            }
                        />

                    </Route>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn PropertyReservations() -> impl IntoView {
    let params = use_params::<PropertyIdParam>();
    let pid = move || {
        params
            .with(|params| params.as_ref().map(|params| params.pid).unwrap_or_default())
            .expect("Failed to get pid.")
    };

    // Maybe use spawn_local?
    let reservations = create_resource(
        || (),
        move |_| async move { get_property_reservations(pid()).await },
    );

    view! {
        <div>
            <Suspense fallback=move || view! { <p>"Loading data"</p> }>
                <ErrorBoundary fallback=|_| {
                    view! { <p>"Something went wrong"</p> }
                }>
                    {move || {
                        reservations
                            .get()
                            .map(move |vec_resv| match vec_resv {
                                Err(e) => {
                                    view! {
                                        <pre class="error">"Server Error: " {e.to_string()}</pre>
                                    }
                                        .into_view()
                                }
                                Ok(vecc_resv) => {
                                    if vecc_resv.is_empty() {
                                        view! { <p>"No reservations were found."</p> }.into_view()
                                    } else {
                                        vecc_resv
                                            .into_iter()
                                            .map(|resv| {
                                                view! {
                                                    <div>
                                                        <li>
                                                            <p>{resv.id}</p>
                                                            <p>{resv.name}</p>
                                                            <p>{resv.contact}</p>
                                                            <p>{resv.seating}</p>
                                                            <p>{resv.specific_seating_requested}</p>
                                                            <p>{resv.advance}</p>
                                                            // <p>{resv.advance_method}</p>
                                                            <p>{resv.advance_amount}</p>
                                                            <p>{resv.confirmed}</p>
                                                            // <p>{resv.reservation_date}</p>
                                                            // <p>{resv.reservation_time}</p>
                                                            <p>{String::from(resv.property_id)}</p>
                                                        </li>
                                                    </div>
                                                }
                                            })
                                            .collect_view()
                                    }
                                }
                            })
                    }}

                </ErrorBoundary>
            </Suspense>
            <Outlet/>
        </div>
    }
}

#[component]
fn Properties() -> impl IntoView {
    let all_properties = create_resource(
        || (),
        |_| async move { get_all_properties().await.unwrap() },
    );

    view! {
        <div>
            <Suspense fallback=move || view! { <p>"Loading data"</p> }>
                // handles the error from the resource
                <ErrorBoundary fallback=|_| {
                    view! { <p>"Something went wrong while fetching all properties"</p> }
                }>
                    {move || {
                        all_properties
                            .get()
                            .map(move |properties| {
                                properties
                                    .into_iter()
                                    .map(move |property| {
                                        view! {
                                            <li>
                                                <A href=String::from(
                                                    property.property_id,
                                                )>{move || String::from(property.property_id)}</A>
                                            </li>
                                        }
                                    })
                                    .collect_view()
                            })
                    }}

                </ErrorBoundary>
            </Suspense>
            <Outlet/>
        </div>
    }
}

// #[component]
// fn Resvm() -> impl IntoView {
//     let add_reservation = create_server_multi_action::<AddReservation>();
//     let delete_reservation = create_server_action::<DeleteReservation>();
//     let submissions = add_reservation.submissions();
//
//     let all_users = create_resource(
//         move || {
//             (
//                 add_reservation.version().get(),
//                 delete_reservation.version().get(),
//             )
//         },
//         move |_| get_property_reservations(),
//     );
//     view! {
//         <div>
//
//             <MultiActionForm action=add_reservation class="add_user_form">
//                 <div class="add_user_form">
//                     <label>"Enter name:" <input type="text" name="user_name" required/></label>
//                 </div>
//                 <div class="add_user_form">
//                     <label>"Enter role:" <input type="text" name="user_role" required/></label>
//                 </div>
//                 <div class="add_user_form">
//                     <input type="submit" value="Add User"/>
//                 </div>
//             </MultiActionForm>
//
//             <Transition fallback=move || view! { <p>"Loading...."</p> }>
//                 <ErrorBoundary fallback=move |error| {
//                     view! { <ErrorTemplate errors=error/> }
//                 }>
//
//                     {move || {
//                         let existing_users = {
//                             move || {
//                                 all_users
//                                     .get()
//                                     .map(move |users| match users {
//                                         Err(e) => {
//                                             view! {
//                                                 <pre class="error">"Server Error: " {e.to_string()}</pre>
//                                             }
//                                                 .into_view()
//                                         }
//                                         Ok(users) => {
//                                             if users.is_empty() {
//                                                 view! { <p>"No users were found."</p> }.into_view()
//                                             } else {
//                                                 users
//                                                     .into_iter()
//                                                     .map(move |users| {
//                                                         view! {
//                                                             <li>
//                                                                 {users.name} <ActionForm action=delete_reservation>
//                                                                     <input type="hidden" name="user_id" value=users.id/>
//                                                                     <input type="submit" value="X"/>
//                                                                 </ActionForm>
//                                                             </li>
//                                                         }
//                                                     })
//                                                     .collect_view()
//                                             }
//                                         }
//                                     })
//                                     .unwrap_or_default()
//                             }
//                         };
//                         let pending_users = move || {
//                             submissions
//                                 .get()
//                                 .into_iter()
//                                 .filter(|submission| submission.pending().get())
//                                 .map(|submission| {
//                                     view! {
//                                         <li class="pending">
//                                             {move || {
//                                                 submission.input.get().map(|data| data.user_name)
//                                             }}
//
//                                         </li>
//                                     }
//                                 })
//                                 .collect_view()
//                         };
//                         view! { <ul>{existing_users} {pending_users}</ul> }
//                     }}
//
//                 </ErrorBoundary>
//             </Transition>
//
//         </div>
//     }
// }
