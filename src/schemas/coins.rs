diesel::table! {
    coins (id) {
        id -> Integer,
        uuid -> Text,
        symbol -> Text,
        name -> Text,
    }
}
