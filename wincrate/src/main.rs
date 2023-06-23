use winapi::ctypes::*;
use winapi::um::memoryapi::*;
use winapi::um::processthreadsapi::*;
use winapi::um::winnt::{ MEM_COMMIT, MEM_RELEASE, PAGE_EXECUTE_READWRITE };
use winapi::um::errhandlingapi::GetLastError;
use ntapi::ntexapi::*;

// 構造体にメモリから値を取得して格納する
fn fill_structure_from_memory<T>(struct_ptr: &mut T, memory_ptr: *const c_void, process_handle: *mut c_void) -> i32 {
    unsafe {
        let mut bytes_read: usize = 0;
        let res = ReadProcessMemory(process_handle, memory_ptr, struct_ptr as *mut _ as *mut c_void, std::mem::size_of::<T>(), &mut bytes_read);
        return res;
    }
}

// SystemProcessInformation を buffer に取得
fn get_system_process_information(mut buffer_size: u32) -> *mut c_void {
    unsafe {
        let mut base_address = VirtualAlloc(std::ptr::null_mut(), buffer_size as usize, MEM_COMMIT, PAGE_EXECUTE_READWRITE);

        let tries = 0;
        let max_tries = 5;
        loop {
            // プロセス情報を取得
            // SystemProcessInformation : 各プロセスの情報（オプション定数）
            // base_address             : 格納先
            // buffer_size              : 格納先のサイズ
            // &mut buffer_size         : 実際に取得したサイズ
            let res = NtQuerySystemInformation(SystemProcessInformation, base_address, buffer_size, &mut buffer_size);
            
            if res == 0 {
                break;
            }
            if tries == max_tries {
                break;
            }

            // realloc
            VirtualFree(base_address, 0, MEM_RELEASE);
            base_address = VirtualAlloc(std::ptr::null_mut(), buffer_size as usize, MEM_COMMIT, PAGE_EXECUTE_READWRITE);
        }

        return base_address;
    }
}

fn main() {
    unsafe {
        // プロセス情報を取得
        let base_address = get_system_process_information(0x10000);

        // base_address に取得したプロセス情報を SYSTEM_PROCESS_INFORMATION 構造体 system_process_info に格納
        let mut system_process_info: SYSTEM_PROCESS_INFORMATION = std::mem::zeroed();

        let mut next_address = base_address as isize;
        // すべてのプロセス情報を取得
        loop {
            // 次のプロセス情報の格納先アドレス
            next_address += system_process_info.NextEntryOffset as isize;

            // base_address の該当オフセット値から SYSTEM_PROCESS_INFORMATION 構造体の情報をプロセス1つ分取得
            if fill_structure_from_memory(&mut system_process_info, next_address as *const c_void, GetCurrentProcess()) == 0 {
                let err = GetLastError();
                panic!("fill_structure_from_memory failed: {}", err);
            }
            
            // プロセス名を取得
            let mut image_name_vec: Vec<u16> = vec![0; system_process_info.ImageName.Length as usize];
            ReadProcessMemory(
                GetCurrentProcess(), system_process_info.ImageName.Buffer as *const c_void, image_name_vec.as_mut_ptr() as *mut c_void, 
                system_process_info.ImageName.Length as usize, std::ptr::null_mut()
            );
            // \0 を除去
            let proc_name = String::from_utf16_lossy(&image_name_vec).trim_matches(char::from(0)).to_string();

            // プロセスIDを取得
            let proc_id = system_process_info.UniqueProcessId as u32;
            
            // プロセス名とプロセスIDを表示
            println!("pid {} - {:#x?}", proc_id, proc_name);

            // すべてのプロセス情報を取得したら終了
            if system_process_info.NextEntryOffset == 0 {
                break;
            }
        }   

        VirtualFree(base_address, 0x0, MEM_RELEASE);
    }
}
