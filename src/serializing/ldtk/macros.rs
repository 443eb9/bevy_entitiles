#[macro_export]
macro_rules! transfer_str {
    ($field_type:ident, $value_type:ident, $type_name:expr, $value:expr) => {
        if let Some(v) = $value {
            if let FieldValue::$field_type(s) = v {
                Some(FieldValue::$value_type(s))
            } else {
                return Err(A::Error::custom(format!("expected {}, got {:?}", $type_name, v)));
            }
        } else {
            None
        }
    };
}

#[macro_export]
macro_rules! transfer_enum {
    ($enum_type:ident, $type_name:expr, $name:expr, $value:expr) => {
        if let Some(v) = $value {
            if let FieldValue::String(s) = v {
                Some(FieldValue::$enum_type(LdtkEnum {
                    name: $name,
                    variant: s,
                }))
            } else {
                return Err(A::Error::custom(format!("expected {}, got {:?}", $type_name, v)));
            }
        } else {
            None
        }
    };
}

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
macro_rules! unwrap_field {
    ($field:ident, $field_name:expr) => {
        $field.ok_or_else(|| Error::missing_field($field_name))?
    };
}
