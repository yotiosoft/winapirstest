use windows::Win32::System::WindowsProgramming::SYSTEM_PROCESS_INFORMATION;
use winapi::shared::ntdef::{ HRESULT, NTSTATUS, NT_SUCCESS };
use winapi::ctypes::*;
use winapi::um::memoryapi::*;
use winapi::um::processthreadsapi::*;
use winapi::um::handleapi::{ DuplicateHandle, CloseHandle, INVALID_HANDLE_VALUE };
use winapi::um::winnt::{HANDLE,
    MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_EXECUTE_READWRITE, PROCESS_DUP_HANDLE, DUPLICATE_SAME_ACCESS
};
use ntapi::ntexapi::*;
use ntapi::ntexapi::SYSTEM_HANDLE_TABLE_ENTRY_INFO_EX;
use ntapi::ntexapi::SystemExtendedHandleInformation;

fn FillStructureFromMemory<T>(struct_ptr: &mut T, memory_ptr: *const c_void, process_handle: *mut c_void) {
    unsafe {
        let mut bytes_read: usize = 0;
        let res = ReadProcessMemory(process_handle, memory_ptr, struct_ptr as *mut _ as *mut c_void, std::mem::size_of::<T>(), &mut bytes_read);
        //println!("res: {:x}", res);
        //println!("bytes_read: {}", bytes_read);
    }
}

fn main() {
    unsafe {
        let mut info_length: u32 = 0x10000;
        let mut baseaddress = VirtualAlloc(std::ptr::null_mut(), info_length as usize, 0x1000, 0x40);

        let mut tries = 0;
        loop {
            let res = NtQuerySystemInformation(SystemExtendedHandleInformation, baseaddress, info_length, &mut info_length);

            println!("res: {:x}", res);
            println!("info_length: {}", info_length);
            println!("tries: {}", tries);

            if res == 0 {
                break;
            }
            if tries == 5 {
                break;
            }

            VirtualFree(baseaddress, 0x0, 0x8000);

            baseaddress = VirtualAlloc(std::ptr::null_mut(), info_length as usize, 0x1000, 0x40);

            tries += 1;
        }

        println!("baseaddress: {:x?}", baseaddress);

        let mut spi: SYSTEM_HANDLE_INFORMATION_EX = std::mem::zeroed();
        
        let a = GetCurrentProcess();
        FillStructureFromMemory(&mut spi, baseaddress as *const c_void, GetCurrentProcess());

        println!("spi.NumberOfHandles: {}", spi.NumberOfHandles);

        // Handlesのオフセット
        let offset = std::mem::size_of::<usize>() * 2;

        for i in 0..spi.NumberOfHandles as usize {
            //if spi.Handles[i].UniqueProcessId as u32 == GetCurrentProcessId() {
            //    println!("Handle: {:#x?} pid: {:#x}", spi.handles[i].HandleValue, spi.handles[i].UniqueProcessId);
            //}

            let mut handle: SYSTEM_HANDLE_TABLE_ENTRY_INFO_EX = std::mem::zeroed();
            FillStructureFromMemory(&mut handle, (baseaddress as usize + (offset + i * std::mem::size_of::<SYSTEM_HANDLE_TABLE_ENTRY_INFO_EX>())) as *const c_void, GetCurrentProcess());
            if handle.ObjectTypeIndex == 0x18 {
                println!("Handle: {:#x?} pid: {:#x}", handle.ObjectTypeIndex, handle.UniqueProcessId);

                // プロセスを開く
                let proc_hand = OpenProcess(PROCESS_DUP_HANDLE, 0, handle.UniqueProcessId as u32);
                if proc_hand != INVALID_HANDLE_VALUE {
                    let mut file_hand: HANDLE = std::mem::zeroed();
                    if DuplicateHandle(proc_hand, handle.HandleValue as *mut c_void, GetCurrentProcess(), file_hand as *mut *mut c_void, 0, 0, DUPLICATE_SAME_ACCESS) == 0 {
                        println!("DuplicateHandle failed");
                        continue;
                    }

                    CloseHandle(proc_hand);

                    if file_hand != INVALID_HANDLE_VALUE {
                        let mut path: [u16; 0x100] = [0; 0x100];
                        let res = GetFinalPathNameByHandleW(file_hand, path.as_mut_ptr(), 0x100, 0);
                        if res != 0 {
                            println!("path: {:?}", std::ffi::OsString::from_wide(&path));
                        }
                        CloseHandle(file_hand);
                    }
                }
            }
        }

        VirtualFree(baseaddress, 0x0, 0x8000);
    }
}
