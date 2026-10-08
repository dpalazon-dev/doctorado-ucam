//! Windows handle-based identity and managed-file operations.
//!
//! This module is the only place allowed to use Windows handle FFI for library files. Directory
//! handles deny write/delete sharing and stay alive for the lifetime of the bound root.

use crate::transport::error::{AppError, ErrorCode};
use std::{
    fs::{File, OpenOptions},
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagedOpenCause {
    Missing,
    AccessDenied,
    SharingViolation,
}

#[derive(Debug)]
pub enum ManagedOpenError {
    Os(ManagedOpenCause),
    Fatal(AppError),
}

impl From<AppError> for ManagedOpenError {
    fn from(error: AppError) -> Self {
        Self::Fatal(error)
    }
}

impl From<ManagedOpenError> for AppError {
    fn from(error: ManagedOpenError) -> Self {
        match error {
            ManagedOpenError::Os(ManagedOpenCause::Missing) => Self::new(ErrorCode::NotFound),
            ManagedOpenError::Os(
                ManagedOpenCause::AccessDenied | ManagedOpenCause::SharingViolation,
            ) => Self::new(ErrorCode::StorageUnavailable),
            ManagedOpenError::Fatal(error) => error,
        }
    }
}

#[cfg(windows)]
fn classify_ntstatus(status: i32) -> ManagedOpenError {
    use windows_sys::Win32::Foundation::{
        STATUS_ACCESS_DENIED, STATUS_OBJECT_NAME_COLLISION, STATUS_OBJECT_NAME_NOT_FOUND,
        STATUS_OBJECT_PATH_NOT_FOUND, STATUS_SHARING_VIOLATION,
    };

    if status == STATUS_OBJECT_NAME_NOT_FOUND || status == STATUS_OBJECT_PATH_NOT_FOUND {
        ManagedOpenError::Os(ManagedOpenCause::Missing)
    } else if status == STATUS_ACCESS_DENIED {
        ManagedOpenError::Os(ManagedOpenCause::AccessDenied)
    } else if status == STATUS_SHARING_VIOLATION {
        ManagedOpenError::Os(ManagedOpenCause::SharingViolation)
    } else if status == STATUS_OBJECT_NAME_COLLISION {
        ManagedOpenError::Fatal(AppError::new(ErrorCode::Conflict))
    } else {
        ManagedOpenError::Fatal(AppError::new(ErrorCode::StorageUnavailable))
    }
}

#[derive(Debug)]
pub struct ManagedRoot {
    canonical_path: PathBuf,
    // Retaining every ancestor handle prevents renaming or replacing any component in the
    // authorized path while this root identity is in use.
    _ancestors: Vec<Arc<File>>,
    _root_pin: std::sync::Mutex<Option<Arc<File>>>,
    database_pin: std::sync::Mutex<Option<Arc<File>>>,
}

impl ManagedRoot {
    pub fn bind(path: &Path, create_missing: bool) -> Result<Arc<Self>, AppError> {
        #[cfg(windows)]
        {
            bind_windows(path, create_missing).map(Arc::new)
        }
        #[cfg(not(windows))]
        {
            if create_missing {
                std::fs::create_dir_all(path)
                    .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?;
            }
            let canonical_path = std::fs::canonicalize(path)
                .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?;
            let database_path = canonical_path.join("research.sqlite");
            let database = if database_path.exists() {
                Some(Arc::new(
                    File::open(database_path)
                        .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?,
                ))
            } else {
                let pin = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .truncate(false)
                    .open(canonical_path.join(".rw-directory-pin"))
                    .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?;
                Some(Arc::new(pin))
            };
            let db_exists = database_path.exists();
            let root_pin = if db_exists { None } else { database.clone() };
            let root_handle = Arc::new(
                File::open(&canonical_path)
                    .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?,
            );
            Ok(Arc::new(Self {
                canonical_path,
                _ancestors: vec![root_handle],
                _root_pin: std::sync::Mutex::new(root_pin),
                database_pin: std::sync::Mutex::new(if db_exists { database } else { None }),
            }))
        }
    }

    pub fn canonical_path(&self) -> &Path {
        &self.canonical_path
    }

    pub fn database_exists(&self) -> Result<bool, AppError> {
        self.database_pin
            .lock()
            .map(|pin| pin.is_some())
            .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))
    }

    pub fn create_database_after_lock(&self) -> Result<(), AppError> {
        let mut database_pin = self
            .database_pin
            .lock()
            .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?;
        if database_pin.is_some() {
            return Err(AppError::new(ErrorCode::Busy));
        }
        let root = self
            ._ancestors
            .last()
            .ok_or_else(|| AppError::new(ErrorCode::PathNotAllowed))?;
        #[cfg(windows)]
        let file = open_relative_file(
            root,
            "research.sqlite",
            RelativeFileAccess::ReadPin,
            RelativeFileDisposition::Create,
        )
        .map_err(AppError::from)
        .map_err(|error| {
            if error.code == ErrorCode::Conflict {
                AppError::new(ErrorCode::Busy)
            } else {
                error
            }
        })?;
        #[cfg(not(windows))]
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(self.canonical_path.join("research.sqlite"))
            .map_err(|error| {
                if error.kind() == std::io::ErrorKind::AlreadyExists {
                    AppError::new(ErrorCode::Busy)
                } else {
                    AppError::new(ErrorCode::StorageUnavailable)
                }
            })?;
        verify_regular_file(&file)?;
        verify_file_parent(&self.canonical_path, &file)?;
        verify_same_directory(root)?;
        *database_pin = Some(Arc::new(file));
        Ok(())
    }
}

#[cfg(windows)]
fn bind_windows(path: &Path, create_missing: bool) -> Result<ManagedRoot, AppError> {
    use std::os::windows::fs::OpenOptionsExt;
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_LIST_DIRECTORY,
        FILE_READ_ATTRIBUTES, FILE_SHARE_READ, FILE_SHARE_WRITE, FILE_TRAVERSE,
        GetVolumeInformationByHandleW, SYNCHRONIZE,
    };

    let Some(prefix) = path.components().next() else {
        return Err(AppError::new(ErrorCode::PathNotAllowed));
    };
    if !path.is_absolute()
        || !matches!(prefix, std::path::Component::Prefix(p) if matches!(p.kind(), std::path::Prefix::Disk(_)))
    {
        return Err(AppError::new(ErrorCode::PathNotAllowed));
    }
    let mut volume_path = prefix.as_os_str().to_os_string();
    volume_path.push("\\");
    let volume = PathBuf::from(volume_path);
    let mut options = OpenOptions::new();
    options
        .read(true)
        .access_mode(FILE_LIST_DIRECTORY | FILE_TRAVERSE | FILE_READ_ATTRIBUTES | SYNCHRONIZE)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT);
    let volume_handle = options
        .open(&volume)
        .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?;
    verify_directory(&volume_handle)?;
    let mut fs_name = [0u16; 64];
    // SAFETY: `volume_handle` is live, and the fixed UTF-16 buffer is writable for its length.
    let volume_ok = unsafe {
        GetVolumeInformationByHandleW(
            std::os::windows::io::AsRawHandle::as_raw_handle(&volume_handle),
            std::ptr::null_mut(),
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            fs_name.as_mut_ptr(),
            fs_name.len() as u32,
        )
    };
    if volume_ok == 0 || String::from_utf16_lossy(&fs_name).trim_end_matches('\0') != "NTFS" {
        return Err(AppError::new(ErrorCode::PathNotAllowed));
    }
    let mut handles = vec![Arc::new(volume_handle)];
    let mut canonical_path = final_path(handles.last().unwrap())?;
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::Normal(name) => components.push(
                name.to_str()
                    .ok_or_else(|| AppError::new(ErrorCode::PathNotAllowed))?,
            ),
            std::path::Component::Prefix(_) | std::path::Component::RootDir => {}
            std::path::Component::CurDir | std::path::Component::ParentDir => {
                return Err(AppError::new(ErrorCode::PathNotAllowed));
            }
        }
    }
    let component_count = components.len();
    for (index, component) in components.drain(..).enumerate() {
        validate_component(component)?;
        let parent = handles.last().unwrap();
        let (child, _) = open_relative_directory(parent, component, create_missing, false)?;
        verify_directory(&child)?;
        verify_same_directory(parent)?;
        canonical_path = final_path(&child)?;
        handles.push(Arc::new(child));
        if index == component_count - 1 {
            verify_same_directory(handles.last().unwrap())?;
        }
    }
    if handles.len() < 2 {
        return Err(AppError::new(ErrorCode::PathNotAllowed));
    }
    let root = handles.last().unwrap();
    let database = match open_relative_file(
        root,
        "research.sqlite",
        RelativeFileAccess::ReadPin,
        RelativeFileDisposition::Open,
    )
    .map_err(AppError::from)
    {
        Ok(database) => Some(Arc::new(database)),
        Err(error) if error.code == ErrorCode::NotFound => None,
        Err(error) => return Err(error),
    };
    if let Some(database) = &database {
        verify_regular_file(database)?;
        verify_file_parent(&canonical_path, database)?;
        verify_same_directory(root)?;
    }
    let root_pin = if database.is_none() {
        Some(Arc::new(open_relative_file(
            root,
            ".rw-directory-pin",
            RelativeFileAccess::ReadPin,
            RelativeFileDisposition::OpenOrCreate,
        )?))
    } else {
        None
    };
    if let Some(pin) = &root_pin {
        verify_regular_file(pin)?;
        verify_file_parent(&canonical_path, pin)?;
        verify_same_directory(root)?;
    }
    Ok(ManagedRoot {
        canonical_path,
        _ancestors: handles,
        _root_pin: std::sync::Mutex::new(root_pin),
        database_pin: std::sync::Mutex::new(database),
    })
}

#[derive(Clone)]
pub struct ManagedDirectory {
    root: Arc<ManagedRoot>,
    path: PathBuf,
    handle: Arc<File>,
    _pin: Arc<File>,
    ancestors: Vec<Arc<File>>,
}

impl ManagedRoot {
    pub fn directory(root: &Arc<Self>) -> Result<ManagedDirectory, AppError> {
        let handle = root
            ._ancestors
            .last()
            .cloned()
            .ok_or_else(|| AppError::new(ErrorCode::PathNotAllowed))?;
        let pin = root
            .database_pin
            .lock()
            .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?
            .clone()
            .or_else(|| root._root_pin.lock().ok()?.clone())
            .ok_or_else(|| AppError::new(ErrorCode::PathNotAllowed))?;
        Ok(ManagedDirectory {
            root: root.clone(),
            path: root.canonical_path.clone(),
            handle,
            _pin: pin,
            ancestors: root._ancestors.clone(),
        })
    }
}

impl ManagedDirectory {
    pub fn open_child(&self, name: &str, create: bool) -> Result<Self, AppError> {
        self.open_child_inner(name, create).map_err(AppError::from)
    }

    pub fn open_child_for_document(&self, name: &str) -> Result<Self, ManagedOpenError> {
        self.open_child_inner(name, false)
    }

    fn open_child_inner(&self, name: &str, create: bool) -> Result<Self, ManagedOpenError> {
        validate_component(name)?;
        let (handle, _) = open_relative_directory(&self.handle, name, create, true)?;
        verify_directory(&handle)?;
        let path = final_path(&handle)?;
        let pin = Arc::new(open_relative_file(
            &handle,
            ".rw-directory-pin",
            RelativeFileAccess::ReadPin,
            RelativeFileDisposition::OpenOrCreate,
        )?);
        verify_regular_file(&pin)?;
        verify_file_parent(&path, &pin)?;
        verify_same_directory(&self.handle)?;
        verify_directory(&handle)?;
        verify_descendant(self.root.canonical_path(), &handle)?;
        let handle = Arc::new(handle);
        let mut ancestors = self.ancestors.clone();
        ancestors.push(handle.clone());
        Ok(Self {
            root: self.root.clone(),
            path,
            handle,
            _pin: pin,
            ancestors,
        })
    }

    pub fn open_child_if_exists(&self, name: &str) -> Result<Option<Self>, AppError> {
        validate_component(name)?;
        match self.open_child(name, false) {
            Ok(directory) => Ok(Some(directory)),
            Err(error) if error.code == ErrorCode::NotFound => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub fn create_file(&self, name: &str) -> Result<ManagedFile, AppError> {
        validate_component(name)?;
        let path = self.path.join(name);
        let handle = open_relative_file(
            &self.handle,
            name,
            RelativeFileAccess::Managed,
            RelativeFileDisposition::Create,
        )?;
        verify_regular_file(&handle)?;
        verify_file_parent(&self.path, &handle)?;
        Ok(ManagedFile {
            file: handle,
            parent: self.clone(),
            name: name.to_owned(),
            path,
        })
    }

    pub fn open_file(&self, name: &str) -> Result<ManagedFile, AppError> {
        self.open_file_inner(name).map_err(AppError::from)
    }

    pub fn open_file_for_document(&self, name: &str) -> Result<ManagedFile, ManagedOpenError> {
        self.open_file_inner(name)
    }

    fn open_file_inner(&self, name: &str) -> Result<ManagedFile, ManagedOpenError> {
        validate_component(name)?;
        let path = self.path.join(name);
        let handle = open_relative_file(
            &self.handle,
            name,
            RelativeFileAccess::Managed,
            RelativeFileDisposition::Open,
        )?;
        verify_regular_file(&handle)?;
        verify_file_parent(&self.path, &handle)?;
        Ok(ManagedFile {
            file: handle,
            parent: self.clone(),
            name: name.to_owned(),
            path,
        })
    }

    pub fn open_file_if_exists(&self, name: &str) -> Result<Option<ManagedFile>, AppError> {
        validate_component(name)?;
        match self.open_file(name) {
            Ok(file) => Ok(Some(file)),
            Err(error) if error.code == ErrorCode::NotFound => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub fn sync_all(&self) -> Result<(), AppError> {
        self.handle
            .sync_all()
            .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

pub struct ManagedFile {
    file: File,
    parent: ManagedDirectory,
    name: String,
    path: PathBuf,
}

impl ManagedFile {
    pub fn file_mut(&mut self) -> &mut File {
        &mut self.file
    }

    pub fn sync_all(&self) -> Result<(), AppError> {
        self.file
            .sync_all()
            .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))
    }

    pub fn size(&self) -> Result<u64, AppError> {
        self.file
            .metadata()
            .map(|metadata| metadata.len())
            .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))
    }

    pub fn read_bounded(&mut self, limit: u64) -> Result<Vec<u8>, AppError> {
        use std::io::{Read, Seek, SeekFrom};
        self.file
            .seek(SeekFrom::Start(0))
            .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?;
        let mut bytes = Vec::new();
        let mut bounded = (&mut self.file).take(limit.saturating_add(1));
        bounded
            .read_to_end(&mut bytes)
            .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?;
        if bytes.len() as u64 > limit {
            return Err(AppError::new(ErrorCode::InvalidInput));
        }
        Ok(bytes)
    }

    pub fn remove(self) -> Result<(), AppError> {
        #[cfg(windows)]
        {
            use windows_sys::Win32::Storage::FileSystem::{
                FILE_DISPOSITION_INFO, FileDispositionInfo, SetFileInformationByHandle,
            };
            let mut disposition = FILE_DISPOSITION_INFO { DeleteFile: true };
            // SAFETY: `self.file` is a live handle opened with DELETE access. The disposition
            // record is initialized, correctly aligned, and its exact size is passed to Win32.
            let ok = unsafe {
                SetFileInformationByHandle(
                    std::os::windows::io::AsRawHandle::as_raw_handle(&self.file),
                    FileDispositionInfo,
                    (&mut disposition as *mut FILE_DISPOSITION_INFO).cast(),
                    std::mem::size_of::<FILE_DISPOSITION_INFO>() as u32,
                )
            };
            if ok == 0 {
                return Err(AppError::new(ErrorCode::StorageUnavailable));
            }
            Ok(())
        }
        #[cfg(not(windows))]
        {
            std::fs::remove_file(&self.path)
                .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))
        }
    }

    pub fn rename_no_replace(
        mut self,
        destination: &ManagedDirectory,
        name: &str,
    ) -> Result<Self, AppError> {
        validate_component(name)?;
        #[cfg(windows)]
        rename_handle_no_replace(&self.file, &destination.path.join(name))?;
        #[cfg(not(windows))]
        {
            let target = destination.path.join(name);
            if target.exists() {
                return Err(AppError::new(ErrorCode::Conflict));
            }
            std::fs::rename(&self.path, &target)
                .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?;
        }
        self.parent = destination.clone();
        self.name = name.to_owned();
        self.path = destination.path.join(name);
        Ok(self)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn parent(&self) -> &ManagedDirectory {
        &self.parent
    }
}

fn validate_component(name: &str) -> Result<(), AppError> {
    if name.is_empty()
        || name == "."
        || name == ".."
        || name.contains('\0')
        || name.contains(['/', '\\', ':'])
        || Path::new(name).components().count() != 1
        || name.encode_utf16().count() > (u16::MAX as usize / 2)
    {
        return Err(AppError::new(ErrorCode::PathNotAllowed));
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum RelativeFileAccess {
    ReadPin,
    Managed,
}

#[derive(Clone, Copy)]
enum RelativeFileDisposition {
    Open,
    OpenOrCreate,
    Create,
}

#[cfg(windows)]
fn open_relative_directory(
    parent: &File,
    name: &str,
    create: bool,
    writable: bool,
) -> Result<(File, bool), ManagedOpenError> {
    use std::os::windows::io::{AsRawHandle, FromRawHandle};
    use windows_sys::{
        Wdk::{
            Foundation::OBJECT_ATTRIBUTES,
            Storage::FileSystem::{
                FILE_DIRECTORY_FILE, FILE_OPEN, FILE_OPEN_IF, FILE_OPEN_REPARSE_POINT,
                FILE_SYNCHRONOUS_IO_NONALERT, NtCreateFile,
            },
        },
        Win32::{
            Foundation::{OBJ_CASE_INSENSITIVE, OBJ_DONT_REPARSE, UNICODE_STRING},
            Storage::FileSystem::{
                FILE_ADD_FILE, FILE_ADD_SUBDIRECTORY, FILE_LIST_DIRECTORY, FILE_READ_ATTRIBUTES,
                FILE_SHARE_READ, FILE_SHARE_WRITE, FILE_TRAVERSE, SYNCHRONIZE,
            },
            System::IO::IO_STATUS_BLOCK,
        },
    };

    validate_component(name)?;
    let mut name_wide = name.encode_utf16().collect::<Vec<_>>();
    let byte_len = u16::try_from(
        name_wide
            .len()
            .checked_mul(2)
            .ok_or_else(|| AppError::new(ErrorCode::PathNotAllowed))?,
    )
    .map_err(|_| AppError::new(ErrorCode::PathNotAllowed))?;
    let mut object_name = UNICODE_STRING {
        Length: byte_len,
        MaximumLength: byte_len,
        Buffer: name_wide.as_mut_ptr(),
    };
    let attributes = OBJECT_ATTRIBUTES {
        Length: std::mem::size_of::<OBJECT_ATTRIBUTES>() as u32,
        RootDirectory: AsRawHandle::as_raw_handle(parent).cast(),
        ObjectName: &mut object_name,
        Attributes: OBJ_CASE_INSENSITIVE | OBJ_DONT_REPARSE,
        SecurityDescriptor: std::ptr::null(),
        SecurityQualityOfService: std::ptr::null(),
    };
    let mut status_block = IO_STATUS_BLOCK::default();
    let mut handle = std::ptr::null_mut();
    let disposition = if create { FILE_OPEN_IF } else { FILE_OPEN };
    let desired_access = FILE_LIST_DIRECTORY
        | FILE_READ_ATTRIBUTES
        | FILE_TRAVERSE
        | SYNCHRONIZE
        | if writable {
            FILE_ADD_FILE | FILE_ADD_SUBDIRECTORY
        } else {
            0
        };
    // SAFETY: all pointers remain live through the synchronous call; the object name is a single
    // validated component and RootDirectory is a retained, open directory handle.
    let status = unsafe {
        NtCreateFile(
            &mut handle,
            desired_access,
            &attributes,
            &mut status_block,
            std::ptr::null(),
            0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            disposition,
            FILE_DIRECTORY_FILE | FILE_OPEN_REPARSE_POINT | FILE_SYNCHRONOUS_IO_NONALERT,
            std::ptr::null(),
            0,
        )
    };
    if status < 0 {
        return Err(classify_ntstatus(status));
    }
    let created = status_block.Information == 2;
    // SAFETY: NtCreateFile returned success and transferred ownership of a valid kernel handle.
    Ok((unsafe { File::from_raw_handle(handle.cast()) }, created))
}

#[cfg(not(windows))]
fn open_relative_directory(
    parent: &File,
    name: &str,
    create: bool,
    _writable: bool,
) -> Result<(File, bool), ManagedOpenError> {
    let path = final_path(parent)?.join(name);
    let mut created = false;
    if create && !path.exists() {
        std::fs::create_dir(&path).map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?;
        created = true;
    }
    Ok((
        File::open(path).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                ManagedOpenError::Os(ManagedOpenCause::Missing)
            } else if error.kind() == std::io::ErrorKind::PermissionDenied {
                ManagedOpenError::Os(ManagedOpenCause::AccessDenied)
            } else {
                ManagedOpenError::Fatal(AppError::new(ErrorCode::StorageUnavailable))
            }
        })?,
        created,
    ))
}

#[cfg(windows)]
fn open_relative_file(
    parent: &File,
    name: &str,
    access: RelativeFileAccess,
    disposition: RelativeFileDisposition,
) -> Result<File, ManagedOpenError> {
    use std::os::windows::io::{AsRawHandle, FromRawHandle};
    use windows_sys::{
        Wdk::{
            Foundation::OBJECT_ATTRIBUTES,
            Storage::FileSystem::{
                FILE_CREATE, FILE_NON_DIRECTORY_FILE, FILE_OPEN, FILE_OPEN_IF,
                FILE_OPEN_REPARSE_POINT, FILE_SYNCHRONOUS_IO_NONALERT, NtCreateFile,
            },
        },
        Win32::{
            Foundation::{OBJ_CASE_INSENSITIVE, OBJ_DONT_REPARSE, UNICODE_STRING},
            Storage::FileSystem::{
                DELETE, FILE_READ_ATTRIBUTES, FILE_READ_DATA, FILE_SHARE_READ, FILE_SHARE_WRITE,
                FILE_WRITE_DATA, SYNCHRONIZE,
            },
            System::IO::IO_STATUS_BLOCK,
        },
    };

    validate_component(name)?;
    let mut name_wide = name.encode_utf16().collect::<Vec<_>>();
    let byte_len = u16::try_from(
        name_wide
            .len()
            .checked_mul(2)
            .ok_or_else(|| AppError::new(ErrorCode::PathNotAllowed))?,
    )
    .map_err(|_| AppError::new(ErrorCode::PathNotAllowed))?;
    let mut object_name = UNICODE_STRING {
        Length: byte_len,
        MaximumLength: byte_len,
        Buffer: name_wide.as_mut_ptr(),
    };
    let attributes = OBJECT_ATTRIBUTES {
        Length: std::mem::size_of::<OBJECT_ATTRIBUTES>() as u32,
        RootDirectory: AsRawHandle::as_raw_handle(parent).cast(),
        ObjectName: &mut object_name,
        Attributes: OBJ_CASE_INSENSITIVE | OBJ_DONT_REPARSE,
        SecurityDescriptor: std::ptr::null(),
        SecurityQualityOfService: std::ptr::null(),
    };
    let mut status_block = IO_STATUS_BLOCK::default();
    let mut handle = std::ptr::null_mut();
    let (desired_access, share_access) = match access {
        RelativeFileAccess::ReadPin => (
            FILE_READ_DATA | FILE_READ_ATTRIBUTES | SYNCHRONIZE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
        ),
        RelativeFileAccess::Managed => (
            FILE_READ_DATA | FILE_WRITE_DATA | FILE_READ_ATTRIBUTES | DELETE | SYNCHRONIZE,
            FILE_SHARE_READ,
        ),
    };
    let create_disposition = match disposition {
        RelativeFileDisposition::Open => FILE_OPEN,
        RelativeFileDisposition::OpenOrCreate => FILE_OPEN_IF,
        RelativeFileDisposition::Create => FILE_CREATE,
    };
    // SAFETY: the name and attributes stay alive through the synchronous call; the single
    // component and retained RootDirectory prevent traversal outside the authorized directory.
    let status = unsafe {
        NtCreateFile(
            &mut handle,
            desired_access,
            &attributes,
            &mut status_block,
            std::ptr::null(),
            0x80,
            share_access,
            create_disposition,
            FILE_NON_DIRECTORY_FILE | FILE_OPEN_REPARSE_POINT | FILE_SYNCHRONOUS_IO_NONALERT,
            std::ptr::null(),
            0,
        )
    };
    if status < 0 {
        return Err(classify_ntstatus(status));
    }
    // SAFETY: NtCreateFile returned success and transferred ownership of a valid kernel handle.
    Ok(unsafe { File::from_raw_handle(handle.cast()) })
}

#[cfg(not(windows))]
fn open_relative_file(
    parent: &File,
    name: &str,
    access: RelativeFileAccess,
    disposition: RelativeFileDisposition,
) -> Result<File, ManagedOpenError> {
    let path = final_path(parent)?.join(name);
    let mut options = OpenOptions::new();
    options.read(true);
    if matches!(access, RelativeFileAccess::Managed) {
        options.write(true);
    }
    match disposition {
        RelativeFileDisposition::Open => {}
        RelativeFileDisposition::OpenOrCreate => {
            options.create(true).truncate(false);
        }
        RelativeFileDisposition::Create => {
            options.create_new(true);
        }
    }
    options.open(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            ManagedOpenError::Os(ManagedOpenCause::Missing)
        } else if error.kind() == std::io::ErrorKind::PermissionDenied {
            ManagedOpenError::Os(ManagedOpenCause::AccessDenied)
        } else if error.kind() == std::io::ErrorKind::AlreadyExists {
            ManagedOpenError::Fatal(AppError::new(ErrorCode::Conflict))
        } else {
            ManagedOpenError::Fatal(AppError::new(ErrorCode::StorageUnavailable))
        }
    })
}

fn verify_same_directory(file: &File) -> Result<(), AppError> {
    verify_directory(file)
}

fn verify_directory(file: &File) -> Result<(), AppError> {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        use windows_sys::Win32::Storage::FileSystem::{
            FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_REPARSE_POINT,
        };
        let attributes = file
            .metadata()
            .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?
            .file_attributes();
        if attributes & FILE_ATTRIBUTE_DIRECTORY == 0
            || attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
        {
            return Err(AppError::new(ErrorCode::PathNotAllowed));
        }
    }
    #[cfg(not(windows))]
    if !file
        .metadata()
        .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?
        .is_dir()
    {
        return Err(AppError::new(ErrorCode::PathNotAllowed));
    }
    Ok(())
}

fn verify_regular_file(file: &File) -> Result<(), AppError> {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        use windows_sys::Win32::Storage::FileSystem::{
            FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_REPARSE_POINT,
        };
        let attributes = file
            .metadata()
            .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?
            .file_attributes();
        if attributes & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT) != 0 {
            return Err(AppError::new(ErrorCode::PathNotAllowed));
        }
    }
    #[cfg(not(windows))]
    if !file
        .metadata()
        .map_err(|_| AppError::new(ErrorCode::StorageUnavailable))?
        .is_file()
    {
        return Err(AppError::new(ErrorCode::PathNotAllowed));
    }
    Ok(())
}

fn verify_descendant(root: &Path, file: &File) -> Result<(), AppError> {
    let path = final_path(file)?;
    if !path.starts_with(root) || path == root {
        return Err(AppError::new(ErrorCode::PathNotAllowed));
    }
    Ok(())
}

fn verify_file_parent(parent: &Path, file: &File) -> Result<(), AppError> {
    let path = final_path(file)?;
    if path.parent() != Some(parent) {
        return Err(AppError::new(ErrorCode::PathNotAllowed));
    }
    Ok(())
}

#[cfg(windows)]
fn final_path(file: &File) -> Result<PathBuf, AppError> {
    use std::{ffi::OsString, os::windows::ffi::OsStringExt};
    use windows_sys::Win32::Storage::FileSystem::{GetFinalPathNameByHandleW, VOLUME_NAME_DOS};
    let mut buffer = vec![0u16; 32768];
    // SAFETY: the live file handle and writable UTF-16 buffer meet the API contract; buffer
    // length is exactly the maximum number of code units provided.
    let count = unsafe {
        GetFinalPathNameByHandleW(
            std::os::windows::io::AsRawHandle::as_raw_handle(file),
            buffer.as_mut_ptr(),
            buffer.len() as u32,
            VOLUME_NAME_DOS,
        )
    };
    if count == 0 || count as usize >= buffer.len() {
        return Err(AppError::new(ErrorCode::StorageUnavailable));
    }
    buffer.truncate(count as usize);
    Ok(PathBuf::from(OsString::from_wide(&buffer)))
}

#[cfg(not(windows))]
fn final_path(file: &File) -> Result<PathBuf, AppError> {
    use std::os::fd::AsRawFd;
    let link = PathBuf::from(format!("/proc/self/fd/{}", file.as_raw_fd()));
    std::fs::read_link(link).map_err(|_| AppError::new(ErrorCode::StorageUnavailable))
}

#[cfg(windows)]
fn rename_handle_no_replace(file: &File, destination: &Path) -> Result<(), AppError> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_RENAME_INFO, FILE_RENAME_INFO_0, FileRenameInfo, SetFileInformationByHandle,
    };
    let name: Vec<u16> = destination.as_os_str().encode_wide().collect();
    let name_bytes = name
        .len()
        .checked_mul(std::mem::size_of::<u16>())
        .and_then(|length| u32::try_from(length).ok())
        .ok_or_else(|| AppError::new(ErrorCode::InvalidInput))?;
    let name_offset = std::mem::offset_of!(FILE_RENAME_INFO, FileName);
    let total = name_offset
        .checked_add(name_bytes as usize)
        .ok_or_else(|| AppError::new(ErrorCode::InvalidInput))?;
    let words = total.div_ceil(std::mem::size_of::<usize>());
    let mut buffer = vec![0usize; words];
    let info = buffer.as_mut_ptr().cast::<FILE_RENAME_INFO>();
    // SAFETY: the Vec<usize> allocation is suitably aligned, sized for the fixed struct and
    // UTF-16 tail; every field is initialized before the call. The retained destination handle
    // chain prevents parent replacement, and ReplaceIfExists=false rejects a racing target.
    unsafe {
        std::ptr::write(
            info,
            FILE_RENAME_INFO {
                Anonymous: FILE_RENAME_INFO_0 {
                    ReplaceIfExists: false,
                },
                RootDirectory: std::ptr::null_mut(),
                FileNameLength: name_bytes,
                FileName: [0],
            },
        );
        std::ptr::copy_nonoverlapping(
            name.as_ptr(),
            std::ptr::addr_of_mut!((*info).FileName).cast::<u16>(),
            name.len(),
        );
        let ok = SetFileInformationByHandle(
            std::os::windows::io::AsRawHandle::as_raw_handle(file),
            FileRenameInfo,
            info.cast(),
            total as u32,
        );
        if ok == 0 {
            return Err(AppError::new(ErrorCode::StorageUnavailable));
        }
    }
    Ok(())
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use std::{fs, io::Write};

    #[test]
    fn ntstatus_open_mapping_classifies_only_the_three_expected_os_causes() {
        use windows_sys::Win32::Foundation::{
            STATUS_ACCESS_DENIED, STATUS_OBJECT_NAME_NOT_FOUND, STATUS_SHARING_VIOLATION,
            STATUS_UNSUCCESSFUL,
        };

        assert!(matches!(
            classify_ntstatus(STATUS_OBJECT_NAME_NOT_FOUND),
            ManagedOpenError::Os(ManagedOpenCause::Missing)
        ));
        assert!(matches!(
            classify_ntstatus(STATUS_ACCESS_DENIED),
            ManagedOpenError::Os(ManagedOpenCause::AccessDenied)
        ));
        assert!(matches!(
            classify_ntstatus(STATUS_SHARING_VIOLATION),
            ManagedOpenError::Os(ManagedOpenCause::SharingViolation)
        ));
        assert!(matches!(
            classify_ntstatus(STATUS_UNSUCCESSFUL),
            ManagedOpenError::Fatal(AppError {
                code: ErrorCode::StorageUnavailable,
                ..
            })
        ));
    }

    #[test]
    fn managed_document_open_reports_a_real_ntfs_sharing_violation() {
        let fixture = tempfile::tempdir().unwrap();
        let root = ManagedRoot::bind(fixture.path(), false).unwrap();
        let root_directory = ManagedRoot::directory(&root).unwrap();
        let documents = root_directory.open_child("documents", true).unwrap();
        let directory = documents
            .open_child("123e4567-e89b-42d3-a456-426614174000", true)
            .unwrap();
        let _held = directory.create_file("original.pdf").unwrap();

        assert!(matches!(
            directory.open_file_for_document("original.pdf"),
            Err(ManagedOpenError::Os(ManagedOpenCause::SharingViolation))
        ));
    }

    #[test]
    fn directory_pin_blocks_reparse_and_file_delete() {
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::{
            Foundation::{ERROR_DIR_NOT_EMPTY, ERROR_SHARING_VIOLATION},
            Storage::FileSystem::{
                FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_SHARE_DELETE,
                FILE_SHARE_READ, FILE_SHARE_WRITE, FILE_WRITE_ATTRIBUTES,
            },
        };

        let fixture = tempfile::tempdir().unwrap();
        let root = ManagedRoot::bind(fixture.path(), false).unwrap();
        let root_directory = ManagedRoot::directory(&root).unwrap();
        let guarded = root_directory.open_child("guarded", true).unwrap();
        let pin_path = guarded.path().join(".rw-directory-pin");
        assert!(pin_path.is_file());

        let empty = fixture.path().join("empty-control");
        let target = fixture.path().join("target-control");
        fs::create_dir(&empty).unwrap();
        fs::create_dir(&target).unwrap();
        let mut control = OpenOptions::new();
        control
            .read(true)
            .access_mode(FILE_WRITE_ATTRIBUTES)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT);
        let empty_handle = control.open(&empty).unwrap();
        // The same FSCTL succeeds on an ordinary empty directory, proving the guarded failure
        // below comes from the retained pin rather than privilege or test-environment limits.
        set_mount_point_reparse(&empty_handle, &target).unwrap();
        delete_reparse_point(&empty_handle);

        let guarded_handle = control.open(guarded.path()).unwrap();
        let guarded_result = set_mount_point_reparse(&guarded_handle, &target);
        assert!(guarded_result.is_err());
        // Capture the native error from the exact failing FSCTL call through the helper's return.
        assert_eq!(guarded_result.unwrap_err(), ERROR_DIR_NOT_EMPTY);

        let delete = fs::remove_file(&pin_path).unwrap_err();
        assert_eq!(delete.raw_os_error(), Some(ERROR_SHARING_VIOLATION as i32));
        drop(guarded);
        fs::remove_file(&pin_path).unwrap();
    }

    #[test]
    fn relative_acquisition_rejects_a_reparse_component_without_writing_outside() {
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::Storage::FileSystem::{
            FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_SHARE_DELETE,
            FILE_SHARE_READ, FILE_SHARE_WRITE, FILE_WRITE_ATTRIBUTES,
        };

        let fixture = tempfile::tempdir().unwrap();
        let root = ManagedRoot::bind(fixture.path(), false).unwrap();
        let root_directory = ManagedRoot::directory(&root).unwrap();
        let component = fixture.path().join("component");
        let outside = fixture.path().join("outside");
        fs::create_dir(&component).unwrap();
        fs::create_dir(&outside).unwrap();
        let mut options = OpenOptions::new();
        options
            .read(true)
            .access_mode(FILE_WRITE_ATTRIBUTES)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT);
        let component_handle = options.open(&component).unwrap();
        set_mount_point_reparse(&component_handle, &outside).unwrap();

        let error = match root_directory.open_child("component", false) {
            Ok(_) => panic!("a reparse directory was accepted"),
            Err(error) => error,
        };
        assert_eq!(error.code, ErrorCode::PathNotAllowed);
        assert_eq!(fs::read_dir(&outside).unwrap().count(), 0);
        delete_reparse_point(&component_handle);
    }

    fn set_mount_point_reparse(handle: &File, target: &Path) -> Result<(), u32> {
        use std::os::windows::ffi::OsStrExt;
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::{Foundation::GetLastError, System::IO::DeviceIoControl};

        const FSCTL_SET_REPARSE_POINT: u32 = 0x0009_00A4;
        const IO_REPARSE_TAG_MOUNT_POINT: u32 = 0xA000_0003;
        let target_path = target
            .to_str()
            .ok_or(87u32)?
            .strip_prefix("\\\\?\\")
            .unwrap_or(target.to_str().ok_or(87u32)?);
        let substitute = format!("\\??\\{target_path}")
            .encode_utf16()
            .collect::<Vec<_>>();
        let printable = target.as_os_str().encode_wide().collect::<Vec<_>>();
        let substitute_bytes = u16::try_from(substitute.len() * 2).map_err(|_| 87u32)?;
        let print_offset = substitute_bytes.checked_add(2).ok_or(87u32)?;
        let print_bytes = u16::try_from(printable.len() * 2).map_err(|_| 87u32)?;
        let mut buffer = Vec::with_capacity(16 + (substitute.len() + printable.len() + 2) * 2);
        buffer.extend_from_slice(&IO_REPARSE_TAG_MOUNT_POINT.to_le_bytes());
        let data_len =
            u16::try_from(8 + (substitute.len() + printable.len() + 2) * 2).map_err(|_| 87u32)?;
        buffer.extend_from_slice(&data_len.to_le_bytes());
        buffer.extend_from_slice(&0u16.to_le_bytes());
        buffer.extend_from_slice(&0u16.to_le_bytes());
        buffer.extend_from_slice(&substitute_bytes.to_le_bytes());
        buffer.extend_from_slice(&print_offset.to_le_bytes());
        buffer.extend_from_slice(&print_bytes.to_le_bytes());
        for unit in substitute
            .into_iter()
            .chain([0])
            .chain(printable)
            .chain([0])
        {
            buffer.extend_from_slice(&unit.to_le_bytes());
        }
        let mut returned = 0;
        // SAFETY: `handle` remains live and `buffer` contains a fully initialized mount-point
        // REPARSE_DATA_BUFFER; no output buffer is requested by FSCTL_SET_REPARSE_POINT.
        let ok = unsafe {
            DeviceIoControl(
                AsRawHandle::as_raw_handle(handle),
                FSCTL_SET_REPARSE_POINT,
                buffer.as_ptr().cast(),
                buffer.len() as u32,
                std::ptr::null_mut(),
                0,
                &mut returned,
                std::ptr::null_mut(),
            )
        };
        if ok == 0 {
            Err(unsafe { GetLastError() })
        } else {
            Ok(())
        }
    }

    fn delete_reparse_point(handle: &File) {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::System::IO::DeviceIoControl;
        const FSCTL_DELETE_REPARSE_POINT: u32 = 0x0009_00AC;
        const IO_REPARSE_TAG_MOUNT_POINT: u32 = 0xA000_0003;
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&IO_REPARSE_TAG_MOUNT_POINT.to_le_bytes());
        buffer.extend_from_slice(&0u16.to_le_bytes());
        buffer.extend_from_slice(&0u16.to_le_bytes());
        let mut returned = 0;
        // SAFETY: the handle is live and the delete structure contains the required tag and
        // zero-length reserved fields for the mount-point reparse type.
        let ok = unsafe {
            DeviceIoControl(
                AsRawHandle::as_raw_handle(handle),
                FSCTL_DELETE_REPARSE_POINT,
                buffer.as_ptr().cast(),
                buffer.len() as u32,
                std::ptr::null_mut(),
                0,
                &mut returned,
                std::ptr::null_mut(),
            )
        };
        assert_ne!(ok, 0, "failed to clean up positive-control reparse point");
    }

    #[test]
    fn retained_destination_parent_blocks_replacement_and_rename_is_no_clobber() {
        let fixture = tempfile::tempdir().unwrap();
        let root = ManagedRoot::bind(fixture.path(), false).unwrap();
        let root_directory = ManagedRoot::directory(&root).unwrap();
        fs::create_dir(fixture.path().join("destination")).unwrap();
        let destination = root_directory.open_child("destination", false).unwrap();

        assert!(
            fs::rename(
                fixture.path().join("destination"),
                fixture.path().join("moved")
            )
            .is_err()
        );

        let mut staged = root_directory.create_file("staged.pdf").unwrap();
        staged.file_mut().write_all(b"staged-original").unwrap();
        fs::write(destination.path().join("original.pdf"), b"racing-target").unwrap();

        let promoted = staged.rename_no_replace(&destination, "original.pdf");
        assert!(promoted.is_err());
        assert_eq!(
            fs::read(destination.path().join("original.pdf")).unwrap(),
            b"racing-target"
        );
        assert_eq!(
            fs::read(fixture.path().join("staged.pdf")).unwrap(),
            b"staged-original"
        );
    }

    #[test]
    fn database_create_after_lock_is_atomic_and_preserves_collision() {
        use crate::adapters::windows::{lock::LibraryLock, paths::LibraryRoot};

        let fixture = tempfile::tempdir().unwrap();
        let library_path = fixture.path().join("library");
        let mut root = LibraryRoot::at(library_path.clone());
        root.create_and_bind().unwrap();
        assert!(!root.database_exists().unwrap());
        let _lock = LibraryLock::acquire(&library_path).unwrap();

        let raced_bytes = b"racing database must remain unopened";
        fs::write(root.database(), raced_bytes).unwrap();
        let error = root.create_database_after_lock().unwrap_err();
        assert_eq!(error.code, ErrorCode::Busy);
        assert_eq!(fs::read(root.database()).unwrap(), raced_bytes);
    }
}
