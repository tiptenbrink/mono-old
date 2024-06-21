//! # `lazy-borink`
//! `lazy-borink` exposes one struct, [Lazy], and a trait, [UnwrapLazy]. See [Lazy] for additional information.

use base64::{engine::general_purpose as b64, Engine as _};
#[cfg(feature = "derive")]
pub use lazy_borink_derive::UnwrapLazy;
pub use rmp_serde::decode::Error;
use serde::de::{self, DeserializeOwned, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{self, Debug};
use std::marker::PhantomData;

/// `Lazy` is a lazy (de)serialization type. It only serializes its data when the entire struct its contained in is serialized and only deserializes into
/// its inner type when specifically asked to do so.
///
/// Consider some information about a client. You might want to send this to the client for storage and retrieve it at a later date. However, the struct contains some data only
/// useful to the server. An example:
/// ```
/// # struct ServerData {}
/// struct ClientInfo {
///     client_id: String,
///     server_data: ServerData
/// }
/// ```
///
/// You don't want to have to expose the inner structure of ServerData in your public API. But if you serialize it to e.g. JSON, the structure is easily visible to the client.
/// The client might also need to define the structure if they want to conveniently deserialize it. Instead, wrap ServerData in `Lazy`:
/// ```
/// # use serde::{Deserialize, Serialize};
/// # #[derive(Serialize, Deserialize)]
/// # struct ServerData {}
/// use lazy_borink::Lazy;
///
/// #[derive(Serialize, Deserialize)]
/// struct ClientInfo {
///     client_id: String,
///     server_data: Lazy<ServerData>
/// }
/// ```
///
/// `Lazy` has custom implementations for [serde::Serialize] and [serde::Deserialize] to make this all possible. It is compatible with both JSON and MessagePack and has tests for both.
///
/// Example serialization:
///
/// ```
/// # use serde::{Deserialize, Serialize};
/// # #[derive(Serialize, Deserialize)]
/// # struct ServerData { data: String }
/// # use lazy_borink::Lazy;
/// # #[derive(Serialize, Deserialize)]
/// # struct ClientInfo {
/// #     client_id: String,
/// #     server_data: Lazy<ServerData>
/// # }
/// // ServerData can be anything, as long as it implements Serialize
/// let server_data = ServerData { data: "some_data".to_owned() };
/// let client_info = ClientInfo { client_id: "some_id".to_owned(), server_data: Lazy::from_inner(server_data) };
/// let client_json = serde_json::to_string(&client_info).unwrap();
///
/// assert_eq!(client_json, "{\"client_id\":\"some_id\",\"server_data\":\"gaRkYXRhqXNvbWVfZGF0YQ\"}");
/// ```
///
/// Instantiating a Lazy type is free, it doesn't yet do any serialization. But if we then serialize and deserialize, it will still have the original bytes.
/// However, `Lazy<T>` implements the trait [UnwrapLazy], allowing us to call [UnwrapLazy::unwrap_lazy] to get the structured data.
///
/// ```
/// # use serde::{Deserialize, Serialize};
/// # #[derive(Serialize, Deserialize, Debug)]
/// # struct ServerData { data: String }
/// # use lazy_borink::Lazy;
/// # let server_data = ServerData { data: "some_data".to_owned() };
/// use lazy_borink::UnwrapLazy;
///
/// let lazy = Lazy::from_inner(server_data);
/// assert_eq!(format!("{:?}", lazy), "ServerData { data: \"some_data\" }");
/// let bytes = rmp_serde::encode::to_vec_named(&lazy).unwrap();
/// let decoded: Lazy<ServerData> = rmp_serde::decode::from_slice(&bytes).unwrap();
/// assert_eq!(format!("{:?}", decoded), "Lazy { bytes: [129, 164, 100, 97, 116, 97, 169, 115, 111, 109, 101, 95, 100, 97, 116, 97] }");
/// let structured = decoded.unwrap_lazy();
/// assert_eq!(format!("{:?}", structured), "ServerData { data: \"some_data\" }");
/// ```
///
/// You can also automatically derive [UnwrapLazy] (this is gated behind the "derive" feature, which is enabled by default).
///
/// Other convenience methods are also available on the `Lazy` type, in case you only want the serialized binary data or just the inner structured type.
/// See [Lazy::inner], [Lazy::take], [Lazy::bytes], [Lazy::take_bytes].
///
/// Note that `Lazy`` assumes a correct implementation of Deserialize and Serialize for the inner structs. If one has to deal with possible incorrect data,
/// there is also [Lazy::try_inner], [Lazy::try_take] and [UnwrapLazy::try_unwrap_lazy], which return a Result with [rmp_serde::decode::Error].
/// Furthermore, it is always true that only one of bytes or inner ever holds a value, never both or neither.
#[derive(Clone, PartialEq, Eq)]
pub struct Lazy<T> {
    bytes: Option<Vec<u8>>,
    inner: Option<T>,
}

impl<T> Debug for Lazy<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(inner) = &self.inner {
            write!(f, "{:?}", inner)?;
        } else if let Some(bytes) = &self.bytes {
            f.debug_struct("Lazy").field("bytes", bytes).finish()?;
        }

        Ok(())
    }
}

fn deserialize_lazy<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, Error> {
    rmp_serde::decode::from_slice(bytes)
}

fn serialize_lazy<T: Serialize>(inner: &T) -> Vec<u8> {
    rmp_serde::encode::to_vec_named(inner).unwrap()
}

impl<T> Lazy<T> {
    /// Instantiates a Lazy type with bytes. Does not do any (de)serialization.
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Lazy {
            bytes: Some(bytes),
            inner: None,
        }
    }

    /// Instantiates a Lazy type with the inner type. Does not do any (de)serialization.
    pub fn from_inner(inner: T) -> Self {
        Lazy {
            bytes: None,
            inner: Some(inner),
        }
    }

    /// Returns `true` if the inner type is already deserialized. If this is the case, [Lazy::inner] and [Lazy::take] can be safely called.
    pub fn is_deserialized(&self) -> bool {
        self.inner.is_some()
    }
}

impl<T> Lazy<T>
where
    T: DeserializeOwned,
{
    /// Try to get a reference to the inner value. If it is already in its deserialized state, it will simply return a reference. Otherwise,
    /// it will try to deserialize the binary data to type `T`, discarding the binary data in the process. If it is unable to deserialize, it
    /// returns the [rmp_serde::decode::Error].
    pub fn try_inner(&mut self) -> Result<&T, Error> {
        if let Some(bytes) = self.bytes.take() {
            let inner = deserialize_lazy(&bytes)?;
            self.inner = Some(inner);

            self.try_inner()
        } else if let Some(inner) = &self.inner {
            Ok(inner)
        } else {
            panic!("Lazy structure is invalid, it contains no data!")
        }
    }

    /// Similar to [Lazy::try_inner], but moves the data out of the original Lazy type and returns the owned `T`.
    pub fn try_take(self) -> Result<T, Error> {
        if let Some(bytes) = self.bytes {
            deserialize_lazy(&bytes)
        } else if let Some(inner) = self.inner {
            Ok(inner)
        } else {
            panic!("Lazy structure is invalid, it contains no data!")
        }
    }

    /// Convenience function that unwraps the result from [Lazy::try_inner]. Panics if deserialization failed. Can be safely called if already in a deserialized state.
    pub fn inner(&mut self) -> &T {
        self.try_inner().unwrap()
    }

    /// Convenience function that unwraps the result from [Lazy::try_take]. Panics if deserialization failed. Can be safely called if already in a deserialized state.
    pub fn take(self) -> T {
        self.try_take().unwrap()
    }
}

impl<T> Lazy<T>
where
    T: Serialize,
{
    /// If in a deserialized state, it serializes the inner data using MessagePack and discards the inner T. It stores the serialized data in itself and returns a reference.
    /// It assumes the Serialize implementation is correct and does not fail. If it is not in a deserialized state, it will simply return a reference to the inner bytes.
    pub fn bytes(&mut self) -> &[u8] {
        if let Some(inner) = self.inner.take() {
            let bytes = serialize_lazy(&inner);
            self.bytes = Some(bytes);

            self.bytes()
        } else if let Some(bytes) = &self.bytes {
            bytes
        } else {
            panic!("Lazy structure is invalid, it contains no data!")
        }
    }

    /// Similar to [Lazy::take_bytes], but moves the data out of the `Lazy` and returns the owned bytes.
    pub fn take_bytes(self) -> Vec<u8> {
        if let Some(inner) = self.inner {
            serialize_lazy(&inner)
        } else if let Some(bytes) = self.bytes {
            bytes
        } else {
            panic!("Lazy structure is invalid, it contains no data!")
        }
    }
}

impl<'de, T> Deserialize<'de> for Lazy<T>
where
    T: DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct LazyVisitor<T>
        where
            T: DeserializeOwned,
        {
            _marker: PhantomData<T>,
        }

        impl<'de, T> Visitor<'de> for LazyVisitor<T>
        where
            T: DeserializeOwned,
        {
            type Value = Lazy<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a byte array or str")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match b64::URL_SAFE_NO_PAD.decode(v) {
                    Ok(data) => Ok(Lazy::from_bytes(data)),
                    Err(_) => Err(E::custom("not valid base64url without padding")),
                }
            }

            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Lazy::from_bytes(v))
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Lazy::from_bytes(v.to_vec()))
            }
        }

        if deserializer.is_human_readable() {
            deserializer.deserialize_str(LazyVisitor {
                _marker: PhantomData,
            })
        } else {
            deserializer.deserialize_byte_buf(LazyVisitor {
                _marker: PhantomData,
            })
        }
    }
}

impl<T> Serialize for Lazy<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(data) = &self.bytes {
            if serializer.is_human_readable() {
                let s = b64::URL_SAFE_NO_PAD.encode(data);
                serializer.serialize_str(&s)
            } else {
                serializer.serialize_bytes(data)
            }
        } else if let Some(inner) = &self.inner {
            let data = rmp_serde::encode::to_vec_named(inner).unwrap();

            if serializer.is_human_readable() {
                let s = b64::URL_SAFE_NO_PAD.encode(data);
                serializer.serialize_str(&s)
            } else {
                serializer.serialize_bytes(&data)
            }
        } else {
            panic!("Lazy structure is invalid, it contains no data!")
        }
    }
}

/// `unwrap_lazy()` should fully deserialize any [Lazy] child fields by calling `unwrap_lazy()` on all fields. After calling this once, calling this on subfields should be cheap.
/// This crate provides a procedural macro to automatically derive `UnwrapLazy`. It has also been implemented for many primitive types.
///
/// An example:
/// ```
/// # use serde::{Deserialize, Serialize};
/// use lazy_borink::{Lazy, UnwrapLazy};
///
/// #[derive(Serialize, Deserialize)]
/// struct Claims {
///     my_claim: String,
/// }
/// #[derive(Deserialize, Serialize, UnwrapLazy)]
/// struct User {
///     user_id: String,
///     password_file: String,
///     claims: Lazy<Claims>,
/// }
/// #[derive(Deserialize, Serialize, UnwrapLazy)]
/// struct UserMeta {
///     user: User,
/// }
/// ```
///
/// The tests provide some additional usage examples.
pub trait UnwrapLazy {
    fn unwrap_lazy(self) -> Self
    where
        Self: Sized,
    {
        self.try_unwrap_lazy().unwrap()
    }

    fn try_unwrap_lazy(self) -> Result<Self, Error>
    where
        Self: Sized;
}

impl<T> UnwrapLazy for Lazy<T>
where
    T: DeserializeOwned,
{
    fn unwrap_lazy(self) -> Self {
        Self::from_inner(self.take())
    }

    fn try_unwrap_lazy(self) -> Result<Self, Error> {
        Ok(Self::from_inner(self.try_take()?))
    }
}

impl<'a, T> UnwrapLazy for &'a T
where
    T: UnwrapLazy,
{
    fn try_unwrap_lazy(self) -> Result<Self, Error> {
        Ok(self)
    }
}

impl<'a, T> UnwrapLazy for &'a mut T
where
    T: UnwrapLazy,
{
    fn try_unwrap_lazy(self) -> Result<Self, Error> {
        Ok(self)
    }
}

impl<'a, T> UnwrapLazy for &'a [T]
where
    T: UnwrapLazy,
{
    fn try_unwrap_lazy(self) -> Result<Self, Error> {
        Ok(self)
    }
}

impl<'a, T> UnwrapLazy for &'a mut [T]
where
    T: UnwrapLazy,
{
    fn try_unwrap_lazy(self) -> Result<Self, Error> {
        Ok(self)
    }
}

impl<T> UnwrapLazy for Vec<T>
where
    T: UnwrapLazy,
{
    fn try_unwrap_lazy(self) -> Result<Self, Error> {
        Ok(self)
    }
}

impl<T> UnwrapLazy for Box<T>
where
    T: UnwrapLazy,
{
    fn try_unwrap_lazy(self) -> Result<Self, Error> {
        Ok(self)
    }
}

macro_rules! impl_unwrap_lazy_for_primitives {
    ($($t:ty),*) => {
        $(
            impl UnwrapLazy for $t {
                fn try_unwrap_lazy(self) -> Result<Self, Error> {
                    Ok(self)
                }
            }
        )*
    };
}

impl_unwrap_lazy_for_primitives!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64, bool, char, String
);

#[cfg(test)]
mod test {
    use super::*;

    // Assuming Claims is another struct you want to deserialize lazily.
    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    struct Claims {
        my_claim: String,
    }

    #[derive(Deserialize, Serialize, Debug, Clone, PartialEq, UnwrapLazy)]
    struct User {
        user_id: String,
        password_file: String,
        claims: Lazy<Claims>,
    }

    #[derive(Deserialize, Serialize, Debug, PartialEq, UnwrapLazy)]
    struct UserMeta {
        user: User,
    }

    #[test]
    fn lazy_unwrap() {
        let claims = Claims {
            my_claim: "hi".to_owned(),
        };

        let lazy_claims = Lazy::from_inner(claims.clone());
        let user = User {
            user_id: "abc".to_owned(),
            password_file: "hi".to_owned(),
            claims: lazy_claims,
        };

        assert_eq!(
            user.claims,
            Lazy {
                inner: Some(claims.clone()),
                bytes: None
            }
        );
        //println!("user:\n{:?}", user);

        let user_json = serde_json::to_string(&user).unwrap();

        assert_eq!(
            user_json,
            "{\"user_id\":\"abc\",\"password_file\":\"hi\",\"claims\":\"gahteV9jbGFpbaJoaQ\"}"
        );
        //println!("json repr\n{}", user_json);

        let claims_bytes = serialize_lazy(&claims);
        let user: User = serde_json::from_str(&user_json).unwrap();
        assert_eq!(
            user.claims,
            Lazy {
                inner: None,
                bytes: Some(claims_bytes.clone())
            }
        );
        //println!("user:\n{:?}", user);

        assert_eq!(
            user.clone().claims.unwrap_lazy(),
            Lazy {
                inner: Some(claims.clone()),
                bytes: None
            }
        );
        assert_eq!(user.clone().claims.take(), claims);
        let mut user_mut = user.clone();
        assert_eq!(user_mut.claims.inner(), &claims);
        assert_eq!(
            user_mut.claims,
            Lazy {
                inner: Some(claims.clone()),
                bytes: None
            }
        );

        let user_m = UserMeta { user };
        assert_eq!(
            user_m.user.claims,
            Lazy {
                inner: None,
                bytes: Some(claims_bytes.clone())
            }
        );

        let user_m = user_m.unwrap_lazy();
        assert_eq!(
            user_m.user.unwrap_lazy().claims,
            Lazy {
                inner: Some(claims.clone()),
                bytes: None
            }
        );

        //println!("user_m:\n{:?}", user_m);
    }

    #[test]
    fn lazy_trait() {
        let claims = Claims {
            my_claim: "hi".to_owned(),
        };
        let claims_bytes = serialize_lazy(&claims);
        let lazy_claims = Lazy::from_inner(claims.clone());
        assert_eq!(
            lazy_claims,
            Lazy {
                inner: Some(claims.clone()),
                bytes: None
            }
        );
        let mut lazy_mut = lazy_claims.clone();
        assert_eq!(claims_bytes, lazy_mut.bytes());
        assert_eq!(
            lazy_mut,
            Lazy {
                inner: None,
                bytes: Some(claims_bytes.clone())
            }
        );
        assert_eq!(claims_bytes, lazy_claims.clone().take_bytes());
        let lazy_serial = lazy_mut.clone();
        assert_eq!(&claims, lazy_mut.inner());
        assert_eq!(
            lazy_mut,
            Lazy {
                inner: Some(claims.clone()),
                bytes: None
            }
        );
        assert_eq!(claims, lazy_serial.take());
    }

    #[test]
    fn msgpack_round_trip() {
        let claims = Claims {
            my_claim: "hi".to_owned(),
        };
        let claims_bytes = serialize_lazy(&claims);
        let lazy_claims = Lazy::from_inner(claims.clone());
        let user = User {
            user_id: "abc".to_owned(),
            password_file: "hi".to_owned(),
            claims: lazy_claims,
        };

        let user_lazy = User {
            user_id: "abc".to_owned(),
            password_file: "hi".to_owned(),
            claims: Lazy::from_bytes(claims_bytes),
        };

        let bytes = rmp_serde::encode::to_vec_named(&user).unwrap();

        let decoded: User = rmp_serde::decode::from_slice(&bytes).unwrap();

        assert_eq!(user_lazy, decoded);
        assert_eq!(user, decoded.unwrap_lazy())
    }
}
