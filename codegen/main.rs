use std::fs::File;
use std::io::prelude::*;

fn main() {
    generate_libcast().unwrap();
}

fn generate_libcast() -> std::io::Result<()> {
    let int_sizes = [
        8, 16, 24, 32, 40, 48, 56, 64, 72, 80, 88, 96, 104, 112, 120, 128, 136, 144, 152, 160, 168,
        176, 184, 192, 200, 208, 216, 224, 232, 240, 248, 256,
    ]
    .iter()
    .map(|size| generate_cast(*size))
    .collect::<Vec<String>>()
    .join("\n");

    let libcast = format!(
        "{}{}{}{}",
        HEADER,
        ERROR_DEFINITION,
        int_sizes,
        MINI_MASK_DEFINITION,
    );

    let mut f = File::create("src/libcast.huff")?;

    f.write_all(libcast.as_bytes())?;

    Ok(())
}

fn generate_cast(size: u16) -> String {
    let name = format!("U{}", size);
    let mut mask = String::with_capacity(size as usize / 8 + 2);
    mask.push_str("0x");

    for _ in 0..size / 8 {
        mask.push_str("ff");
    }

    let mask_template = MASK_TEMPLATE
        .replace("TYPENAME", &name)
        .replace("TYPEMASK", &mask)
        .replace("TYPESIZE", &size.to_string());

    if size < 32 {
        return mask_template;
    }

    let mini_mask_template = MINI_MASK_TEMPLATE
        .replace("TYPENAME", &name)
        .replace("TYPEMASK", &mask)
        .replace("TYPESIZE", &size.to_string());

    format!("{}{}", mask_template, mini_mask_template)
}

const HEADER: &'static str = r#"
//  ------------------------------------------------------------------------------------------------
//! # Casting Library
//! 
//! Provides macros for casting values.
//! 
//! Bit sizes supported range from 8 to 256 inclusive and are multiples of 8.
//! 
//! Items prefixed with `UNSAFE_` will not revert on overflow.
//! 
//! Items prefixed with `MINI_` will consume more runtime gas to the benefit of a smaller runtime
//! size.
//! 
//! ## API
//! 
//! For a given type, `TYPENAME`:
//! 
//! - `TYPENAME_MASK` - Used to downcast a value to a smaller type.
//! - `TO_TYPENAME` - Downcasts a value to a smaller type.
//! - `UNSAFE_TO_TYPENAME` - Downcasts a value to a smaller type.
//! - `MINI_TYPENAME_MASK` - Used to downcast a value to a smaller type.
//! - `MINI_TO_TYPENAME` - Downcasts a value to a smaller type.
//! - `UNSAFE_MINI_TO_TYPENAME` - Downcasts a value to a smaller type.
//! 
"#;

const ERROR_DEFINITION: &'static str = r#"
/// ## Overflow Error
/// 
/// Thrown when a cast overflows.
#define error Overflow()
"#;

const MASK_TEMPLATE: &'static str = r#"
/// ## TYPENAME Mask
/// 
/// Used to downcast a value to a smaller type.
/// 
/// ### Usage
/// 
/// ```huff
/// #define macro MAIN() = takes (0) returns (0) {
///     0x00 calldataload
///     TYPENAME_MASK() and
/// }
/// ```
#define macro TYPENAME_MASK() = takes (0) returns (1) { TYPEMASK }

/// ## TYPENAME Cast
/// 
/// Downcasts a value to a smaller type.
/// 
/// The `UNSAFE_TO_TYPENAME` macro will not revert on overflow.
#define macro TO_TYPENAME() = takes (1) returns (1) {
    // takes:               // [value]
    dup1                    // [value, value]
    TYPENAME_MASK()         // [mask, value, value]
    and                     // [masked_value, value]
    dup2                    // [value, masked_value, value]
    eq                      // [is_safe, value]
    is_safe                 // [is_safe_dest, is_safe, value]
    jumpi                   // [value]
        __ERROR(Overflow)   // [err]
        0x00                // [ptr, err]
        mstore              // []
        0x04                // [err_len]
        0x00                // [ptr, err_len]
        revert              // []
    is_safe:                // [value]
}"#;

const MINI_MASK_TEMPLATE: &'static str = r#"

/// ## Mini TYPENAME Mask
/// 
/// Used to downcast a value to a smaller type.
/// 
/// This consumes more runtime gas to the benefit of a smaller runtime size.
/// 
/// ### Usage
/// 
/// ```huff
/// #define macro MAIN() = takes (0) returns (0) {
///     0x00 calldataload
///     MINI_TYPENAME_MASK() and
/// }
/// ```
#define macro MINI_TYPENAME_MASK() = takes (0) returns (1) { __MINI_MASK(TYPESIZE) }

/// ## Mini TYPENAME Cast
/// 
/// Downcasts a value to a smaller type.
/// 
/// This consumes more runtime gas to the benefit of a smaller runtime size.
/// 
/// The `UNSAFE_MINI_TO_TYPENAME` macro will not revert on overflow.
#define macro UNSAFE_MINI_TO_TYPENAME() = takes (0) returns (0) {
    // takes:               // [value]
    MINI_TYPENAME_MASK()         // [mask, value]
    and                     // [masked_value]
}"#;

const MINI_MASK_DEFINITION: &'static str = r#"
/// ## Mini Mask
///
/// Used as a utility to generate the mask
///
/// The macro body is functionally equivalent to the following: `2 ** bitsize - 1`
///
/// ### Template Arguments
///
/// - `bitsize` - The number of bits to generate a mask for.
///
/// ### Usage
///
/// ```huff
/// #define macro MINI_U32_MASK() = takes (0) returns (1) { __MINI_MASK(32)}
/// ```
#define macro __MINI_MASK(bitsize) = takes (0) returns (1) {
    0x01        // [one]
    dup1        // [one, one]
    <bitsize>   // [bisize, one, one]
    shl         // [mask_plus_one, one]
    sub         // [mask]
}
"#;
