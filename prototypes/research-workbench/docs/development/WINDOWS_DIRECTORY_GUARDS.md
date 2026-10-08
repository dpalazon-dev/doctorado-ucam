# Windows directory guards — F10 resolution

Coordinator decision, 2026-10-02; clarification of ADR-016. This supersedes the assumption that sharing flags prevent changes to reparse attributes. Implementation must meet this protocol before integrating T02. Research does not establish product validation.

## Scope and mechanism

The v0.1 library resides on a local Windows x64 NTFS volume. The adapter checks the volume from its handle and safely rejects remote/UNC paths and other filesystems; it does not test capabilities by writing into the library. This restriction concerns library storage, not selection of a source PDF. No resistance to administrators, drivers or arbitrary filters is promised.

Each directory in the authorized chain retains a handle with FILE_LIST_DIRECTORY and without FILE_SHARE_DELETE. Its next retained child prevents it from becoming empty. The final directory retains a child file opened with FILE_READ_DATA | FILE_READ_ATTRIBUTES | SYNCHRONIZE, sharing READ | WRITE without DELETE. After acquiring that child, check the same directory handle again: type, absence of reparse, and authorized identity/path. Only then publish the binding for path-based operations.

The mechanism does not depend on preventing FILE_WRITE_ATTRIBUTES through sharing. The directory remains non-empty, which blocks conversion to reparse within the adopted scope. An attributes-only handle is insufficient as a pin: the experiment showed that it still allowed deletion of the child.

## Acquisition and startup

1. Open and retain the local volume anchor. Walk each component with NtCreateFile relative to the parent handle, using one validated component, OBJ_CASE_INSENSITIVE | OBJ_DONT_REPARSE, FILE_OPEN_REPARSE_POINT and FILE_SYNCHRONOUS_IO_NONALERT. Retain earlier handles. Verify the acquired object and, after retaining the child, revalidate the parent. Do not create components through an absolute path that is still unprotected.
2. Open `research.sqlite` relatively with FILE_OPEN and read-pin access. The native open determines presence/absence; path.exists grants no authority. For an existing DB, that regular non-reparse file is the root pin: create no sentinels before the version probe. An error other than absence does not mean a new library.
3. Without a DB, initialization may create or relatively open the reserved `.rw-directory-pin` file inside the root. Revalidate the root after acquiring it, before allowing absolute paths. Do not create a DB as a substitute for the pin before the writer lock. After acquiring LibraryLock, create research.sqlite atomically with NtCreateFile FILE_CREATE and retain the new regular file with READ_DATA pin rights and without DELETE sharing until the connection closes. Only successful creation authorizes opening the new DB with SQLite. If a DB appears after classification as absent, FILE_CREATE must return a collision: return Busy without opening SQLite, preserve its bytes and retry through a new startup that classifies it as existing. An absence check followed by Connection::open does not satisfy this rule. Do not create the DB before the writer lock. Absence of all initialization artifacts is not promised when external DB creation occurs concurrently.
4. An existing root with a future-schema DB acquires only the DB pin and read guards: preserve the library inventory and bytes exactly; create no pin, writer lock, staging or manifest. The Store constructor remains free of I/O and recovery does not run.
5. Acquire a Store-managed directory relatively. Before path-based I/O, create/open `.rw-directory-pin` relatively, retain it and revalidate that same directory. If it became a reparse point before pin acquisition, fail without absolute fallback. Retain guards/pins through clones and all admitted jobs. LibraryRoot::at remains free of I/O; Store shares the actor's complete binding.

The root pin remains alive at least until SQLite has closed and admitted jobs have finished. Pins do not authorize file deletion or replace library, namespace, reference and hash checks. Promotion may retain FileRenameInfo with an absolute destination derived from the protected chain and ReplaceIfExists=false; reading/hashing/deletion remain on the same verified handle.

## Reserved file and persistence

`.rw-directory-pin` is an internal protection detail, created empty. If it exists, open it without truncation and verify that it is regular/non-reparse; its content is never authority and is neither executed nor interpreted. Do not create it in ancestors outside the authorized root. Do not delete it to clean up an import or use cleanup wildcards: remove only the PDFs/owned resources declared by the intent. Staging directories may retain this file after an operation completes.

Export and backup manifests omit these pins; they contain neither knowledge nor state required to recover an intent. Restore of a supported format regenerates them when acquiring directories in the new root. The manifest neither requires them nor grants authority to a copied pin. Switch/restore must await connection and job closure, release all guards on the previous root before operations requiring its rename, and keep that root recoverable. There is no automatic purge or migration change.

## FFI and tests

Keep windows-sys=0.61.2 and Foundation/Storage_FileSystem. Also authorize Wdk_Foundation, Wdk_Storage_FileSystem, Win32_Security and Win32_System_IO for the NtCreateFile/OBJECT_ATTRIBUTES/IO_STATUS_BLOCK binding. Win32_Security does not authorize ACL editing. Do not add NtSetInformationFile for promotion, a SQLite VFS, a generic executor or unsafe parser code.

The private wrapper uses closed types/layouts for directory/pin/file, never a UI path API. Validate a component without separators, colon, NUL, dot/double-dot; check UTF-16 length without truncation. Keep buffers and the parent handle alive during the synchronous call; check NTSTATUS before converting a handle to RAII. Every unsafe block documents its invariants. An incomplete binding must not be returned as a ready root.

Required regressions: FSCTL WRITE_ATTRIBUTES with a positive unguarded control and rejection145 under a pin; DELETE sharing violation and a control after closure; attacked relative acquisition before the pin without external writes; normal/no-clobber promotion; cleanup retains the pin; absent DB/revalidation; real writable SQLite/rusqlite with WAL, commit/checkpoint, closure and reopening; future schema with identical inventory/hashes. The experimental C# test replaces neither these Rust tests, independent security/Rust review nor installation validation.

Primary sources: [CreateFileW](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew), [FSCTL_SET_REPARSE_POINT](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-fsa/4aeefef8-92c3-4abc-af7a-a610caf8a165), [NtCreateFile](https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntcreatefile), [OBJECT_ATTRIBUTES](https://learn.microsoft.com/en-us/windows/win32/api/ntdef/ns-ntdef-_object_attributes). Experimental evidence and limits: task-02-windows-handle-ruling report and probe-2.log archived in docs/reviews/task-02/. Consulted and tested on Windows10.0.26200 x64 / C:NTFS; this does not certify all drivers or machines.
