use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};

use crate::generic::Cow;
use crate::traits::internal::{Beef, Capacity};

impl<T, U> Serialize for Cow<'_, T, U>
where
    T: Beef + Serialize + ?Sized,
    U: Capacity,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        T::serialize(self.as_ref(), serializer)
    }
}

impl<'de, 'a, T: ?Sized, U> Deserialize<'de> for Cow<'a, T, U>
where
    T: Beef,
    U: Capacity,
    T::Owned: Deserialize<'de>,
{
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::Owned::deserialize(deserializer).map(Cow::owned)
    }
}

#[cfg(test)]
mod tests {
    use serde_derive::{Deserialize, Serialize};

    #[test]
    fn wide_cow_de() {
        use crate::Cow;

        #[derive(Serialize, Deserialize)]
        struct Test<'a> {
            #[serde(borrow)]
            foo: Cow<'a, str>,
            bar: Cow<'a, str>,
        }

        let json = r#"{"foo":"Hello","bar":"\tWorld!"}"#;
        let test: Test = serde_json::from_str(json).unwrap();

        assert_eq!(test.foo, "Hello");
        assert_eq!(test.bar, "\tWorld!");

        assert!(test.foo.is_borrowed());
        assert!(test.bar.is_owned());

        let out = serde_json::to_string(&test).unwrap();

        assert_eq!(json, out);
    }
}
