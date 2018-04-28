table! {
    devices (id) {
        id -> Integer,
        device_name -> Text,
        device_url -> Nullable<Text>,
        device_owner -> Nullable<Text>,
        comments -> Nullable<Text>,
        reservation_status -> ::models::ReservationStatusMapping,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}
