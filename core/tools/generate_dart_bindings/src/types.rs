//! Dart FFI type system representation
//!
//! This module defines the Dart FFI types used in code generation.
//! Each type knows how to render itself for FFI signatures and native Dart code.

/// Dart FFI types used in native function signatures
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DartFfiType {
    /// `Void` - no return value
    Void,
    /// `Bool` - boolean value (1 byte)
    Bool,
    /// `Int8` - signed 8-bit integer
    Int8,
    /// `Int16` - signed 16-bit integer
    Int16,
    /// `Int32` - signed 32-bit integer
    Int32,
    /// `Int64` - signed 64-bit integer
    Int64,
    /// `Uint8` - unsigned 8-bit integer
    Uint8,
    /// `Uint16` - unsigned 16-bit integer
    Uint16,
    /// `Uint32` - unsigned 32-bit integer
    Uint32,
    /// `Uint64` - unsigned 64-bit integer
    Uint64,
    /// `Float` - 32-bit floating point
    Float,
    /// `Double` - 64-bit floating point
    Double,
    /// `IntPtr` - platform-dependent signed integer
    IntPtr,
    /// `Size` - platform-dependent unsigned integer (size_t)
    Size,
    /// `Pointer<Utf8>` - pointer to UTF-8 string
    PointerUtf8,
    /// `Pointer<T>` - pointer to custom type
    Pointer(String),
}

impl DartFfiType {
    /// Returns the FFI type as it appears in typedef declarations
    pub fn ffi_type(&self) -> &str {
        match self {
            DartFfiType::Void => "Void",
            DartFfiType::Bool => "Bool",
            DartFfiType::Int8 => "Int8",
            DartFfiType::Int16 => "Int16",
            DartFfiType::Int32 => "Int32",
            DartFfiType::Int64 => "Int64",
            DartFfiType::Uint8 => "Uint8",
            DartFfiType::Uint16 => "Uint16",
            DartFfiType::Uint32 => "Uint32",
            DartFfiType::Uint64 => "Uint64",
            DartFfiType::Float => "Float",
            DartFfiType::Double => "Double",
            DartFfiType::IntPtr => "IntPtr",
            DartFfiType::Size => "Size",
            DartFfiType::PointerUtf8 => "Pointer<Utf8>",
            DartFfiType::Pointer(_) => "Pointer<Void>",
        }
    }

    /// Returns the native Dart type for use in wrapper functions
    pub fn dart_type(&self) -> &str {
        match self {
            DartFfiType::Void => "void",
            DartFfiType::Bool => "bool",
            DartFfiType::Int8
            | DartFfiType::Int16
            | DartFfiType::Int32
            | DartFfiType::Int64
            | DartFfiType::Uint8
            | DartFfiType::Uint16
            | DartFfiType::Uint32
            | DartFfiType::Uint64
            | DartFfiType::IntPtr
            | DartFfiType::Size => "int",
            DartFfiType::Float | DartFfiType::Double => "double",
            DartFfiType::PointerUtf8 => "String",
            DartFfiType::Pointer(_) => "Pointer<Void>",
        }
    }

    /// Returns the Dart FFI type for use in Function type signatures
    pub fn dart_ffi_function_type(&self) -> &str {
        match self {
            DartFfiType::Void => "void",
            DartFfiType::Bool => "bool",
            DartFfiType::Int8
            | DartFfiType::Int16
            | DartFfiType::Int32
            | DartFfiType::Int64
            | DartFfiType::Uint8
            | DartFfiType::Uint16
            | DartFfiType::Uint32
            | DartFfiType::Uint64
            | DartFfiType::IntPtr
            | DartFfiType::Size => "int",
            DartFfiType::Float | DartFfiType::Double => "double",
            DartFfiType::PointerUtf8 => "Pointer<Utf8>",
            DartFfiType::Pointer(_) => "Pointer<Void>",
        }
    }

    /// Returns whether this type requires memory cleanup
    pub fn requires_free(&self) -> bool {
        matches!(self, DartFfiType::PointerUtf8)
    }

    /// Returns the import needed for this type (if any beyond dart:ffi)
    pub fn required_import(&self) -> Option<&str> {
        match self {
            DartFfiType::PointerUtf8 => Some("package:ffi/ffi.dart"),
            _ => None,
        }
    }

    /// Returns whether this type is a pointer type
    pub fn is_pointer(&self) -> bool {
        matches!(self, DartFfiType::PointerUtf8 | DartFfiType::Pointer(_))
    }

    /// Returns whether this type is numeric
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            DartFfiType::Int8
                | DartFfiType::Int16
                | DartFfiType::Int32
                | DartFfiType::Int64
                | DartFfiType::Uint8
                | DartFfiType::Uint16
                | DartFfiType::Uint32
                | DartFfiType::Uint64
                | DartFfiType::Float
                | DartFfiType::Double
                | DartFfiType::IntPtr
                | DartFfiType::Size
        )
    }
}

impl std::fmt::Display for DartFfiType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ffi_type())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_type_string_representation() {
        assert_eq!(DartFfiType::Void.ffi_type(), "Void");
        assert_eq!(DartFfiType::Bool.ffi_type(), "Bool");
        assert_eq!(DartFfiType::Int32.ffi_type(), "Int32");
        assert_eq!(DartFfiType::Uint64.ffi_type(), "Uint64");
        assert_eq!(DartFfiType::Double.ffi_type(), "Double");
        assert_eq!(DartFfiType::PointerUtf8.ffi_type(), "Pointer<Utf8>");
    }

    #[test]
    fn test_dart_type_mapping() {
        assert_eq!(DartFfiType::Void.dart_type(), "void");
        assert_eq!(DartFfiType::Bool.dart_type(), "bool");
        assert_eq!(DartFfiType::Int32.dart_type(), "int");
        assert_eq!(DartFfiType::Uint8.dart_type(), "int");
        assert_eq!(DartFfiType::Double.dart_type(), "double");
        assert_eq!(DartFfiType::PointerUtf8.dart_type(), "String");
    }

    #[test]
    fn test_dart_ffi_function_type() {
        assert_eq!(DartFfiType::Void.dart_ffi_function_type(), "void");
        assert_eq!(DartFfiType::Int32.dart_ffi_function_type(), "int");
        assert_eq!(
            DartFfiType::PointerUtf8.dart_ffi_function_type(),
            "Pointer<Utf8>"
        );
    }

    #[test]
    fn test_requires_free() {
        assert!(DartFfiType::PointerUtf8.requires_free());
        assert!(!DartFfiType::Int32.requires_free());
        assert!(!DartFfiType::Bool.requires_free());
        assert!(!DartFfiType::Void.requires_free());
    }

    #[test]
    fn test_required_import() {
        assert_eq!(
            DartFfiType::PointerUtf8.required_import(),
            Some("package:ffi/ffi.dart")
        );
        assert_eq!(DartFfiType::Int32.required_import(), None);
        assert_eq!(DartFfiType::Void.required_import(), None);
    }

    #[test]
    fn test_is_pointer() {
        assert!(DartFfiType::PointerUtf8.is_pointer());
        assert!(DartFfiType::Pointer("MyStruct".to_string()).is_pointer());
        assert!(!DartFfiType::Int32.is_pointer());
        assert!(!DartFfiType::Bool.is_pointer());
    }

    #[test]
    fn test_is_numeric() {
        assert!(DartFfiType::Int32.is_numeric());
        assert!(DartFfiType::Uint64.is_numeric());
        assert!(DartFfiType::Double.is_numeric());
        assert!(!DartFfiType::Bool.is_numeric());
        assert!(!DartFfiType::Void.is_numeric());
        assert!(!DartFfiType::PointerUtf8.is_numeric());
    }

    #[test]
    fn test_display_trait() {
        assert_eq!(format!("{}", DartFfiType::Int32), "Int32");
        assert_eq!(format!("{}", DartFfiType::PointerUtf8), "Pointer<Utf8>");
    }

    #[test]
    fn test_clone_and_eq() {
        let t1 = DartFfiType::Int32;
        let t2 = t1.clone();
        assert_eq!(t1, t2);

        let t3 = DartFfiType::Pointer("Foo".to_string());
        let t4 = DartFfiType::Pointer("Foo".to_string());
        assert_eq!(t3, t4);
    }
}
