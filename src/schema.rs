// @generated automatically by Diesel CLI.

diesel::table! {
    visitors (id) {
        id -> Text,
        view_count -> Integer,
    }
}
