//! Minimal serializer — serialize framework state for SSR transfer and server actions.
//!
//! Uses a compact JSON-like format. No external dependencies (no serde).
//! Supports primitives, strings, Vec, Option, and tuples up to 4 elements.

/// Serialize a value to a compact string format.
pub fn serialize<T: Serialize>(value: &T) -> String {
    let mut buf = String::new();
    value.serialize_to(&mut buf);
    buf
}

/// Deserialize a value from the compact string format.
pub fn deserialize<T: Deserialize>(data: &str) -> Option<T> {
    let mut input = data.trim();
    T::deserialize_from(&mut input)
}

/// Trait for types that can be serialized by the framework.
pub trait Serialize {
    /// Serialize to writer.
    fn serialize_to(&self, writer: &mut impl std::fmt::Write);
}

/// Trait for types that can be deserialized by the framework.
pub trait Deserialize: Sized {
    /// Deserialize from reader.
    fn deserialize_from(input: &mut &str) -> Option<Self>;
}

// === Primitive types ===

impl Serialize for bool {
    fn serialize_to(&self, w: &mut impl std::fmt::Write) {
        let _ = write!(w, "{}", self);
    }
}

impl Deserialize for bool {
    fn deserialize_from(input: &mut &str) -> Option<Self> {
        let trimmed = input.trim_start();
        if trimmed.starts_with("true") {
            *input = &trimmed[4..];
            Some(true)
        } else if trimmed.starts_with("false") {
            *input = &trimmed[5..];
            Some(false)
        } else {
            None
        }
    }
}

impl Serialize for i32 {
    fn serialize_to(&self, w: &mut impl std::fmt::Write) {
        let _ = write!(w, "{}", self);
    }
}

impl Deserialize for i32 {
    fn deserialize_from(input: &mut &str) -> Option<Self> {
        let trimmed = input.trim_start();
        let end = trimmed
            .find(|c: char| !c.is_ascii_digit() && c != '-')
            .unwrap_or(trimmed.len());
        let val = trimmed[..end].parse().ok()?;
        *input = &trimmed[end..];
        Some(val)
    }
}

impl Serialize for i64 {
    fn serialize_to(&self, w: &mut impl std::fmt::Write) {
        let _ = write!(w, "{}", self);
    }
}

impl Deserialize for i64 {
    fn deserialize_from(input: &mut &str) -> Option<Self> {
        let trimmed = input.trim_start();
        let end = trimmed
            .find(|c: char| !c.is_ascii_digit() && c != '-')
            .unwrap_or(trimmed.len());
        let val = trimmed[..end].parse().ok()?;
        *input = &trimmed[end..];
        Some(val)
    }
}

impl Serialize for u32 {
    fn serialize_to(&self, w: &mut impl std::fmt::Write) {
        let _ = write!(w, "{}", self);
    }
}

impl Deserialize for u32 {
    fn deserialize_from(input: &mut &str) -> Option<Self> {
        let trimmed = input.trim_start();
        let end = trimmed
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(trimmed.len());
        let val = trimmed[..end].parse().ok()?;
        *input = &trimmed[end..];
        Some(val)
    }
}

impl Serialize for f64 {
    fn serialize_to(&self, w: &mut impl std::fmt::Write) {
        let _ = write!(w, "{}", self);
    }
}

impl Deserialize for f64 {
    fn deserialize_from(input: &mut &str) -> Option<Self> {
        let trimmed = input.trim_start();
        let end = trimmed
            .find(|c: char| !c.is_ascii_digit() && c != '-' && c != '.')
            .unwrap_or(trimmed.len());
        let val = trimmed[..end].parse().ok()?;
        *input = &trimmed[end..];
        Some(val)
    }
}

impl Serialize for String {
    fn serialize_to(&self, w: &mut impl std::fmt::Write) {
        let _ = write!(w, "\"{}\"", self.replace('\\', "\\\\").replace('"', "\\\""));
    }
}

impl Deserialize for String {
    fn deserialize_from(input: &mut &str) -> Option<Self> {
        let trimmed = input.trim_start();
        if !trimmed.starts_with('"') {
            return None;
        }
        let rest = &trimmed[1..];
        let mut result = String::new();
        let mut chars = rest.chars();
        let mut consumed = 1; // opening quote
        while let Some(c) = chars.next() {
            consumed += c.len_utf8();
            if c == '\\' {
                if let Some(next) = chars.next() {
                    consumed += next.len_utf8();
                    match next {
                        '"' => result.push('"'),
                        '\\' => result.push('\\'),
                        _ => {
                            result.push('\\');
                            result.push(next);
                        }
                    }
                }
            } else if c == '"' {
                *input = &trimmed[consumed..];
                return Some(result);
            } else {
                result.push(c);
            }
        }
        None
    }
}

impl Serialize for &str {
    fn serialize_to(&self, w: &mut impl std::fmt::Write) {
        let _ = write!(w, "\"{}\"", self.replace('\\', "\\\\").replace('"', "\\\""));
    }
}

// === Vec<T> ===

impl<T: Serialize> Serialize for Vec<T> {
    fn serialize_to(&self, w: &mut impl std::fmt::Write) {
        let _ = write!(w, "[");
        for (i, item) in self.iter().enumerate() {
            if i > 0 {
                let _ = write!(w, ",");
            }
            item.serialize_to(w);
        }
        let _ = write!(w, "]");
    }
}

impl<T: Deserialize> Deserialize for Vec<T> {
    fn deserialize_from(input: &mut &str) -> Option<Self> {
        let trimmed = input.trim_start();
        if !trimmed.starts_with('[') {
            return None;
        }
        *input = &trimmed[1..];
        let mut result = Vec::new();
        loop {
            let trimmed = input.trim_start();
            if trimmed.starts_with(']') {
                *input = &trimmed[1..];
                return Some(result);
            }
            let item = T::deserialize_from(input)?;
            result.push(item);
            let trimmed = input.trim_start();
            if trimmed.starts_with(',') {
                *input = &trimmed[1..];
            } else if trimmed.starts_with(']') {
                *input = &trimmed[1..];
                return Some(result);
            } else {
                return None;
            }
        }
    }
}

// === Option<T> ===

impl<T: Serialize> Serialize for Option<T> {
    fn serialize_to(&self, w: &mut impl std::fmt::Write) {
        match self {
            Some(v) => {
                let _ = write!(w, "some(");
                v.serialize_to(w);
                let _ = write!(w, ")");
            }
            None => {
                let _ = write!(w, "null");
            }
        }
    }
}

impl<T: Deserialize> Deserialize for Option<T> {
    fn deserialize_from(input: &mut &str) -> Option<Self> {
        let trimmed = input.trim_start();
        if trimmed.starts_with("null") {
            *input = &trimmed[4..];
            return Some(None);
        }
        if trimmed.starts_with("some(") {
            *input = &trimmed[5..];
            let val = T::deserialize_from(input)?;
            let trimmed = input.trim_start();
            if trimmed.starts_with(')') {
                *input = &trimmed[1..];
                return Some(Some(val));
            }
        }
        None
    }
}

// === Tuples ===

impl<A: Serialize, B: Serialize> Serialize for (A, B) {
    fn serialize_to(&self, w: &mut impl std::fmt::Write) {
        let _ = write!(w, "(");
        self.0.serialize_to(w);
        let _ = write!(w, ",");
        self.1.serialize_to(w);
        let _ = write!(w, ")");
    }
}

impl<A: Deserialize, B: Deserialize> Deserialize for (A, B) {
    fn deserialize_from(input: &mut &str) -> Option<Self> {
        let trimmed = input.trim_start();
        if !trimmed.starts_with('(') {
            return None;
        }
        *input = &trimmed[1..];
        let a = A::deserialize_from(input)?;
        let trimmed = input.trim_start();
        if !trimmed.starts_with(',') {
            return None;
        }
        *input = &trimmed[1..];
        let b = B::deserialize_from(input)?;
        let trimmed = input.trim_start();
        if !trimmed.starts_with(')') {
            return None;
        }
        *input = &trimmed[1..];
        Some((a, b))
    }
}

impl<A: Serialize, B: Serialize, C: Serialize> Serialize for (A, B, C) {
    fn serialize_to(&self, w: &mut impl std::fmt::Write) {
        let _ = write!(w, "(");
        self.0.serialize_to(w);
        let _ = write!(w, ",");
        self.1.serialize_to(w);
        let _ = write!(w, ",");
        self.2.serialize_to(w);
        let _ = write!(w, ")");
    }
}

impl<A: Deserialize, B: Deserialize, C: Deserialize> Deserialize for (A, B, C) {
    fn deserialize_from(input: &mut &str) -> Option<Self> {
        let trimmed = input.trim_start();
        if !trimmed.starts_with('(') {
            return None;
        }
        *input = &trimmed[1..];
        let a = A::deserialize_from(input)?;
        let trimmed = input.trim_start();
        if !trimmed.starts_with(',') {
            return None;
        }
        *input = &trimmed[1..];
        let b = B::deserialize_from(input)?;
        let trimmed = input.trim_start();
        if !trimmed.starts_with(',') {
            return None;
        }
        *input = &trimmed[1..];
        let c = C::deserialize_from(input)?;
        let trimmed = input.trim_start();
        if !trimmed.starts_with(')') {
            return None;
        }
        *input = &trimmed[1..];
        Some((a, b, c))
    }
}

impl<A: Serialize, B: Serialize, C: Serialize, D: Serialize> Serialize for (A, B, C, D) {
    fn serialize_to(&self, w: &mut impl std::fmt::Write) {
        let _ = write!(w, "(");
        self.0.serialize_to(w);
        let _ = write!(w, ",");
        self.1.serialize_to(w);
        let _ = write!(w, ",");
        self.2.serialize_to(w);
        let _ = write!(w, ",");
        self.3.serialize_to(w);
        let _ = write!(w, ")");
    }
}

impl<A: Deserialize, B: Deserialize, C: Deserialize, D: Deserialize> Deserialize for (A, B, C, D) {
    fn deserialize_from(input: &mut &str) -> Option<Self> {
        let trimmed = input.trim_start();
        if !trimmed.starts_with('(') {
            return None;
        }
        *input = &trimmed[1..];
        let a = A::deserialize_from(input)?;
        let trimmed = input.trim_start();
        if !trimmed.starts_with(',') {
            return None;
        }
        *input = &trimmed[1..];
        let b = B::deserialize_from(input)?;
        let trimmed = input.trim_start();
        if !trimmed.starts_with(',') {
            return None;
        }
        *input = &trimmed[1..];
        let c = C::deserialize_from(input)?;
        let trimmed = input.trim_start();
        if !trimmed.starts_with(',') {
            return None;
        }
        *input = &trimmed[1..];
        let d = D::deserialize_from(input)?;
        let trimmed = input.trim_start();
        if !trimmed.starts_with(')') {
            return None;
        }
        *input = &trimmed[1..];
        Some((a, b, c, d))
    }
}

// === Result<T, E> ===

impl<T: Serialize, E: Serialize> Serialize for Result<T, E> {
    fn serialize_to(&self, w: &mut impl std::fmt::Write) {
        match self {
            Ok(v) => {
                let _ = write!(w, "ok(");
                v.serialize_to(w);
                let _ = write!(w, ")");
            }
            Err(e) => {
                let _ = write!(w, "err(");
                e.serialize_to(w);
                let _ = write!(w, ")");
            }
        }
    }
}

impl<T: Deserialize, E: Deserialize> Deserialize for Result<T, E> {
    fn deserialize_from(input: &mut &str) -> Option<Self> {
        let trimmed = input.trim_start();
        if trimmed.starts_with("ok(") {
            *input = &trimmed[3..];
            let val = T::deserialize_from(input)?;
            let trimmed = input.trim_start();
            if trimmed.starts_with(')') {
                *input = &trimmed[1..];
                return Some(Ok(val));
            }
        } else if trimmed.starts_with("err(") {
            *input = &trimmed[4..];
            let val = E::deserialize_from(input)?;
            let trimmed = input.trim_start();
            if trimmed.starts_with(')') {
                *input = &trimmed[1..];
                return Some(Err(val));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_deserialize_bool() {
        assert_eq!(serialize(&true), "true");
        assert_eq!(serialize(&false), "false");
        assert_eq!(deserialize::<bool>("true"), Some(true));
        assert_eq!(deserialize::<bool>("false"), Some(false));
    }

    #[test]
    fn test_serialize_deserialize_i32() {
        assert_eq!(serialize(&42i32), "42");
        assert_eq!(serialize(&-7i32), "-7");
        assert_eq!(deserialize::<i32>("42"), Some(42));
        assert_eq!(deserialize::<i32>("-7"), Some(-7));
    }

    #[test]
    fn test_serialize_deserialize_f64() {
        assert_eq!(serialize(&3.14f64), "3.14");
        assert_eq!(deserialize::<f64>("3.14"), Some(3.14));
    }

    #[test]
    fn test_serialize_deserialize_string() {
        assert_eq!(serialize(&"hello".to_string()), "\"hello\"");
        assert_eq!(deserialize::<String>("\"hello\""), Some("hello".to_string()));
        assert_eq!(serialize(&"a\"b".to_string()), "\"a\\\"b\"");
        assert_eq!(deserialize::<String>("\"a\\\"b\""), Some("a\"b".to_string()));
    }

    #[test]
    fn test_serialize_deserialize_vec() {
        let v: Vec<i32> = vec![1, 2, 3];
        assert_eq!(serialize(&v), "[1,2,3]");
        assert_eq!(deserialize::<Vec<i32>>("[1,2,3]"), Some(v));
    }

    #[test]
    fn test_serialize_deserialize_option() {
        assert_eq!(serialize(&Some(42i32)), "some(42)");
        assert_eq!(serialize(&None::<i32>), "null");
        assert_eq!(deserialize::<Option<i32>>("some(42)"), Some(Some(42)));
        assert_eq!(deserialize::<Option<i32>>("null"), Some(None));
    }

    #[test]
    fn test_serialize_deserialize_tuple() {
        let t = (1i32, "hello".to_string());
        assert_eq!(serialize(&t), "(1,\"hello\")");
        assert_eq!(deserialize::<(i32, String)>("(1,\"hello\")"), Some(t));
    }

    #[test]
    fn test_serialize_deserialize_result() {
        let ok: Result<i32, String> = Ok(42);
        assert_eq!(serialize(&ok), "ok(42)");
        assert_eq!(deserialize::<Result<i32, String>>("ok(42)"), Some(Ok(42)));

        let err: Result<i32, String> = Err("fail".to_string());
        assert_eq!(serialize(&err), "err(\"fail\")");
        assert_eq!(deserialize::<Result<i32, String>>("err(\"fail\")"), Some(Err("fail".to_string())));
    }

    #[test]
    fn test_serialize_deserialize_empty_vec() {
        let v: Vec<i32> = vec![];
        assert_eq!(serialize(&v), "[]");
        assert_eq!(deserialize::<Vec<i32>>("[]"), Some(v));
    }

    #[test]
    fn test_serialize_deserialize_nested() {
        let data: (Vec<i32>, Option<String>) = (vec![1, 2], Some("hi".to_string()));
        let serialized = serialize(&data);
        assert_eq!(deserialize::<(Vec<i32>, Option<String>)>(&serialized), Some(data));
    }
}
