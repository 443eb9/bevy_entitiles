#[macro_export]
macro_rules! transfer_field {
    ($field:ident, $field_name:expr, $json_map:expr) => {{
        if $field.is_some() {
            return Err(A::Error::duplicate_field($field_name));
        }
        $field = Some($json_map.next_value()?);
    }};
}

#[macro_export]
macro_rules! match_field {
    ($deser_value:expr, $variant:ident, $value_type:ty, $json_map:expr) => {
        if let Some(v) = $json_map.next_value::<Option<$value_type>>()? {
            $deser_value = Some(FieldValue::$variant(v));
        } else {
            $deser_value = None;
        }
    };
}

#[macro_export]
macro_rules! match_field_enum {
    ($deser_value:expr, $variant:ident, $value_type:ty, $enum_type:expr, $json_map:expr) => {
        if let Some(v) = $json_map.next_value::<Option<$value_type>>()? {
            $deser_value = Some(FieldValue::$variant(($enum_type, v)));
        } else {
            $deser_value = None;
        }
    };
}

#[macro_export]
macro_rules! unwrap_field {
    ($field:ident, $field_name:expr) => {{
        $field.ok_or_else(|| Error::missing_field($field_name))?}
    };
}
