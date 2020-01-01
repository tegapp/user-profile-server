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
    users (id) {
        id -> Int4,
        auth0_id -> Text,
    }
}

joinable!(machines -> users (user_id));

allow_tables_to_appear_in_same_query!(
    machines,
    users,
);
