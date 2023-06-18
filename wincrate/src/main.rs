use windows::Win32::System::WindowsProgramming::SYSTEM_PROCESS_INFORMATION;
use windows::Win32::Foundation::HANDLE;
use winapi::shared::ntdef::{ HRESULT, NTSTATUS, NT_SUCCESS };
use winapi::ctypes::*;
use winapi::um::memoryapi::*;
use winapi::um::processthreadsapi::*;
use ntapi::ntexapi::*;

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
        while true {
            let res = NtQuerySystemInformation(0x5, baseaddress, info_length, &mut info_length);

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

        let mut spi = SYSTEM_PROCESS_INFORMATION::default();    // from windows crate
        let a = GetCurrentProcess();
        FillStructureFromMemory(&mut spi, baseaddress as *const c_void, GetCurrentProcess());

        println!("{:#x?}", spi);

        while true {
            if spi.NextEntryOffset == 0 {
                break;
            }

            let mut previous_addres = baseaddress;
            let mut next_address = baseaddress as isize + spi.NextEntryOffset as isize;

            FillStructureFromMemory(&mut spi, next_address as *const c_void, GetCurrentProcess());
            
            println!("{:#x?}", spi);

            baseaddress = next_address as *mut c_void;
        }   

        VirtualFree(baseaddress, 0x0, 0x8000);
    }
}
