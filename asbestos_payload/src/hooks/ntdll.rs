use std::{ffi::OsStr, os::windows::ffi::OsStrExt, ptr::NonNull};

use widestring::{U16CStr, U16Str};
use winapi::{
    shared::{
        ntdef::{
            NTSTATUS, OBJECT_ATTRIBUTES, PHANDLE, PLARGE_INTEGER, POBJECT_ATTRIBUTES, PVOID, ULONG,
            UNICODE_STRING, USHORT,
        },
        ntstatus,
    },
    um::winnt::ACCESS_MASK,
};

use asbestos_shared::log_error;

use super::decl_detour;

use crate::{
    missing_from_winapi::{PFILE_BASIC_INFORMATION, PIO_STATUS_BLOCK},
    vfs,
};

decl_detour!(
    "ntdll.dll",
    ntcreatefile,
    NTSTATUS NtCreateFile(
        PHANDLE            FileHandle,
        ACCESS_MASK        DesiredAccess,
        POBJECT_ATTRIBUTES ObjectAttributes,
        PIO_STATUS_BLOCK   IoStatusBlock,
        PLARGE_INTEGER     AllocationSize,
        ULONG              FileAttributes,
        ULONG              ShareAccess,
        ULONG              CreateDisposition,
        ULONG              CreateOptions,
        PVOID              EaBuffer,
        ULONG              EaLength
    ) {
        let mut conn = crate::get_conn();
        let conn = conn.as_mut().unwrap();

        if let Some(mut object_attributes) = NonNull::new(ObjectAttributes) {
            let object_attributes = unsafe { object_attributes.as_mut() };
            if let Some(object_name) = NonNull::new(object_attributes.ObjectName) {
                let object_name = unsafe { object_name.as_ref() };
                if !object_name.Buffer.is_null() {
                    // The following lines are a bit unfortunate. It seems like we can't trust the length reported by
                    // `ObjectAttributes.ObjectName.Length`. Treating the string like it is null-terminated seems to
                    // work, but just in case null-termination is ever incorrect, we log the result of both
                    // representations so that it may be discovered after the fact.
                    let object_name_1 = unsafe { U16Str::from_ptr(object_name.Buffer, object_name.Length as usize) };
                    let object_name_2 = unsafe { U16CStr::from_ptr_str(object_name.Buffer) };
                    log_info!(
                        conn,
                        "NtCreateFile(ObjectAttributes.ObjectName  (trust length) : {})\n\
                         NtCreateFile(ObjectAttributes.ObjectName (null-terminate): {})",
                        object_name_1.to_string_lossy(),
                        object_name_2.to_string_lossy(),
                    )
                    .ok();

                    let os_object_name_2 = object_name_2.to_os_string();
                    let utf8_object_name_2 = os_object_name_2.to_string_lossy();
                    match vfs::resolve_path(Some(conn), os_object_name_2.as_ref()) {
                        Err(err) => {
                            log_error!(conn, "Error while redirecting from {}: {}", utf8_object_name_2, err).ok();
                        }
                        Ok(redirected_object_name) => {
                            // Check if redirection is actully required.
                            if redirected_object_name != os_object_name_2 {
                                log_info!(
                                    conn,
                                    r#"Redirected "{}" to "{}""#,
                                    utf8_object_name_2,
                                    redirected_object_name.display()
                                )
                                .ok();

                                let redirected_object_name: &OsStr = redirected_object_name.as_ref().as_ref();
                                // Dropping `redirected_object_name` after it's been passed to `NtCreateFile` should be
                                // safe since it's not supposed to modify it in any way. If it does, then this may
                                // introduce a double-free/use-after-free.
                                let mut redirected_object_name: Vec<_> = redirected_object_name.encode_wide().collect();
                                let mut new_object_attributes = OBJECT_ATTRIBUTES {
                                    ObjectName: &mut UNICODE_STRING {
                                        Length: 2 * redirected_object_name.len() as USHORT,
                                        MaximumLength: 2 * redirected_object_name.capacity() as USHORT,
                                        Buffer: redirected_object_name.as_mut_ptr(),
                                    },
                                    ..*object_attributes
                                };

                                let res = unsafe {
                                    Hook.call(
                                        FileHandle,
                                        DesiredAccess,
                                        &mut new_object_attributes,
                                        IoStatusBlock,
                                        AllocationSize,
                                        FileAttributes,
                                        ShareAccess,
                                        CreateDisposition,
                                        CreateOptions,
                                        EaBuffer,
                                        EaLength,
                                    )
                                };

                                // Update the fields of `ObjectAttributes` just in case the call to `NtCreateFile`
                                // mutated anything. While it is very unlikely that `ObjectAttributes` will be mutated
                                // since its marked as an input, it's not entirely impossible for a bug to do so.
                                // If mutation does happen, then it's not up to us to fix or deal with it in any way.
                                object_attributes.Length = new_object_attributes.Length;
                                object_attributes.RootDirectory = new_object_attributes.RootDirectory;
                                // Don't update `object_attributes.ObjectName` since we've swapped that out.
                                object_attributes.Attributes = new_object_attributes.Attributes;
                                object_attributes.SecurityDescriptor = new_object_attributes.SecurityDescriptor;
                                object_attributes.SecurityQualityOfService = new_object_attributes.SecurityQualityOfService;

                                // If there's an error after we've modified the input, it's likely that there may be
                                // an error on our end. To check what values correspond to what constants, see:
                                // https://docs.microsoft.com/en-us/openspecs/windows_protocols/ms-erref/596a1078-e883-4972-9bbc-49e60bebca55
                                if res != ntstatus::STATUS_SUCCESS {
                                    log_error!(
                                        conn,
                                        r#"Error while redirecting `NtCreateFile` "{}": 0x{:X}. `RootDirectory` was 0x{:x}"#,
                                        utf8_object_name_2,
                                        res,
                                        object_attributes.RootDirectory as usize
                                    )
                                    .ok();
                                }

                                return res;
                            }
                        }
                    }
                }
            }
        }

        // If any of the inputs contain null pointers or if there's no need to redirect the call to `NtCreateFile` then
        // we pass through all the unmodified arguments to `NtCreateFile`.
        unsafe {
            Hook.call(
                FileHandle,
                DesiredAccess,
                ObjectAttributes,
                IoStatusBlock,
                AllocationSize,
                FileAttributes,
                ShareAccess,
                CreateDisposition,
                CreateOptions,
                EaBuffer,
                EaLength,
            )
        }
    }
);

decl_detour!(
    "ntdll.dll",
    ntqueryattributesfile,
    NTSTATUS NtQueryAttributesFile(
        POBJECT_ATTRIBUTES      ObjectAttributes,
        PFILE_BASIC_INFORMATION FileInformation
    ) {
        let mut conn = crate::get_conn();
        let conn = conn.as_mut().unwrap();

        if let Some(mut object_attributes) = NonNull::new(ObjectAttributes) {
            let object_attributes = unsafe { object_attributes.as_mut() };
            if let Some(object_name) = NonNull::new(object_attributes.ObjectName) {
                let object_name = unsafe { object_name.as_ref() };
                if !object_name.Buffer.is_null() {
                    // The following lines are a bit unfortunate. It seems like we can't trust the length reported by
                    // `ObjectAttributes.ObjectName.Length`. Treating the string like it is null-terminated seems to
                    // work, but just in case null-termination is ever incorrect, we log the result of both
                    // representations so that it may be discovered after the fact.
                    let object_name_1 = unsafe { U16Str::from_ptr(object_name.Buffer, object_name.Length as usize) };
                    let object_name_2 = unsafe { U16CStr::from_ptr_str(object_name.Buffer) };
                    log_info!(
                        conn,
                        "NtQueryAttributesFile(ObjectAttributes.ObjectName  (trust length) : {})\n\
                         NtQueryAttributesFile(ObjectAttributes.ObjectName (null-terminate): {})",
                        object_name_1.to_string_lossy(),
                        object_name_2.to_string_lossy(),
                    )
                    .ok();

                    let os_object_name_2 = object_name_2.to_os_string();
                    let utf8_object_name_2 = os_object_name_2.to_string_lossy();
                    match vfs::resolve_path(Some(conn), os_object_name_2.as_ref()) {
                        Err(err) => {
                            log_error!(conn, "Error while redirecting from {}: {}", utf8_object_name_2, err).ok();
                        }
                        Ok(redirected_object_name) => {
                            // Check if redirection is actully required.
                            if redirected_object_name != os_object_name_2 {
                                log_info!(
                                    conn,
                                    r#"Redirected "{}" to "{}""#,
                                    utf8_object_name_2,
                                    redirected_object_name.display()
                                )
                                .ok();

                                let redirected_object_name: &OsStr = redirected_object_name.as_ref().as_ref();
                                // Dropping `redirected_object_name` after it's been passed to `NtQueryAttributesFile` should be
                                // safe since it's not supposed to modify it in any way. If it does, then this may
                                // introduce a double-free/use-after-free.
                                let mut redirected_object_name: Vec<_> = redirected_object_name.encode_wide().collect();
                                let mut new_object_attributes = OBJECT_ATTRIBUTES {
                                    ObjectName: &mut UNICODE_STRING {
                                        Length: 2 * redirected_object_name.len() as USHORT,
                                        MaximumLength: 2 * redirected_object_name.capacity() as USHORT,
                                        Buffer: redirected_object_name.as_mut_ptr(),
                                    },
                                    ..*object_attributes
                                };

                                let res = unsafe {
                                    Hook.call(
                                        &mut new_object_attributes,
                                        FileInformation,
                                    )
                                };

                                // Update the fields of `ObjectAttributes` just in case the call to `NtQueryAttributesFile`
                                // mutated anything. While it is very unlikely that `ObjectAttributes` will be mutated
                                // since its marked as an input, it's not entirely impossible for a bug to do so.
                                // If mutation does happen, then it's not up to us to fix or deal with it in any way.
                                object_attributes.Length = new_object_attributes.Length;
                                object_attributes.RootDirectory = new_object_attributes.RootDirectory;
                                // Don't update `object_attributes.ObjectName` since we've swapped that out.
                                object_attributes.Attributes = new_object_attributes.Attributes;
                                object_attributes.SecurityDescriptor = new_object_attributes.SecurityDescriptor;
                                object_attributes.SecurityQualityOfService = new_object_attributes.SecurityQualityOfService;

                                // If there's an error after we've modified the input, it's likely that there may be
                                // an error on our end. To check what values correspond to what constants, see:
                                // https://docs.microsoft.com/en-us/openspecs/windows_protocols/ms-erref/596a1078-e883-4972-9bbc-49e60bebca55
                                if res != ntstatus::STATUS_SUCCESS {
                                    log_error!(
                                        conn,
                                        r#"Error while redirecting `NtQueryAttributesFile` "{}": 0x{:X}. `RootDirectory` was 0x{:x}"#,
                                        utf8_object_name_2,
                                        res,
                                        object_attributes.RootDirectory as usize
                                    )
                                    .ok();
                                }

                                return res;
                            }
                        }
                    }
                }
            }
        }

        // If any of the inputs contain null pointers or if there's no need to redirect the call to `NtCreateFile` then
        // we pass through all the unmodified arguments to `NtCreateFile`.
        unsafe {
            Hook.call(
                ObjectAttributes,
                FileInformation,
            )
        }
    }
);
