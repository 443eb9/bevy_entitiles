#[macro_export]
macro_rules! transfer_str {
    ($field_type:ident, $value_type:ident, $type_name:expr, $value:expr) => {
        if let Nullable::Data(v) = $value {
            if let FieldValue::$field_type(s) = v {
                Nullable::Data(FieldValue::$value_type(s))
            } else {
                return Err(A::Error::custom(format!(
                    "expected {}, got {:?}",
                    $type_name,
                    v
                )));
            }
        } else {
            Nullable::Null
        }
    }
}
