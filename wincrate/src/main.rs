use windows::{
    core::*, Data::Xml::Dom::*, Win32::Foundation::*, Win32::System::Threading::*,
    Win32::UI::WindowsAndMessaging::*,
    Win32::System::WindowsProgramming::{ NtQuerySystemInformation, SYSTEM_INFORMATION_CLASS }
};

fn get_object_list() -> Result<Vec<String>> {
    let mut buffer = vec![0u8; 0x1000];
    let mut return_length = 0u32;
    let system_information_class = SYSTEM_INFORMATION_CLASS::default(); // SystemHandleInformation
    let status = unsafe {
        NtQuerySystemInformation(
            system_information_class, // SystemHandleInformation
            buffer.as_mut_ptr() as *mut _,
            buffer.len() as u32,
            &mut return_length,
        )
    };
    if status.is_err() {
        //return Err(Error::from(status));
        return Err(Error::new(HRESULT::from(status), HSTRING::new()));
    }

    let mut handles = Vec::new();
    let handle_info = unsafe { &*(buffer.as_ptr() as String) };
    for i in 0..handle_info.HandleCount {
        let handle = unsafe { handle_info.Handles[i as usize] };
        if handle.ObjectTypeNumber == 0x0D { // Process
            handles.push(format!("0x{:X}", handle.Handle));
        }
    }

    Ok(handles)
}

fn main() -> Result<()> {
    let doc = XmlDocument::new()?;
    doc.LoadXml(h!("<html>hello world</html>"))?;

    let root = doc.DocumentElement()?;
    assert!(root.NodeName()? == "html");
    assert!(root.InnerText()? == "hello world");

    let handles = get_object_list()?;
    println!("Process handles: {:?}", handles);

    unsafe {
        let event = CreateEventW(None, true, false, None)?;
        SetEvent(event).ok()?;
        WaitForSingleObject(event, 0).ok()?;
        CloseHandle(event).ok()?;

        MessageBoxA(None, s!("Ansi"), s!("Caption"), MB_OK);
        MessageBoxW(None, w!("Wide"), w!("Caption"), MB_OK);
    }

    Ok(())
}
