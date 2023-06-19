use windows::Win32::System::WindowsProgramming::SYSTEM_PROCESS_INFORMATION;
use winapi::shared::ntdef::{ HRESULT, NTSTATUS, NT_SUCCESS };
use winapi::ctypes::*;
use winapi::um::memoryapi::*;
use winapi::um::processthreadsapi::*;
use winapi::um::winnt::{HANDLE,
    MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_EXECUTE_READWRITE, PROCESS_ALL_ACCESS
};
use ntapi::ntexapi::*;
use ntapi::ntexapi::SYSTEM_HANDLE_TABLE_ENTRY_INFO;
use ntapi::ntexapi::SYSTEM_HANDLE_INFORMATION;

struct SYSTEM_HANDLE_INFORMATION {
    pub NumberOfHandles: u32,
    pub Handles: [SYSTEM_HANDLE_TABLE_ENTRY_INFO; 1],
}

fn FillStructureFromMemory<T>(struct_ptr: &mut T, memory_ptr: *const c_void, process_handle: *mut c_void) {
    unsafe {
        let mut bytes_read: usize = 0;
        let res = ReadProcessMemory(process_handle, memory_ptr, struct_ptr as *mut _ as *mut c_void, std::mem::size_of::<T>(), &mut bytes_read);
        println!("res: {:x}", res);
        println!("bytes_read: {}", bytes_read);
    }
}

fn main() {
    unsafe {
        let mut info_length: u32 = 0x10000;
        let mut baseaddress = VirtualAlloc(std::ptr::null_mut(), info_length as usize, 0x1000, 0x40);

        let mut tries = 0;
        loop {
            let res = NtQuerySystemInformation(0x10, baseaddress, info_length, &mut info_length);

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

        let mut spi = SYSTEM_HANDLE_INFORMATION {
            NumberOfHandles: 0,
            Handles: [SYSTEM_HANDLE_TABLE_ENTRY_INFO {
                UniqueProcessId: 0,
                CreatorBackTraceIndex: 0,
                ObjectTypeIndex: 0,
                HandleAttributes: 0,
                HandleValue: 0,
                Object: 0 as *mut c_void,
                GrantedAccess: 0,
            }; 1],
        };    // from windows crate
        
        let a = GetCurrentProcess();
        FillStructureFromMemory(&mut spi, baseaddress as *const c_void, GetCurrentProcess());

        for i in 0..spi.NumberOfHandles {
            if spi.Handles[i].UniqueProcessId == GetCurrentProcessId() {
                println!("Handle: {:#x?}", spi.Handles[i].HandleValue);
            }
        }

        VirtualFree(baseaddress, 0x0, 0x8000);
    }
}
