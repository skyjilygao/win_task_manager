use std::{error::Error, time::Duration};
use tui::{
    backend::Backend,
    layout::Constraint,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame, Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use sysinfo::{Pid, System};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS,
};
use std::mem;

enum SortBy {
    Cpu,
    Memory,
    Pid,
    Name,
}

pub struct ProcessInfo {
    pid: u32,
    name: String,
    cpu_usage: f32,
    memory: u64,
    threads: u32,
}

fn main() -> Result<(), Box<dyn Error>> {
    // 初始化终端
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = tui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 主循环
    let mut sort_by = SortBy::Cpu;
    let mut system = System::new_all();
    let res = run_app(&mut terminal, &mut system, &mut sort_by);

    // 清理终端
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    system: &mut System,
    sort_by: &mut SortBy,
) -> Result<(), Box<dyn Error>> {
    loop {
        terminal.draw(|f| ui(f, system, sort_by))?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('c') => *sort_by = SortBy::Cpu,
                    KeyCode::Char('m') => *sort_by = SortBy::Memory,
                    KeyCode::Char('p') => *sort_by = SortBy::Pid,
                    KeyCode::Char('n') => *sort_by = SortBy::Name,
                    _ => {}
                }
            }
        }
        system.refresh_all();
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, system: &System, sort_by: &SortBy) {
    let size = f.size();
    
    // 获取进程列表
    let mut processes = get_processes(system);
    
    // 排序
    match sort_by {
        SortBy::Cpu => processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap()),
        SortBy::Memory => processes.sort_by(|a, b| b.memory.cmp(&a.memory)),
        SortBy::Pid => processes.sort_by(|a, b| a.pid.cmp(&b.pid)),
        SortBy::Name => processes.sort_by(|a, b| a.name.cmp(&b.name)),
    }

    // 创建表格
    let rows = processes.iter().map(|p| {
        Row::new(vec![
            Cell::from(p.pid.to_string()),
            Cell::from(p.name.clone()),
            Cell::from(format!("{:.1}%", p.cpu_usage)),
            Cell::from(format!("{:.1} MB", p.memory as f64 / 1024.0 / 1024.0)),
            Cell::from(p.threads.to_string()),
        ])
    });

    let table = Table::new(rows)
        .header(
            Row::new(vec!["PID", "Name", "CPU%", "Memory", "Threads"])
                .style(Style::default().add_modifier(Modifier::BOLD))
                .bottom_margin(1),
        )
        .block(Block::default().borders(Borders::ALL).title("Process Manager"))
        .widths(&[
            Constraint::Length(8),
            Constraint::Percentage(40),
            Constraint::Length(8),
            Constraint::Length(12),
            Constraint::Length(8),
        ])
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_widget(table, size);
}

fn get_processes(system: &System) -> Vec<ProcessInfo> {
    let mut processes = Vec::new();
    
    // 获取进程快照
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == 0 {
        return processes;
    }

    let mut process_entry: PROCESSENTRY32 = unsafe { mem::zeroed() };
    process_entry.dwSize = mem::size_of::<PROCESSENTRY32>() as u32;

    if unsafe { Process32First(snapshot, &mut process_entry) } == 0 {
        unsafe { windows_sys::Win32::Foundation::CloseHandle(snapshot) };
        return processes;
    }

    loop {
        let pid = process_entry.th32ProcessID;
        
        // 获取进程名
        let name_bytes: Vec<u8> = process_entry.szExeFile.iter()
            .take_while(|&&c| c != 0)
            .map(|&c| c as u8)
            .collect();
        let name = String::from_utf8_lossy(&name_bytes).into_owned();
        
        // 使用sysinfo获取CPU和内存信息
        if let Some(process) = system.process(Pid::from_u32(pid)) {
            processes.push(ProcessInfo {
                pid,
                name,
                cpu_usage: process.cpu_usage(),
                memory: process.memory(),
                threads: process_entry.cntThreads,
            });
        }

        if unsafe { Process32Next(snapshot, &mut process_entry) } == 0 {
            break;
        }
    }

    unsafe { windows_sys::Win32::Foundation::CloseHandle(snapshot) };
    processes
}