// @generated automatically by Diesel CLI.

diesel::table! {
    myusers (id) {
        id -> Int4,
        #[max_length = 35]
        name -> Varchar,
        #[max_length = 20]
        role -> Varchar,
    }
}

diesel::table! {
    property (property_id) {
        property_id -> Uuid,
        property_name -> Varchar,
        property_password -> Varchar,
        property_email -> Varchar,
        property_phone -> Varchar,
    }
}

diesel::table! {
    propertyusers (user_id) {
        user_id -> Int4,
        user_name -> Varchar,
        user_password -> Varchar,
        user_role -> Int4,
        property_id -> Uuid,
    }
}

diesel::table! {
    reservation (id) {
        id -> Int4,
        name -> Varchar,
        contact -> Varchar,
        seating -> Varchar,
        specific_seating_requested -> Bool,
        advance -> Bool,
        advance_method -> Jsonb,
        advance_amount -> Nullable<Int4>,
        confirmed -> Bool,
        reservation_date -> Date,
        reservation_time -> Time,
        property_id -> Uuid,
    }
}

diesel::table! {
    roles (role_id) {
        role_id -> Int4,
        #[max_length = 15]
        role_name -> Varchar,
    }
}

diesel::joinable!(propertyusers -> property (property_id));
diesel::joinable!(propertyusers -> roles (user_role));
diesel::joinable!(reservation -> property (property_id));

diesel::allow_tables_to_appear_in_same_query!(
    myusers,
    property,
    propertyusers,
    reservation,
    roles,
);
