//! Hooks for functions that deal with the file system.

// The following is a list of functions that may need to be hooked:
//
// [ ] CreateDirectoryA                            priority: high
// [ ] CreateDirectoryW                            priority: high
// [ ] CreateFile2 (W?) (A?)                       priority:
// [ ] CreateFileA                                 priority: high
// [ ] CreateFileW                                 priority: high
// [ ] ? DefineDosDeviceW (A?)                     priority:
// [ ] DeleteFileA                                 priority: high
// [ ] DeleteFileW                                 priority: high
// [ ] ? DeleteVolumeMountPointW (A?)              priority:
// [ ] ? FindClose                                 priority: high
// [ ] ? FindCloseChangeNotification               priority:
// [ ] FindFirstChangeNotificationA                priority:
// [ ] FindFirstChangeNotificationW                priority:
// [ ] FindFirstFileA                              priority: high
// [ ] FindFirstFileW                              priority: high
// [ ] FindFirstFileExA                            priority: high
// [ ] FindFirstFileExW                            priority: high
// [ ] FindFirstFileNameW (A?)                     priority: high
// [ ] ? FindFirstStreamW (A?)                     priority:
// [ ] ? FindFirstVolumeW (A?)                     priority:
// [ ] FindNextChangeNotification (W?) (A?)        priority:
// [ ] FindNextFileA                               priority: high
// [ ] FindNextFileW                               priority: high
// [ ] FindNextFileNameW (A?)                      priority: high
// [ ] ? FindNextStreamW (A?)                      priority:
// [ ] ? FindNextVolumeW (A?)                      priority:
// [ ] ? FindVolumeClose                           priority:
// [ ] GetCompressedFileSizeA                      priority:
// [ ] GetCompressedFileSizeW                      priority:
// [ ] ? GetDiskFreeSpaceA                         priority:
// [ ] ? GetDiskFreeSpaceW                         priority:
// [ ] ? GetDiskFreeSpaceExA                       priority:
// [ ] ? GetDiskFreeSpaceExW                       priority:
// [ ] ? GetDriveTypeA                             priority:
// [ ] ? GetDriveTypeW                             priority:
// [ ] GetFileAttributesA                          priority:
// [ ] GetFileAttributesExA                        priority:
// [ ] GetFileAttributesW                          priority:
// [ ] GetFileAttributesExW                        priority:
// [ ] ? GetFileInformationByHandle                priority:
// [ ] GetFileSize                                 priority:
// [ ] ? GetFinalPathNameByHandleA                 priority:
// [ ] ? GetFinalPathNameByHandleW                 priority:
// [ ] GetFullPathNameA                            priority: medium
// [ ] GetFullPathNameW                            priority: medium
// [ ] GetLongPathNameA                            priority: medium
// [ ] GetLongPathNameW                            priority: medium
// [ ] GetShortPathNameW (A?)                      priority: medium
// [ ] ? GetTempFileNameA                          priority:
// [ ] ? GetTempFileNameW                          priority:
// [ ] ? GetTempPathA                              priority:
// [ ] ? GetTempPathW                              priority:
// [ ] ? GetVolumeInformationA                     priority:
// [ ] ? GetVolumeInformationW                     priority:
// [ ] ? GetVolumeInformationByHandleW (A?)        priority:
// [ ] ? GetVolumeNameForVolumeMountPointW (A?)    priority:
// [ ] ? GetVolumePathnamesForVolumeNameW (A?)     priority:
// [ ] ? GetVolumePathNameW (A?)                   priority:
// [ ] ? QueryDosDeviceW (A?)                      priority:
// [ ] ? ReadFile                                  priority:
// [ ] ? ReadFileEx                                priority:
// [ ] ? ReadFileScatter                           priority:
// [ ] RemoveDirectoryA                            priority: medium
// [ ] RemoveDirectoryW                            priority: medium
// [ ] SetFileAttributesA                          priority:
// [ ] SetFileAttributesW                          priority:
// [ ] ? SetFileInformationByHandle                priority:
// [ ] ? WriteFile                                 priority:
// [ ] ? WriteFileEx                               priority:
// [ ] ? WriteFileGather                           priority:
