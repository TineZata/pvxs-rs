#[repr(C)]
pub struct Context {
    _private: [u8; 0], // Prevent direct instantiation
}
