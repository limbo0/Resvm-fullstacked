use crate::error_template::ErrorTemplate;
use crate::models::{NewResv, Reservation};
use crate::SharedPooledConnection;
use axum::{
    extract::{Json, Path, Query, State},
    response::{Html, IntoResponse},
};
use chrono::NaiveDate;
use diesel::prelude::*;
use leptos::*;
use leptos_router::{ActionForm, MultiActionForm, Route, Router, Routes};
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

#[server(GetReservations, "/api", "GetJson", "get_all_reservations")]
async fn get_reservations(
    State(conn): State<SharedPooledConnection>,
    Path(pid): Path<Uuid>,
) -> Result<Vec<Reservation>, ServerFnError> {
    use crate::schema::reservation::dsl::{property_id, reservation, reservation_date};

    match reservation
        .filter(property_id.eq(pid))
        .filter(reservation_date.eq(chrono::offset::Local::now().date_naive()))
        .limit(5)
        .select(Reservation::as_select())
        .load(&mut conn.try_get().unwrap())
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

////TODO: Handle if the data passed in as "" empty string.
//#[server(AddUser, "/api", "Url", "add_user")]
//async fn add_user(user_name: String, user_role: String) -> Result<(), ServerFnError> {
//    use crate::schema::myusers::dsl::{myusers, name};
//    use diesel::*;
//    use ssr::establish_connection;
//
//    let mut db = establish_connection();
//
//    let new_user = NewUser {
//        name: user_name,
//        role: user_role,
//    };
//
//    match diesel::insert_into(myusers)
//        .values(&new_user)
//        .execute(&mut db)
//    {
//        Ok(_row) => Ok(()),
//        Err(e) => Err(ServerFnError::ServerError(e.to_string())),
//    }
//}
//
//#[server(DeleteUser, "/api", "Url", "delete_user")]
//async fn delete_user(user_id: i32) -> Result<(), ServerFnError> {
//    use crate::schema::myusers::dsl::{id, myusers};
//    use diesel::*;
//    use ssr::establish_connection;
//
//    let mut db = establish_connection();
//    match diesel::delete(myusers.filter(id.eq(user_id))).execute(&mut db) {
//        Ok(_row) => Ok(()),
//        Err(e) => Err(ServerFnError::ServerError(e.to_string())),
//    }
//}

#[component]
pub fn ResvmApp() -> impl IntoView {
    view! {
        <Router>
            <header>
                <h1>"This is the resv manager"</h1>
            </header>
            <main>
                <Routes>
                    <Route path="" view=Resvm/>
                </Routes>
            </main>

        </Router>
    }
}

#[component]
fn Resvm() -> impl IntoView {
    let add_user = create_server_multi_action::<todo!()>();
    let delete_user = create_server_action::<todo!()>();
    let submissions = add_user.submissions();

    let all_users = create_resource(
        move || (add_user.version().get(), delete_user.version().get()),
        move |_| get_reservations(),
    );
    view! {
        <div>

            <MultiActionForm action=add_user class="add_user_form">
                <div class="add_user_form">
                    <label>"Enter name:" <input type="text" name="user_name" required/></label>
                </div>
                <div class="add_user_form">
                    <label>"Enter role:" <input type="text" name="user_role" required/></label>
                </div>
                <div class="add_user_form">
                    <input type="submit" value="Add User"/>
                </div>
            </MultiActionForm>

            <Transition fallback=move || view! { <p>"Loading...."</p> }>
                <ErrorBoundary fallback=move |error| {
                    view! { <ErrorTemplate errors=error/> }
                }>

                    {move || {
                        let existing_users = {
                            move || {
                                all_users
                                    .get()
                                    .map(move |users| match users {
                                        Err(e) => {
                                            view! {
                                                <pre class="error">"Server Error: " {e.to_string()}</pre>
                                            }
                                                .into_view()
                                        }
                                        Ok(users) => {
                                            if users.is_empty() {
                                                view! { <p>"No users were found."</p> }.into_view()
                                            } else {
                                                users
                                                    .into_iter()
                                                    .map(move |users| {
                                                        view! {
                                                            <li>
                                                                {users.name} <ActionForm action=delete_user>
                                                                    <input type="hidden" name="user_id" value=users.id/>
                                                                    <input type="submit" value="X"/>
                                                                </ActionForm>
                                                            </li>
                                                        }
                                                    })
                                                    .collect_view()
                                            }
                                        }
                                    })
                                    .unwrap_or_default()
                            }
                        };
                        let pending_users = move || {
                            submissions
                                .get()
                                .into_iter()
                                .filter(|submission| submission.pending().get())
                                .map(|submission| {
                                    view! {
                                        <li class="pending">
                                            {move || {
                                                submission.input.get().map(|data| data.user_name)
                                            }}

                                        </li>
                                    }
                                })
                                .collect_view()
                        };
                        view! { <ul>{existing_users} {pending_users}</ul> }
                    }}

                </ErrorBoundary>
            </Transition>

        </div>
    }
}
