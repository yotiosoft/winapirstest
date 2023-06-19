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

#[repr(C)]
pub struct SYSTEM_HANDLE_INFORMATION {
    pub number_of_handles: u32,
    pub handles: [SYSTEM_HANDLE_TABLE_ENTRY_INFO; 10000],
}

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

        let mut spi: SYSTEM_HANDLE_INFORMATION = std::mem::zeroed();
        
        let a = GetCurrentProcess();
        FillStructureFromMemory(&mut spi, baseaddress as *const c_void, GetCurrentProcess());

        println!("spi.NumberOfHandles: {}", spi.number_of_handles);

        for i in 0..spi.number_of_handles as usize {
            //if spi.Handles[i].UniqueProcessId as u32 == GetCurrentProcessId() {
            //    println!("Handle: {:#x?} pid: {:#x}", spi.handles[i].HandleValue, spi.handles[i].UniqueProcessId);
            //}

            let mut handle: SYSTEM_HANDLE_TABLE_ENTRY_INFO = std::mem::zeroed();
            FillStructureFromMemory(&mut handle, (baseaddress as usize + (8 + i * std::mem::size_of::<SYSTEM_HANDLE_TABLE_ENTRY_INFO>())) as *const c_void, GetCurrentProcess());
            if handle.UniqueProcessId as u32 == GetCurrentProcessId() {
                println!("Handle: {:#x?} pid: {:#x}", handle.HandleValue, handle.UniqueProcessId);
            }
        }

        VirtualFree(baseaddress, 0x0, 0x8000);
    }
}
