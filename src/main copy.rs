use std::process;
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS
};
use std::mem;

fn main() {
    // 获取系统进程快照
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == 0 {
        eprintln!("Failed to create process snapshot");
        process::exit(1);
    }

    let mut process_entry: PROCESSENTRY32 = unsafe { mem::zeroed() };
    process_entry.dwSize = mem::size_of::<PROCESSENTRY32>() as u32;

    // 获取第一个进程
    if unsafe { Process32First(snapshot, &mut process_entry) } == 0 {
        eprintln!("Failed to get first process");
        unsafe { windows_sys::Win32::Foundation::CloseHandle(snapshot) };
        process::exit(1);
    }

    println!("{:<8} {:<30} {:<8} {:<8}", 
             "PID", "Process Name", "PPID", "Threads");

    // 遍历所有进程
    loop {
        let pid = process_entry.th32ProcessID;
        let ppid = process_entry.th32ParentProcessID;
        
        let exe_file_bytes: Vec<u8> = process_entry.szExeFile.iter()
            .take_while(|&&c| c != 0)
            .map(|&c| c as u8)
            .collect();
        let name = String::from_utf8_lossy(&exe_file_bytes);
        
        println!("{:<8} {:<30} {:<8} {:<8}", 
                 pid, name, ppid, process_entry.cntThreads);

        // 获取下一个进程
        if unsafe { Process32Next(snapshot, &mut process_entry) } == 0 {
            break;
        }
    }

    unsafe { windows_sys::Win32::Foundation::CloseHandle(snapshot) };
}