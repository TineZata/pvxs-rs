#[derive(Debug, PartialEq)]
pub enum StoreType {
    Null        = 0,
    Bool        = 1,
    UInteger    = 2,
    Integer     = 3,
    Real        = 4,
    String      = 5,
    Compound    = 6,
    Array       = 7,
}

/// Convert an integer to a `StoreType`
/// From the original C++ code:
///        //! selector for union FieldStorage::store
///        enum struct StoreType : uint8_t {
///            Null,     //!< no associate storage
///            Bool,     //!< bool
///            UInteger, //!< uint64_t
///            Integer,  //!< int64_t
///            Real,     //!< double
///            String,   //!< std::string
///            Compound, //!< Value
///            Array,    //!< shared_array<const void>
///        }; 
///     
impl From<i32> for StoreType {
    fn from(value: i32) -> Self {
        match value {
            0 => StoreType::Null,
            1 => StoreType::Bool,
            2 => StoreType::UInteger,
            3 => StoreType::Integer,
            4 => StoreType::Real,
            5 => StoreType::String,
            6 => StoreType::Compound,
            7 => StoreType::Array,
            _ => StoreType::Null,
        }
    }
}
