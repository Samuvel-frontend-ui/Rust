// @generated automatically by Diesel CLI.

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
