// @generated automatically by Diesel CLI.

diesel::table! {
    password_reset_tokens (id) {
        id -> Int4,
        user_id -> Uuid,
        token -> Varchar,
        expires_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 100]
        name -> Varchar,
        #[max_length = 100]
        email -> Varchar,
        #[max_length = 255]
        password -> Varchar,
        address -> Text,
        #[max_length = 20]
        phoneno -> Varchar,
        #[max_length = 10]
        account_type -> Varchar,
        #[max_length = 255]
        profile_pic -> Nullable<Varchar>,
        created_at -> Timestamp,
    }
}

diesel::joinable!(password_reset_tokens -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(password_reset_tokens, users,);
