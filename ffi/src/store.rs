//! For storing transferable public keys.
//!
//! The key store stores transferable public keys (TPKs) using an
//! arbitrary label.  Stored keys are automatically updated from
//! remote sources.  This ensures that updates like new subkeys and
//! revocations are discovered in a timely manner.
//!
//! # Security considerations
//!
//! Storing public keys potentially leaks communication partners.
//! Protecting against adversaries inspecting the local storage is out
//! of scope for Sequoia.  Please take the necessary precautions.
//!
//! Sequoia updates keys in compliance with the [network policy] used
//! to create the store.
//!
//! [network policy]: ../../sequoia_core/enum.NetworkPolicy.html
//!
//! # Example
//!
//! ```c, ignore
//! XXX
//! ```


use libc::{uint8_t, uint64_t, c_char};
use std::ffi::{CStr, CString};
use std::ptr;

extern crate openpgp;

use self::openpgp::TPK;
use self::openpgp::{
    Fingerprint,
    KeyID
};
use sequoia_store::{
    self, Store, StoreIter, Binding, BindingIter, Key, KeyIter, LogIter, Pool,
};

use super::error::Status;
use super::core::{Context, sq_string_free};


/// Lists all stores with the given prefix.
#[no_mangle]
pub extern "system" fn sq_store_list_stores(ctx: Option<&mut Context>,
                                            domain_prefix: *const c_char)
                                            -> *mut StoreIter {
    let ctx = ctx.expect("Context is NULL");
    assert!(! domain_prefix.is_null());

    let domain_prefix = unsafe {
        CStr::from_ptr(domain_prefix).to_string_lossy()
    };

    fry_box!(ctx, Store::list(&ctx.c, &domain_prefix))
}

/// Returns the next store.
///
/// Returns `NULL` on exhaustion.  If `domainp` is not `NULL`, the
/// stores domain is stored there.  If `namep` is not `NULL`, the
/// stores name is stored there.  If `policyp` is not `NULL`, the
/// stores network policy is stored there.
#[no_mangle]
pub extern "system" fn sq_store_iter_next(iter: Option<&mut StoreIter>,
                                          domainp: Option<&mut *mut c_char>,
                                          namep: Option<&mut *mut c_char>,
                                          policyp: Option<&mut uint8_t>)
                                          -> *mut Store {
    let iter = iter.expect("Iterator is NULL");
    match iter.next() {
        Some((domain, name, policy, store)) => {
            if domainp.is_some() {
                *domainp.unwrap() = CString::new(domain)
                    .map(|c| c.into_raw())
                    .unwrap_or(ptr::null_mut());
            }

            if namep.is_some() {
                *namep.unwrap() = CString::new(name)
                    .map(|c| c.into_raw())
                    .unwrap_or(ptr::null_mut());
            }

            if policyp.is_some() {
                *policyp.unwrap() = (&policy).into();
            }

            box_raw!(store)
        },
        None => ptr::null_mut(),
    }
}

/// Frees a sq_store_iter_t.
#[no_mangle]
pub extern "system" fn sq_store_iter_free(iter: *mut StoreIter) {
    if iter.is_null() { return };
    unsafe {
        drop(Box::from_raw(iter))
    };
}

/// Lists all keys in the common key pool.
#[no_mangle]
pub extern "system" fn sq_store_list_keys(ctx: Option<&mut Context>)
                                          -> *mut KeyIter {
    let ctx = ctx.expect("Context is NULL");

    fry_box!(ctx, Store::list_keys(&ctx.c))
}

/// Lists all log entries.
#[no_mangle]
pub extern "system" fn sq_store_server_log(ctx: Option<&mut Context>)
                                           -> *mut LogIter {
    let ctx = ctx.expect("Context is NULL");

    fry_box!(ctx, Store::server_log(&ctx.c))
}

/// Returns the next key.
///
/// Returns `NULL` on exhaustion.  If `fpp` is not `NULL`, the key's
/// fingerprint is stored there.
#[no_mangle]
pub extern "system" fn sq_key_iter_next(iter: Option<&mut KeyIter>,
                                        fpp: Option<&mut *mut Fingerprint>)
                                        -> *mut Key {
    let iter = iter.expect("Iterator is NULL");
    match iter.next() {
        Some((fingerprint, key)) => {
            if fpp.is_some() {
                *fpp.unwrap() = box_raw!(fingerprint);
            }

            box_raw!(key)
        },
        None => ptr::null_mut(),
    }
}

/// Frees a sq_key_iter_t.
#[no_mangle]
pub extern "system" fn sq_key_iter_free(iter: *mut KeyIter) {
    if iter.is_null() { return };
    unsafe {
        drop(Box::from_raw(iter))
    };
}


fn cstring(s: &str) -> *mut c_char {
    CString::new(s)
        .map(|c| c.into_raw())
        // Fails only on internal nul bytes.
        .unwrap_or(ptr::null_mut())
}

/// Returns the next log entry.
///
/// Returns `NULL` on exhaustion.
#[no_mangle]
pub extern "system" fn sq_log_iter_next(iter: Option<&mut LogIter>)
                                        -> *mut Log {
    let iter = iter.expect("Iterator is NULL");
    match iter.next() {
        Some(e) => {
            let (status, error) = match e.status {
                Ok(s) => (cstring(&s), ptr::null_mut()),
                Err((s, e)) => (cstring(&s), cstring(&e)),
            };

            box_raw!(Log{
                timestamp: e.timestamp.sec as uint64_t,
                store: maybe_box_raw!(e.store),
                binding: maybe_box_raw!(e.binding),
                key: maybe_box_raw!(e.key),
                slug: cstring(&e.slug),
                status: status,
                error: error,
            })
        },
        None => ptr::null_mut(),
    }
}

/// Frees a sq_log_iter_t.
#[no_mangle]
pub extern "system" fn sq_log_iter_free(iter: *mut LogIter) {
    if iter.is_null() { return };
    unsafe {
        drop(Box::from_raw(iter))
    };
}

/// Opens a store.
///
/// Opens a store with the given name.  If the store does not
/// exist, it is created.  Stores are handles for objects
/// maintained by a background service.  The background service
/// associates state with this name.
///
/// The store updates TPKs in compliance with the network policy
/// of the context that created the store in the first place.
/// Opening the store with a different network policy is
/// forbidden.
#[no_mangle]
pub extern "system" fn sq_store_open(ctx: Option<&mut Context>,
                                     name: *const c_char)
                                     -> *mut Store {
    let ctx = ctx.expect("Context is NULL");
    assert!(! name.is_null());

    let name = unsafe {
        CStr::from_ptr(name).to_string_lossy()
    };

    fry_box!(ctx, Store::open(&ctx.c, &name))
}

/// Frees a sq_store_t.
#[no_mangle]
pub extern "system" fn sq_store_free(store: *mut Store) {
    if store.is_null() { return };
    unsafe {
        drop(Box::from_raw(store))
    };
}

/// Adds a key identified by fingerprint to the store.
#[no_mangle]
pub extern "system" fn sq_store_add(ctx: Option<&mut Context>,
                                    store: Option<&Store>,
                                    label: *const c_char,
                                    fingerprint: Option<&Fingerprint>)
                                    -> *mut Binding {
    let ctx = ctx.expect("Context is NULL");
    let store = store.expect("Store is NULL");
    assert!(! label.is_null());
    let label = unsafe {
        CStr::from_ptr(label).to_string_lossy()
    };
    let fingerprint = fingerprint.expect("Fingerprint is NULL");

    fry_box!(ctx, store.add(&label, fingerprint))
}

/// Imports a key into the store.
#[no_mangle]
pub extern "system" fn sq_store_import(ctx: Option<&mut Context>,
                                       store: Option<&Store>,
                                       label: *const c_char,
                                       tpk: Option<&TPK>)
                                       -> *mut TPK {
    let ctx = ctx.expect("Context is NULL");
    let store = store.expect("Store is NULL");
    assert!(! label.is_null());
    let label = unsafe {
        CStr::from_ptr(label).to_string_lossy()
    };
    let tpk = tpk.expect("TPK is NULL");

    fry_box!(ctx, store.import(&label, tpk))
}

/// Returns the binding for the given label.
#[no_mangle]
pub extern "system" fn sq_store_lookup(ctx: Option<&mut Context>,
                                       store: Option<&Store>,
                                       label: *const c_char)
                                       -> *mut Binding {
    let ctx = ctx.expect("Context is NULL");
    let store = store.expect("Store is NULL");
    assert!(! label.is_null());
    let label = unsafe {
        CStr::from_ptr(label).to_string_lossy()
    };

    fry_box!(ctx, store.lookup(&label))
}

/// Looks up a key in the common key pool by KeyID.
#[no_mangle]
pub extern "system" fn sq_store_lookup_by_keyid(ctx: Option<&mut Context>,
                                                keyid: Option<&KeyID>)
    -> *mut Key
{
    let ctx = ctx.expect("Context is NULL");
    let keyid = keyid.expect("KeyID is NULL");

    fry_box!(ctx, Pool::lookup_by_keyid(&ctx.c, keyid))
}

/// Looks up a key in the common key pool by (Sub)KeyID.
#[no_mangle]
pub extern "system" fn sq_store_lookup_by_subkeyid(ctx: Option<&mut Context>,
                                                   keyid: Option<&KeyID>)
    -> *mut Key
{
    let ctx = ctx.expect("Context is NULL");
    let keyid = keyid.expect("KeyID is NULL");

    fry_box!(ctx, Pool::lookup_by_subkeyid(&ctx.c, keyid))
}

/// Deletes this store.
///
/// Consumes `store`.  Returns != 0 on error.
#[no_mangle]
pub extern "system" fn sq_store_delete(ctx: Option<&mut Context>,
                                       store: *mut Store)
                                       -> Status {
    let ctx = ctx.expect("Context is NULL");
    assert!(! store.is_null());
    let store = unsafe {
        Box::from_raw(store)
    };

    fry_status!(ctx, store.delete())
}

/// Lists all bindings.
#[no_mangle]
pub extern "system" fn sq_store_iter(ctx: Option<&mut Context>,
                                     store: Option<&Store>)
                                     -> *mut BindingIter {
    let ctx = ctx.expect("Context is NULL");
    let store = store.expect("Store is NULL");

    fry_box!(ctx, store.iter())
}

/// Returns the next binding.
///
/// Returns `NULL` on exhaustion.  If `labelp` is not `NULL`, the
/// bindings label is stored there.  If `fpp` is not `NULL`, the
/// bindings fingerprint is stored there.
#[no_mangle]
pub extern "system" fn sq_binding_iter_next(iter: Option<&mut BindingIter>,
                                            labelp: Option<&mut *mut c_char>,
                                            fpp: Option<&mut *mut Fingerprint>)
                                            -> *mut Binding {
    let iter = iter.expect("Iterator is NULL");
    match iter.next() {
        Some((label, fp, binding)) => {
            if labelp.is_some() {
                *labelp.unwrap() = CString::new(label)
                    .map(|c| c.into_raw())
                    .unwrap_or(ptr::null_mut());
            }

            if fpp.is_some() {
                *fpp.unwrap() = box_raw!(fp);
            }

            box_raw!(binding)
        },
        None => ptr::null_mut(),
    }
}

/// Frees a sq_binding_iter_t.
#[no_mangle]
pub extern "system" fn sq_binding_iter_free(iter: *mut BindingIter) {
    if iter.is_null() { return };
    unsafe {
        drop(Box::from_raw(iter))
    };
}

/// Lists all log entries related to this store.
#[no_mangle]
pub extern "system" fn sq_store_log(ctx: Option<&mut Context>,
                                    store: Option<&Store>)
                                    -> *mut LogIter {
    let ctx = ctx.expect("Context is NULL");
    let store = store.expect("Store is NULL");

    fry_box!(ctx, store.log())
}

/// Frees a sq_binding_t.
#[no_mangle]
pub extern "system" fn sq_binding_free(binding: *mut Binding) {
    if binding.is_null() { return };
    unsafe {
        drop(Box::from_raw(binding))
    };
}

/// Frees a sq_key_t.
#[no_mangle]
pub extern "system" fn sq_key_free(key: *mut Key) {
    if key.is_null() { return };
    unsafe {
        drop(Box::from_raw(key))
    };
}

/// Frees a sq_log_t.
#[no_mangle]
pub extern "system" fn sq_log_free(log: *mut Log) {
    if log.is_null() { return };
    let log = unsafe {
        Box::from_raw(log)
    };
    sq_store_free(log.store);
    sq_binding_free(log.binding);
    sq_key_free(log.key);
    sq_string_free(log.slug);
    sq_string_free(log.status);
    sq_string_free(log.error);
    drop(log)
}

/// Returns the `sq_stats_t` of this binding.
#[no_mangle]
pub extern "system" fn sq_binding_stats(ctx: Option<&mut Context>,
                                        binding: Option<&Binding>)
                                        -> *mut Stats {
    let ctx = ctx.expect("Context is NULL");
    let binding = binding.expect("Binding is NULL");

    box_raw!(Stats::new(fry!(ctx, binding.stats())))
}

/// Returns the `sq_key_t` of this binding.
#[no_mangle]
pub extern "system" fn sq_binding_key(ctx: Option<&mut Context>,
                                      binding: Option<&Binding>)
                                     -> *mut Key {
    let ctx = ctx.expect("Context is NULL");
    let binding = binding.expect("Binding is NULL");

    fry_box!(ctx, binding.key())
}

/// Returns the `sq_tpk_t` of this binding.
#[no_mangle]
pub extern "system" fn sq_binding_tpk(ctx: Option<&mut Context>,
                                      binding: Option<&Binding>)
                                     -> *mut TPK {
    let ctx = ctx.expect("Context is NULL");
    let binding = binding.expect("Binding is NULL");

    fry_box!(ctx, binding.tpk())
}

/// Updates this binding with the given TPK.
///
/// If the new key `tpk` matches the current key, i.e. they have
/// the same fingerprint, both keys are merged and normalized.
/// The returned key contains all packets known to Sequoia, and
/// should be used instead of `tpk`.
///
/// If the new key does not match the current key, but carries a
/// valid signature from the current key, it replaces the current
/// key.  This provides a natural way for key rotations.
///
/// If the new key does not match the current key, and it does not
/// carry a valid signature from the current key, an
/// `Error::Conflict` is returned, and you have to resolve the
/// conflict, either by ignoring the new key, or by using
/// `sq_binding_rotate` to force a rotation.
#[no_mangle]
pub extern "system" fn sq_binding_import(ctx: Option<&mut Context>,
                                         binding: Option<&Binding>,
                                         tpk: Option<&TPK>)
                                         -> *mut TPK {
    let ctx = ctx.expect("Context is NULL");
    let binding = binding.expect("Binding is NULL");
    let tpk = tpk.expect("TPK is NULL");

    fry_box!(ctx, binding.import(&tpk))
}


/// Forces a keyrotation to the given TPK.
///
/// The current key is replaced with the new key `tpk`, even if
/// they do not have the same fingerprint.  If a key with the same
/// fingerprint as `tpk` is already in the store, is merged with
/// `tpk` and normalized.  The returned key contains all packets
/// known to Sequoia, and should be used instead of `tpk`.
///
/// Use this function to resolve conflicts returned from
/// `sq_binding_import`.  Make sure that you have authenticated
/// `tpk` properly.  How to do that depends on your thread model.
/// You could simply ask Alice to call her communication partner
/// Bob and confirm that he rotated his keys.
#[no_mangle]
pub extern "system" fn sq_binding_rotate(ctx: Option<&mut Context>,
                                         binding: Option<&Binding>,
                                         tpk: Option<&TPK>)
                                         -> *mut TPK {
    let ctx = ctx.expect("Context is NULL");
    let binding = binding.expect("Binding is NULL");
    let tpk = tpk.expect("TPK is NULL");

    fry_box!(ctx, binding.rotate(&tpk))
}

/// Deletes this binding.
///
/// Consumes `binding`.  Returns != 0 on error.
#[no_mangle]
pub extern "system" fn sq_binding_delete(ctx: Option<&mut Context>,
                                         binding: *mut Binding)
                                         -> Status {
    let ctx = ctx.expect("Context is NULL");
    assert!(! binding.is_null());
    let binding = unsafe {
        Box::from_raw(binding)
    };

    fry_status!(ctx, binding.delete())
}

/// Lists all log entries related to this binding.
#[no_mangle]
pub extern "system" fn sq_binding_log(ctx: Option<&mut Context>,
                                      binding: Option<&Binding>)
                                      -> *mut LogIter {
    let ctx = ctx.expect("Context is NULL");
    let binding = binding.expect("Binding is NULL");

    fry_box!(ctx, binding.log())
}

/// Returns the `sq_stats_t` of this key.
#[no_mangle]
pub extern "system" fn sq_key_stats(ctx: Option<&mut Context>,
                                    key: Option<&Key>)
                                    -> *mut Stats {
    let ctx = ctx.expect("Context is NULL");
    let key = key.expect("Key is NULL");

    box_raw!(Stats::new(fry!(ctx, key.stats())))
}

/// Returns the `sq_tpk_t`.
#[no_mangle]
pub extern "system" fn sq_key_tpk(ctx: Option<&mut Context>,
                                  key: Option<&Key>)
                                  -> *mut TPK {
    let ctx = ctx.expect("Context is NULL");
    let key = key.expect("Key is NULL");

    fry_box!(ctx, key.tpk())
}

/// Updates this stored key with the given TPK.
///
/// If the new key `tpk` matches the current key, i.e. they have
/// the same fingerprint, both keys are merged and normalized.
/// The returned key contains all packets known to Sequoia, and
/// should be used instead of `tpk`.
///
/// If the new key does not match the current key,
/// `Error::Conflict` is returned.
#[no_mangle]
pub extern "system" fn sq_key_import(ctx: Option<&mut Context>,
                                     key: Option<&Key>,
                                     tpk: Option<&TPK>)
                                     -> *mut TPK {
    let ctx = ctx.expect("Context is NULL");
    let key = key.expect("Key is NULL");
    let tpk = tpk.expect("TPK is NULL");

    fry_box!(ctx, key.import(&tpk))
}

/// Lists all log entries related to this key.
#[no_mangle]
pub extern "system" fn sq_key_log(ctx: Option<&mut Context>,
                                  key: Option<&Key>)
                                  -> *mut LogIter {
    let ctx = ctx.expect("Context is NULL");
    let key = key.expect("Key is NULL");

    fry_box!(ctx, key.log())
}

/// Frees a sq_stats_t.
#[no_mangle]
pub extern "system" fn sq_stats_free(stats: *mut Stats) {
    if stats.is_null() { return };
    unsafe {
        drop(Box::from_raw(stats))
    };
}

/// Counter and timestamps.
#[repr(C)]
pub struct Stamps {
    /// Counts how many times this has been used.
    pub count: uint64_t,

    /// Records the time when this has been used first.
    pub first:  uint64_t,

    /// Records the time when this has been used last.
    pub last: uint64_t,
}

impl Stamps {
    fn new(s: &sequoia_store::Stamps) -> Stamps {
        Stamps{
            count: s.count as uint64_t,
            first: s.first.map(|t| t.sec).unwrap_or(0)
                as uint64_t,
            last: s.last.map(|t| t.sec).unwrap_or(0)
                as uint64_t,
        }
    }
}

/// Statistics about bindings and stored keys.
///
/// We collect some data about binginds and stored keys.  This
/// information can be used to make informed decisions about key
/// transitions.
#[repr(C)]
pub struct Stats {
    /// Records the time this item was created.
    pub created: uint64_t,

    /// Records the time this item was last updated.
    pub updated: uint64_t,

    /// Records counters and timestamps of encryptions.
    pub encryption: Stamps,

    /// Records counters and timestamps of verifications.
    pub verification: Stamps,
}

impl Stats {
    fn new(s: sequoia_store::Stats) -> Stats {
        Stats {
            created: s.created.map(|t| t.sec).unwrap_or(0) as uint64_t,
            updated: s.updated.map(|t| t.sec).unwrap_or(0) as uint64_t,
            encryption: Stamps::new(&s.encryption),
            verification: Stamps::new(&s.verification),
        }
    }
}

/// Represents a log entry.
#[repr(C)]
pub struct Log {
    /// Records the time of the entry.
    pub timestamp: uint64_t,

    /// Relates the entry to a store.
    ///
    /// May be `NULL`.
    pub store: *mut Store,

    /// Relates the entry to a binding.
    ///
    /// May be `NULL`.
    pub binding: *mut Binding,

    /// Relates the entry to a key.
    ///
    /// May be `NULL`.
    pub key: *mut Key,

    /// Relates the entry to some object.
    ///
    /// This is a human-readable description of what this log entry is
    /// mainly concerned with.
    pub slug: *mut c_char,

    /// Holds the log message.
    pub status: *mut c_char,

    /// Holds the error message, if any.
    ///
    /// May be `NULL`.
    pub error: *mut c_char,
}
