use winapi::shared::ntdef::{ HRESULT, NTSTATUS, NT_SUCCESS };
use windows::Win32::System::WindowsProgramming::SYSTEM_PROCESS_INFORMATION;
use winapi::ctypes::*;
use winapi::um::memoryapi::*;
use winapi::um::processthreadsapi::*;
use ntapi::ntexapi::*;

fn main() {
    unsafe {
        let mut info_length: u32 = 0x10000;
        let mut tries = 0;

        while true {
            let baseaddress = VirtualAlloc(std::ptr::null_mut(), info_length as usize, 0x1000, 0x40);

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

            tries += 1;
        }

        //println!("baseaddress: {:x?}", baseaddress);
    }
}
