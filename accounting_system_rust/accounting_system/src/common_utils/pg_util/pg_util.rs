use chrono::{DateTime, SecondsFormat, Utc};
use chrono_tz::Tz;
use std::fmt::Write;

use uuid::Uuid;

pub trait ToPostgresString {
    fn fmt_postgres(&self, f: &mut String) -> std::fmt::Result;
    fn db_type_name(&self) -> &'static str;
}

impl ToPostgresString for &i32 {
    fn fmt_postgres(&self, f: &mut String) -> std::fmt::Result {
        write!(f, "{}", self)
    }

    fn db_type_name(&self) -> &'static str {
        "integer"
    }
}

impl ToPostgresString for i32 {
    fn fmt_postgres(&self, f: &mut String) -> std::fmt::Result {
        (&self).fmt_postgres(f)
    }

    fn db_type_name(&self) -> &'static str {
        (&self).db_type_name()
    }
}

impl ToPostgresString for &i16 {
    fn fmt_postgres(&self, f: &mut String) -> std::fmt::Result {
        write!(f, "{}::smallint", self)
    }

    fn db_type_name(&self) -> &'static str {
        "smallint"
    }
}

impl ToPostgresString for i16 {
    fn fmt_postgres(&self, f: &mut String) -> std::fmt::Result {
        (&self).fmt_postgres(f)
    }

    fn db_type_name(&self) -> &'static str {
        (&self).db_type_name()
    }
}

impl ToPostgresString for &i64 {
    fn fmt_postgres(&self, f: &mut String) -> std::fmt::Result {
        write!(f, "{}", self)
    }

    fn db_type_name(&self) -> &'static str {
        "bigint"
    }
}

impl ToPostgresString for i64 {
    fn fmt_postgres(&self, f: &mut String) -> std::fmt::Result {
        (&self).fmt_postgres(f)
    }

    fn db_type_name(&self) -> &'static str {
        (&self).db_type_name()
    }
}

impl ToPostgresString for &f32 {
    fn fmt_postgres(&self, f: &mut String) -> std::fmt::Result {
        write!(f, "{}", self)
    }

    fn db_type_name(&self) -> &'static str {
        "real"
    }
}

impl ToPostgresString for f32 {
    fn fmt_postgres(&self, f: &mut String) -> std::fmt::Result {
        (&self).fmt_postgres(f)
    }

    fn db_type_name(&self) -> &'static str {
        (&self).db_type_name()
    }
}

impl ToPostgresString for &f64 {
    fn fmt_postgres(&self, f: &mut String) -> std::fmt::Result {
        write!(f, "{}", self)
    }

    fn db_type_name(&self) -> &'static str {
        "double precision"
    }
}

impl ToPostgresString for f64 {
    fn fmt_postgres(&self, f: &mut String) -> std::fmt::Result {
        (&self).fmt_postgres(f)
    }

    fn db_type_name(&self) -> &'static str {
        (&self).db_type_name()
    }
}

impl ToPostgresString for &bool {
    fn fmt_postgres(&self, f: &mut String) -> std::fmt::Result {
        write!(f, "{}", self)
    }

    fn db_type_name(&self) -> &'static str {
        "bool"
    }
}

impl ToPostgresString for bool {
    fn fmt_postgres(&self, f: &mut String) -> std::fmt::Result {
        (&self).fmt_postgres(f)
    }

    fn db_type_name(&self) -> &'static str {
        (&self).db_type_name()
    }
}

impl ToPostgresString for &Uuid {
    fn fmt_postgres(&self, f: &mut String) -> std::fmt::Result {
        write!(f, "'{}'", self)
    }

    fn db_type_name(&self) -> &'static str {
        "uuid"
    }
}

impl ToPostgresString for Uuid {
    fn fmt_postgres(&self, f: &mut String) -> std::fmt::Result {
        (&self).fmt_postgres(f)
    }

    fn db_type_name(&self) -> &'static str {
        (&self).db_type_name()
    }
}

impl ToPostgresString for &str {
    fn fmt_postgres(&self, f: &mut String) -> std::fmt::Result {
        write!(f, "'{}'", self)
    }

    fn db_type_name(&self) -> &'static str {
        "text"
    }
}

impl ToPostgresString for String {
    fn fmt_postgres(&self, f: &mut String) -> std::fmt::Result {
        write!(f, "'{}'", self)
    }

    fn db_type_name(&self) -> &'static str {
        "text"
    }
}

impl ToPostgresString for DateTime<Tz> {
    fn fmt_postgres(&self, f: &mut String) -> std::fmt::Result {
        let representation = self.to_rfc3339_opts(SecondsFormat::Micros, false);
        write!(f, "'{}'", &representation)
    }

    fn db_type_name(&self) -> &'static str {
        "timestamp with timezone"
    }
}

impl ToPostgresString for DateTime<Utc> {
    fn fmt_postgres(&self, f: &mut String) -> std::fmt::Result {
        let representation = self.to_rfc3339_opts(SecondsFormat::Micros, false);
        write!(f, "'{}'", &representation)
    }

    fn db_type_name(&self) -> &'static str {
        "timestamp with timezone"
    }
}

impl<T: ToPostgresString> ToPostgresString for Option<T> {
    fn fmt_postgres(&self, f: &mut String) -> std::fmt::Result {
        match self {
            None => {
                write!(f, "null")
            }
            Some(a) => a.fmt_postgres(f),
        }
    }

    fn db_type_name(&self) -> &'static str {
        match self {
            None => {
                panic!("should not reach this code block")
            }
            Some(a) => a.db_type_name(),
        }
    }
}

fn fmt_postgres_slice<T: ToPostgresString>(slice: &[T], f: &mut String) -> std::fmt::Result {
    //this is a deliberate choice since fetching fn db_type_name() is not possible for empty vec
    if slice.is_empty() {
        return write!(f, "null");
    }
    write!(f, "array[")?;
    if let Some(first) = slice.first() {
        first.fmt_postgres(f)?;
    }
    for x in slice.iter().skip(1) {
        write!(f, ",")?;
        x.fmt_postgres(f)?;
    }
    write!(f, "]::{}[]", slice.db_type_name())
}

fn db_type_name_slice<T: ToPostgresString>(slice: &[T]) -> &'static str {
    if slice.is_empty() {
        panic!("cannot find db type name for empty list. this code should be unreachable")
    } else {
        slice[0].db_type_name()
    }
}

impl<T: ToPostgresString> ToPostgresString for Vec<T> {
    fn fmt_postgres(&self, f: &mut String) -> std::fmt::Result {
        fmt_postgres_slice(self, f)
    }

    fn db_type_name(&self) -> &'static str {
        db_type_name_slice(self)
    }
}

impl<T: ToPostgresString> ToPostgresString for &[T] {
    fn fmt_postgres(&self, f: &mut String) -> std::fmt::Result {
        fmt_postgres_slice(*self, f)
    }

    fn db_type_name(&self) -> &'static str {
        db_type_name_slice(*self)
    }
}

pub fn create_composite_type_db_row(
    fields: &[&dyn ToPostgresString],
    f: &mut String,
) -> std::fmt::Result {
    write!(f, "row(")?;
    if let Some(first) = fields.first() {
        first.fmt_postgres(f)?;
    }
    for x in fields.iter().skip(1) {
        write!(f, ",")?;
        x.fmt_postgres(f)?;
    }
    write!(f, ")")
}

#[cfg(test)]
mod tests {
    use crate::common_utils::pg_util::pg_util::{
        create_composite_type_db_row, db_type_name_slice, fmt_postgres_slice, ToPostgresString,
    };
    use rstest::rstest;
    use std::fmt::Write;
    use uuid::Uuid;

    struct P {
        id: Uuid,
        name: String,
        active: bool,
    }

    struct Nested {
        id: Uuid,
        solder: i32,
        ps: Vec<P>,
    }

    impl ToPostgresString for P {
        fn fmt_postgres(&self, f: &mut String) -> std::fmt::Result {
            let fields: &[&dyn ToPostgresString] = &[&self.id, &self.name.as_str(), &self.active];
            create_composite_type_db_row(fields, f)
        }

        fn db_type_name(&self) -> &'static str {
            "create_p"
        }
    }

    impl ToPostgresString for Nested {
        fn fmt_postgres(&self, f: &mut String) -> std::fmt::Result {
            let fields: &[&dyn ToPostgresString] = &[&self.id, &self.solder, &self.ps];
            create_composite_type_db_row(fields, f)
        }
        fn db_type_name(&self) -> &'static str {
            "create_nested"
        }
    }

    #[tokio::test]
    async fn te() {
        let p: P = P {
            id: Default::default(),
            name: "da".to_string(),
            active: false,
        };
        let p1: P = P {
            id: Default::default(),
            name: "d2a".to_string(),
            active: false,
        };
        let p3: P = P {
            id: Default::default(),
            name: "d3a".to_string(),
            active: false,
        };

        let mut str = String::with_capacity(52);
        p.fmt_postgres(&mut str).unwrap();
        assert_eq!(
            str,
            "row('00000000-0000-0000-0000-000000000000','da',false)"
        );
        let n: Nested = Nested {
            id: Default::default(),
            solder: 4,
            ps: vec![p, p1, p3],
        };
        let mut str = String::with_capacity(52);
        n.fmt_postgres(&mut str).unwrap();
        assert_eq!(
            str,
            "row('00000000-0000-0000-0000-000000000000',4,\
        array[row('00000000-0000-0000-0000-000000000000','da',false),\
        row('00000000-0000-0000-0000-000000000000','d2a',false),\
        row('00000000-0000-0000-0000-000000000000','d3a',false)]::create_p[])"
        );
    }

    #[rstest]
    #[case(42, "42", "integer")]
    #[case(Some(42), "42", "integer")]
    #[should_panic]
    #[case(None::< i32 >, "null", "integer")]
    #[case("hello", "'hello'", "text")]
    #[case(& 42, "42", "integer")]
    #[case(vec ! [1, 2, 3], "array[1,2,3]::integer[]", "integer")]
    fn test_fmt_postgres(
        #[case] input: impl ToPostgresString,
        #[case] expected_output: &'static str,
        #[case] expected_type_name: &'static str,
    ) {
        let mut output = String::new();
        input.fmt_postgres(&mut output).unwrap();
        assert_eq!(output, expected_output);
        assert_eq!(input.db_type_name(), expected_type_name);
    }

    #[derive(Debug, PartialEq)]
    struct TestType(i32);

    impl ToPostgresString for TestType {
        fn fmt_postgres(&self, f: &mut String) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }

        fn db_type_name(&self) -> &'static str {
            "int4"
        }
    }

    #[rstest]
    #[case(& [] as & [TestType], "null")]
    fn test_fmt_postgres_slice_empty(#[case] slice: &[TestType], #[case] expected: &str) {
        let mut result = String::new();
        fmt_postgres_slice(slice, &mut result).unwrap();
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case(& [TestType(42)], "array[42]::int4[]")]
    fn test_fmt_postgres_slice_single_element(#[case] slice: &[TestType], #[case] expected: &str) {
        let mut result = String::new();
        fmt_postgres_slice(slice, &mut result).unwrap();
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case(& [TestType(1), TestType(2), TestType(3)], "array[1,2,3]::int4[]")]
    fn test_fmt_postgres_slice_multiple_elements(
        #[case] slice: &[TestType],
        #[case] expected: &str,
    ) {
        let mut result = String::new();
        fmt_postgres_slice(slice, &mut result).unwrap();
        assert_eq!(result, expected);
    }

    #[rstest]
    #[should_panic(
        expected = "cannot find db type name for empty list. this code should be unreachable"
    )]
    fn test_db_type_name_slice_empty() {
        let empty_slice: &[TestType] = &[];
        db_type_name_slice(empty_slice);
    }

    #[rstest]
    #[case(& [TestType(1), TestType(2)], "int4")]
    fn test_db_type_name_slice_non_empty(#[case] slice: &[TestType], #[case] expected: &str) {
        assert_eq!(db_type_name_slice(slice), expected);
    }
}
