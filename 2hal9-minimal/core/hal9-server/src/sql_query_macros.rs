//! Macros for handling SQL queries with optional compile-time verification

/// Macro to handle SQL queries with optional compile-time verification
#[macro_export]
macro_rules! sql_query {
    // For queries without bind parameters
    ($query:expr) => {
        {
            #[cfg(feature = "sqlx-compile-time")]
            {
                sqlx::query!($query)
            }
            #[cfg(not(feature = "sqlx-compile-time"))]
            {
                sqlx::query($query)
            }
        }
    };
    
    // For queries with bind parameters
    ($query:expr, $($args:expr),* $(,)?) => {
        {
            #[cfg(feature = "sqlx-compile-time")]
            {
                sqlx::query!($query, $($args),*)
            }
            #[cfg(not(feature = "sqlx-compile-time"))]
            {
                sqlx::query($query)
                    $(.bind($args))*
            }
        }
    };
}

/// Macro to handle SQL queries that return data with optional compile-time verification
#[macro_export]
macro_rules! sql_query_as {
    // For typed queries
    ($type:ty, $query:expr) => {
        {
            #[cfg(feature = "sqlx-compile-time")]
            {
                sqlx::query_as!($type, $query)
            }
            #[cfg(not(feature = "sqlx-compile-time"))]
            {
                sqlx::query_as::<_, $type>($query)
            }
        }
    };
    
    // For typed queries with bind parameters
    ($type:ty, $query:expr, $($args:expr),* $(,)?) => {
        {
            #[cfg(feature = "sqlx-compile-time")]
            {
                sqlx::query_as!($type, $query, $($args),*)
            }
            #[cfg(not(feature = "sqlx-compile-time"))]
            {
                sqlx::query_as::<_, $type>($query)
                    $(.bind($args))*
            }
        }
    };
}