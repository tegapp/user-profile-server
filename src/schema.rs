table! {
    machines (id) {
        id -> Int4,
        user_id -> Int4,
        public_key -> Text,
        name -> Text,
        slug -> Text,
    }
}

table! {
    sessions (sid) {
        sid -> Text,
        sess -> Text,
        expire -> Timestamptz,
    }
}

table! {
    users (id) {
        id -> Int4,
        email -> Nullable<Text>,
        email_verified -> Bool,
        phone_number -> Nullable<Text>,
        phone_number_verified -> Bool,
        username -> Text,
        hashed_password -> Nullable<Text>,
    }
}

joinable!(machines -> users (user_id));

allow_tables_to_appear_in_same_query!(
    machines,
    sessions,
    users,
);
