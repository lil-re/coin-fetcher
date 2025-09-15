diesel::table! {
    coins (id) {
        id -> Integer,
        uuid -> Varchar,
        symbol -> Varchar,
        name -> Varchar,
    }
}
