//! Pointer helper utilities for safe atomic pointer dereferencing
//!
//! This module provides helper functions to reduce boilerplate when working with
//! `AtomicPtr<c_void>` pointers that store references to Objective-C objects.
//!
//! The helpers encapsulate the common pattern of:
//! 1. Loading a pointer atomically
//! 2. Checking for null
//! 3. Casting and dereferencing to the expected type
//! 4. Calling a closure with the dereferenced value

use std::ffi::c_void;
use std::sync::atomic::{AtomicPtr, Ordering};

/// Execute a closure with a dereferenced atomic pointer if non-null.
///
/// This helper reduces the common pattern of loading an atomic pointer,
/// null-checking, casting, and dereferencing into a single call.
///
/// # Returns
///
/// - `Some(R)` if the pointer is non-null and the closure was executed
/// - `None` if the pointer is null
///
/// # Safety
///
/// The caller must ensure:
/// - The pointer was originally stored from a valid `Retained<T>` or equivalent
/// - The object is still alive (not cleaned up)
/// - This is called from the main thread (for NSObject types)
/// - The type `T` matches the actual stored object type
///
/// # Example
///
/// ```ignore
/// // Before (5 lines):
/// let ptr = settings::EXIT_KEY_FIELD.load(Ordering::Acquire);
/// if !ptr.is_null() {
///     unsafe {
///         let field: &NSTextField = &*(ptr as *const NSTextField);
///         let value = field.stringValue().to_string();
///     }
/// }
///
/// // After (3 lines):
/// let value = unsafe {
///     with_ptr::<NSTextField, _, _>(&settings::EXIT_KEY_FIELD, |field| {
///         field.stringValue().to_string()
///     })
/// };
/// ```
#[inline]
pub unsafe fn with_ptr<T, F, R>(ptr: &AtomicPtr<c_void>, f: F) -> Option<R>
where
    F: FnOnce(&T) -> R,
{
    let raw = ptr.load(Ordering::Acquire);
    if raw.is_null() {
        None
    } else {
        Some(f(&*(raw as *const T)))
    }
}

/// Execute a closure with a dereferenced atomic pointer if non-null (no return value).
///
/// Similar to [`with_ptr`], but for closures that don't need to return a value.
/// This avoids the overhead of wrapping the result in `Option<()>`.
///
/// # Safety
///
/// The caller must ensure:
/// - The pointer was originally stored from a valid `Retained<T>` or equivalent
/// - The object is still alive (not cleaned up)
/// - This is called from the main thread (for NSObject types)
/// - The type `T` matches the actual stored object type
///
/// # Example
///
/// ```ignore
/// // Before (5 lines):
/// let ptr = settings::EXIT_KEY_FIELD.load(Ordering::Acquire);
/// if !ptr.is_null() {
///     unsafe {
///         let field: &NSTextField = &*(ptr as *const NSTextField);
///         field.setStringValue(&NSString::from_str("value"));
///     }
/// }
///
/// // After (3 lines):
/// unsafe {
///     with_ptr_void::<NSTextField, _>(&settings::EXIT_KEY_FIELD, |field| {
///         field.setStringValue(&NSString::from_str("value"));
///     });
/// }
/// ```
#[inline]
pub unsafe fn with_ptr_void<T, F>(ptr: &AtomicPtr<c_void>, f: F)
where
    F: FnOnce(&T),
{
    let raw = ptr.load(Ordering::Acquire);
    if !raw.is_null() {
        f(&*(raw as *const T));
    }
}

/// Execute a closure with a mutable reference to a dereferenced atomic pointer if non-null.
///
/// Similar to [`with_ptr`], but provides a mutable reference to the dereferenced value.
/// Use this when you need to mutate the object through the pointer.
///
/// # Returns
///
/// - `Some(R)` if the pointer is non-null and the closure was executed
/// - `None` if the pointer is null
///
/// # Safety
///
/// The caller must ensure:
/// - The pointer was originally stored from a valid `Retained<T>` or equivalent
/// - The object is still alive (not cleaned up)
/// - This is called from the main thread (for NSObject types)
/// - The type `T` matches the actual stored object type
/// - No other references to this object exist during the closure execution
///
/// # Example
///
/// ```ignore
/// // When you need mutable access:
/// unsafe {
///     with_ptr_mut::<MyMutableType, _, _>(&state::MUTABLE_OBJECT, |obj| {
///         obj.update_value(42);
///         obj.get_result()
///     })
/// };
/// ```
#[inline]
pub unsafe fn with_ptr_mut<T, F, R>(ptr: &AtomicPtr<c_void>, f: F) -> Option<R>
where
    F: FnOnce(&mut T) -> R,
{
    let raw = ptr.load(Ordering::Acquire);
    if raw.is_null() {
        None
    } else {
        Some(f(&mut *(raw as *mut T)))
    }
}

/// Execute a closure with a raw pointer if non-null.
///
/// This variant takes a raw `*mut c_void` pointer directly instead of an
/// `AtomicPtr`. Useful when the pointer has already been loaded or when
/// working with non-atomic pointers.
///
/// # Safety
///
/// The caller must ensure:
/// - The pointer was originally derived from a valid `Retained<T>` or equivalent
/// - The object is still alive (not cleaned up)
/// - This is called from the main thread (for NSObject types)
/// - The type `T` matches the actual stored object type
///
/// # Example
///
/// ```ignore
/// // When you already have a loaded pointer:
/// let ptr = settings::TIMER_VALIDATION.load(Ordering::Acquire);
/// unsafe {
///     with_raw_ptr::<NSTextField, _>(ptr, |label| {
///         label.setStringValue(&NSString::from_str("✓ Valid"));
///     });
/// }
/// ```
#[inline]
pub unsafe fn with_raw_ptr<T, F>(ptr: *mut c_void, f: F)
where
    F: FnOnce(&T),
{
    if !ptr.is_null() {
        f(&*(ptr as *const T));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_ptr_null() {
        let ptr: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
        let result = unsafe { with_ptr::<u32, _, _>(&ptr, |v| *v) };
        assert!(result.is_none());
    }

    #[test]
    fn test_with_ptr_non_null() {
        let value: u32 = 42;
        let ptr: AtomicPtr<c_void> =
            AtomicPtr::new(&value as *const u32 as *const c_void as *mut c_void);
        let result = unsafe { with_ptr::<u32, _, _>(&ptr, |v| *v) };
        assert_eq!(result, Some(42));
    }

    #[test]
    fn test_with_ptr_void_null() {
        let ptr: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
        let mut called = false;
        unsafe {
            with_ptr_void::<u32, _>(&ptr, |_| {
                called = true;
            });
        }
        assert!(!called);
    }

    #[test]
    fn test_with_ptr_void_non_null() {
        let value: u32 = 42;
        let ptr: AtomicPtr<c_void> =
            AtomicPtr::new(&value as *const u32 as *const c_void as *mut c_void);
        let mut called = false;
        let mut seen_value = 0;
        unsafe {
            with_ptr_void::<u32, _>(&ptr, |v| {
                called = true;
                seen_value = *v;
            });
        }
        assert!(called);
        assert_eq!(seen_value, 42);
    }

    #[test]
    fn test_with_raw_ptr_null() {
        let ptr: *mut c_void = std::ptr::null_mut();
        let mut called = false;
        unsafe {
            with_raw_ptr::<u32, _>(ptr, |_| {
                called = true;
            });
        }
        assert!(!called);
    }

    #[test]
    fn test_with_raw_ptr_non_null() {
        let value: u32 = 42;
        let ptr: *mut c_void = &value as *const u32 as *const c_void as *mut c_void;
        let mut called = false;
        let mut seen_value = 0;
        unsafe {
            with_raw_ptr::<u32, _>(ptr, |v| {
                called = true;
                seen_value = *v;
            });
        }
        assert!(called);
        assert_eq!(seen_value, 42);
    }

    #[test]
    fn test_with_ptr_return_string() {
        let value = String::from("hello");
        let ptr: AtomicPtr<c_void> =
            AtomicPtr::new(&value as *const String as *const c_void as *mut c_void);
        let result = unsafe { with_ptr::<String, _, _>(&ptr, |s| s.len()) };
        assert_eq!(result, Some(5));
    }

    #[test]
    fn test_with_ptr_mut_null() {
        let ptr: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
        let result = unsafe { with_ptr_mut::<u32, _, _>(&ptr, |v| *v) };
        assert!(result.is_none());
    }

    #[test]
    fn test_with_ptr_mut_non_null() {
        let mut value: u32 = 42;
        let ptr: AtomicPtr<c_void> =
            AtomicPtr::new(&mut value as *mut u32 as *mut c_void);
        let result = unsafe {
            with_ptr_mut::<u32, _, _>(&ptr, |v| {
                *v = 100;
                *v
            })
        };
        assert_eq!(result, Some(100));
        assert_eq!(value, 100);
    }

    #[test]
    fn test_with_ptr_mut_modifies_value() {
        let mut value: Vec<i32> = vec![1, 2, 3];
        let ptr: AtomicPtr<c_void> =
            AtomicPtr::new(&mut value as *mut Vec<i32> as *mut c_void);
        unsafe {
            with_ptr_mut::<Vec<i32>, _, _>(&ptr, |v| {
                v.push(4);
                v.len()
            });
        }
        assert_eq!(value, vec![1, 2, 3, 4]);
    }
}
