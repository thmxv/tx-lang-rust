
#[cfg(not(feature = "tx32"))]
pub type TxInt = i64;
#[cfg(not(feature = "tx32"))]
pub type TxFloat = f64;

#[cfg(feature = "tx32")]
pub type TxInt = i32;
#[cfg(feature = "tx32")]
pub type TxFloat = f32;

// pub type DynArray<T> = Vec<T>;

