// Handle custom deserialization errors
macro_rules! bail_de {
    ($msg:literal $(,)?) => {
        {
            use serde::de::Error;
            return Err(serde_yaml::Error::custom($msg))
        }
    };
    ($err:expr $(,)?) => {
        {
            use serde::de::Error;
            return Err(serde_yaml::Error::custom($err))
        }
    };
    ($fmt:expr, $($arg:tt)*) => {
        {
            use serde::de::Error;
            let msg = format!($fmt, $($arg)*);
            return Err(serde_yaml::Error::custom(msg))
        }
    };
}
