use crate::{
    crud_properties::{
        add_reservation, get_all_properties, get_property_reservations, total_resv, AddResv,
        PropertyIdParam,
    },
    models::NewResv,
};
use leptos::*;
use leptos_meta::{provide_meta_context, Meta, Stylesheet};
use leptos_router::{
    use_params, use_params_map, ActionForm, MultiActionForm, Outlet, Params, Route, Router, Routes,
    A,
};

#[component]
pub fn ResvmApp() -> impl IntoView {
    provide_meta_context();
    view! {
        <Stylesheet href="/pkg/resvm.css"/>
        <Router>
            <header>
                <h1>"This is the resv manager"</h1>
            </header>
            <nav></nav>
            <a href="/Property">"Properties"</a>
            <main>
                <Routes>
                    <Route path="/" view=|| view! { <h1>"Make this the homepage"</h1> }/>
                    <Route path="/Property" view=Properties>
                        <Route path=":pid" view=PropertyReservations>
                            <Route path="add_resv" view=AddReservation/>
                            <Route
                                path=""
                                view=|| {
                                    view! {
                                        <div>"Handle this inner route after add_resv error"</div>
                                    }
                                }
                            />

                        </Route>
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
fn AddReservation() -> impl IntoView {
    let params = use_params::<PropertyIdParam>();
    let pid = move || {
        params
            .with(|params| params.as_ref().map(|params| params.pid).unwrap_or_default())
            .expect("Failed to get pid.")
    };

    let action = create_server_action::<AddResv>();

    let resv_count = create_resource(
        move || action.version().get(),
        move |_| async move { total_resv(pid()).await },
    );

    view! {
        <ActionForm action class="container">
            <fieldset>
                <legend>Fill reservation data</legend>
                <div class="col">
                    <label>"Name: " <input type="text" name="name"/></label>
                    <label>"Contact: " <input type="text" name="contact"/></label>
                    <label>"Seating: " <input type="text" name="seating"/></label>

                    <fieldset>
                        <legend>specific seating</legend>
                        <label>
                            <input type="radio" name="specific_seating_requested" value="true"/>
                            true
                        </label>
                        <label>
                            <input type="radio" name="specific_seating_requested" value="false"/>
                            false
                        </label>
                    </fieldset>

                    <fieldset>
                        <legend>Advance</legend>
                        <label>
                            <input type="radio" name="advance" value="true"/>
                            true
                        </label>
                        <label>
                            <input type="radio" name="advance" value="false"/>
                            false
                        </label>
                    </fieldset>

                    <fieldset>
                        <legend>Mode of payment</legend>
                        <label>
                            <input type="radio" name="mode_of_payment" value="NotPaid"/>
                            NotPaid
                        </label>
                        <label>
                            <input type="radio" name="mode_of_payment" value="Cash"/>
                            Cash
                        </label>
                        <label>
                            <input type="radio" name="mode_of_payment" value="Card"/>
                            Card
                        </label>
                        <label>
                            <input type="radio" name="mode_of_payment" value="Gpay"/>
                            Gpay
                        </label>
                    </fieldset>

                    <fieldset>
                        <legend>Payment details</legend>
                        <label>
                            "Payment tx id: " <input type="text" name="payment_transaction_id"/>
                        </label>
                        <label>
                            "Payment receiver: " <input type="text" name="payment_receiver"/>
                        </label>
                        <label>
                            "Payment received date: "
                            <input type="date" name="payment_received_date"/>
                        </label>
                        <label>
                            "Advance amount: " <input type="number" name="advance_amount"/>
                        </label>
                    </fieldset>

                    <fieldset>
                        <legend>Date and time</legend>
                        <label>
                            "Confirmed: " <input type="checkbox" name="confirmed" value="true"/>
                        </label>
                        <label>
                            "Reservation date: " <input type="date" name="reservation_date"/>
                        </label>
                        <label>
                            "Reservation time: "
                            <input type="text" name="reservation_time" minlength="4" maxlength="4"/>
                        </label>
                        <label>"Property id: " <input type="text" name="property_id"/></label>
                    </fieldset>
                    <button>Submit</button>
                </div>
            </fieldset>
        </ActionForm>
        <p>You submitted: {move || format!("{:?}", action.input().get())}</p>
        <p>The result was: {move || format!("{:?}", action.value().get())}</p>
        <Transition>
            <p>Total number of reservations: {resv_count}</p>
        </Transition>
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
                                                        <table>
                                                            <thead>
                                                                <tr>
                                                                    <th>Id</th>
                                                                    <th>Name</th>
                                                                    <th>Contact</th>
                                                                    <th>Seating</th>
                                                                    <th>Specific seating</th>
                                                                    <th>Advance</th>
                                                                    <th>Mode of payment</th>
                                                                    <th>Payment tx Id</th>
                                                                    <th>Payment received date</th>
                                                                    <th>Advance amount</th>
                                                                    <th>Confirmed</th>
                                                                    <th>Reservation date</th>
                                                                    <th>Reservation time</th>
                                                                    <th>Property Id</th>
                                                                </tr>
                                                            </thead>

                                                            <tbody>
                                                                <tr>
                                                                    <td>
                                                                        <p>{resv.id}</p>
                                                                    </td>
                                                                    <td>
                                                                        <p>{resv.name}</p>
                                                                    </td>
                                                                    <td>
                                                                        <p>{resv.contact}</p>
                                                                    </td>
                                                                    <td>
                                                                        <p>{resv.seating}</p>
                                                                    </td>
                                                                    <td>
                                                                        <p>{resv.specific_seating_requested}</p>
                                                                    </td>
                                                                    <td>
                                                                        <p>{resv.advance}</p>
                                                                    </td>
                                                                    <td>
                                                                        {move || {
                                                                            view! {
                                                                                <td>
                                                                                    {String::from(
                                                                                        resv
                                                                                            .advance_method["mode_of_payment"]
                                                                                            .as_str()
                                                                                            .unwrap_or_else(|| "Payment not received"),
                                                                                    )}

                                                                                </td>
                                                                                <td>
                                                                                    {String::from(
                                                                                        resv
                                                                                            .advance_method["payment_transaction_id"]
                                                                                            .as_str()
                                                                                            .unwrap_or_else(|| "Not transaction id"),
                                                                                    )}

                                                                                </td>
                                                                                <td>
                                                                                    {String::from(
                                                                                        resv
                                                                                            .advance_method["payment_received_date"]
                                                                                            .as_str()
                                                                                            .unwrap_or_else(|| "Payment not reveived"),
                                                                                    )}

                                                                                </td>
                                                                            }
                                                                        }}

                                                                    </td>
                                                                    <td>
                                                                        {move || {
                                                                            if let Some(amount) = resv.advance_amount {
                                                                                if amount <= 0 { 0 } else { amount }
                                                                            } else {
                                                                                0
                                                                            }
                                                                        }}

                                                                    </td>
                                                                    <td>
                                                                        <p>{resv.confirmed}</p>
                                                                    </td>
                                                                // <p>{resv.reservation_date}</p>
                                                                // <p>{resv.reservation_time}</p>
                                                                </tr>
                                                            </tbody>
                                                        </table>
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
// };
