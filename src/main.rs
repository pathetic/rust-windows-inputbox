use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::{Read, Write};
use std::ptr::null_mut;
use std::sync::mpsc;
use std::thread;
use std::cell::RefCell;

use tokio::runtime::Runtime;
use winapi::shared::minwindef::{DWORD, LPARAM, LRESULT, UINT, WORD, WPARAM};
use winapi::shared::windef::{HMENU, HWND};
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::um::wingdi::{CreateSolidBrush, RGB};
use winapi::um::winuser::{
    BS_DEFPUSHBUTTON, COLOR_BTNFACE, CreateWindowExA, CREATESTRUCTA, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT,
    DefWindowProcA, DestroyWindow, DispatchMessageA, ES_AUTOHSCROLL, GetDlgItem, GetMessageA,
    GetWindowTextA, IDC_ARROW, IDI_APPLICATION, LoadCursorW, LoadIconW, PostQuitMessage, RegisterClassA,
    SendMessageA, ShowWindow, SS_LEFT, SW_SHOW, TranslateMessage, UpdateWindow,
    WM_COMMAND, WM_CREATE, WM_CTLCOLORSTATIC, WM_DESTROY, WNDCLASSA, WS_CAPTION, WS_CHILD,
    WS_EX_CLIENTEDGE, WS_EX_DLGMODALFRAME, WS_EX_TOPMOST, WS_POPUP, WS_SYSMENU, WS_VISIBLE,
};

use winapi::um::winuser::SetProcessDPIAware;

// Constants for edit controls
const EM_LIMITTEXT: UINT = 0x00C5;

// Thread local input buffer
thread_local! {
    static INPUT_TEXT: RefCell<[u8; 256]> = RefCell::new([0; 256]);
    static CURRENT_LAYOUT: RefCell<UiLayout> = RefCell::new(default_layout());
}

// Input box data structure
struct InputBoxData {
    title: String,
    message: String,
}

// UI element positions structure
#[derive(Copy, Clone)]
struct UiLayout {
    title: [u8; 128],  // Fixed-size buffer for title
    description: [u8; 256], // Fixed-size buffer for description
    window_width: i32,
    window_height: i32,
    bg_x: i32,
    bg_y: i32,
    bg_width: i32,
    bg_height: i32,
    label_x: i32,
    label_y: i32,
    label_width: i32,
    label_height: i32,
    edit_x: i32,
    edit_y: i32,
    edit_width: i32,
    edit_height: i32,
    cancel_x: i32,
    cancel_y: i32,
    cancel_width: i32,
    cancel_height: i32,
    ok_x: i32,
    ok_y: i32,
    ok_width: i32,
    ok_height: i32,
}

impl UiLayout {
    // Helper methods to get title and description as strings
    fn get_title(&self) -> String {
        let end = self.title.iter().position(|&c| c == 0).unwrap_or(self.title.len());
        String::from_utf8_lossy(&self.title[0..end]).to_string()
    }
    
    fn get_description(&self) -> String {
        let end = self.description.iter().position(|&c| c == 0).unwrap_or(self.description.len());
        String::from_utf8_lossy(&self.description[0..end]).to_string()
    }
    
    fn set_title(&mut self, title: &str) {
        self.title = [0; 128];
        for (i, &byte) in title.as_bytes().iter().enumerate() {
            if i >= self.title.len() - 1 { break; }
            self.title[i] = byte;
        }
    }
    
    fn set_description(&mut self, desc: &str) {
        self.description = [0; 256];
        for (i, &byte) in desc.as_bytes().iter().enumerate() {
            if i >= self.description.len() - 1 { break; }
            self.description[i] = byte;
        }
    }
}

// Default layout
fn default_layout() -> UiLayout {
    let mut layout = UiLayout {
        title: [0; 128],
        description: [0; 256],
        window_width: 400,
        window_height: 250,
        bg_x: 0,
        bg_y: 0,
        bg_width: 0,
        bg_height: 0,
        label_x: 10,
        label_y: 10,
        label_width: 360,
        label_height: 120,
        edit_x: 10,
        edit_y: 140,
        edit_width: 360,
        edit_height: 25,
        cancel_x: 220,
        cancel_y: 170,
        cancel_width: 70,
        cancel_height: 25,
        ok_x: 305,
        ok_y: 170,
        ok_width: 70,
        ok_height: 25,
    };
    
    layout.set_title("Test InputBox");
    layout.set_description("How are you doing today? Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.");
    
    layout
}

// Read layout from file
fn read_layout_from_file(filename: &str) -> Option<UiLayout> {
    let mut file = match File::open(filename) {
        Ok(file) => file,
        Err(_) => return None,
    };

    let mut contents = String::new();
    if file.read_to_string(&mut contents).is_err() {
        return None;
    }

    let mut layout = default_layout();
    for line in contents.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue; // Skip empty lines and comments
        }
        
        let parts: Vec<&str> = line.splitn(2, '=').collect();
        if parts.len() != 2 {
            continue;
        }
        
        let key = parts[0].trim();
        let value = parts[1].trim();
        
        match key {
            "title" => layout.set_title(value),
            "description" => layout.set_description(value),
            _ => {
                // Try to parse as integer for other fields
                if let Ok(int_value) = value.parse::<i32>() {
                    match key {
                        "window_width" => layout.window_width = int_value,
                        "window_height" => layout.window_height = int_value,
                        "bg_x" => layout.bg_x = int_value,
                        "bg_y" => layout.bg_y = int_value,
                        "bg_width" => layout.bg_width = int_value,
                        "bg_height" => layout.bg_height = int_value,
                        "label_x" => layout.label_x = int_value,
                        "label_y" => layout.label_y = int_value,
                        "label_width" => layout.label_width = int_value,
                        "label_height" => layout.label_height = int_value,
                        "edit_x" => layout.edit_x = int_value,
                        "edit_y" => layout.edit_y = int_value,
                        "edit_width" => layout.edit_width = int_value,
                        "edit_height" => layout.edit_height = int_value,
                        "cancel_x" => layout.cancel_x = int_value,
                        "cancel_y" => layout.cancel_y = int_value,
                        "cancel_width" => layout.cancel_width = int_value,
                        "cancel_height" => layout.cancel_height = int_value,
                        "ok_x" => layout.ok_x = int_value,
                        "ok_y" => layout.ok_y = int_value,
                        "ok_width" => layout.ok_width = int_value,
                        "ok_height" => layout.ok_height = int_value,
                        _ => {},
                    }
                }
            }
        }
    }
    
    Some(layout)
}

// Create and show the input box with a layout
fn show_input_box(title: String, description: String) -> String {
    // Try to load layout from file, or use default if not available
    let mut layout = read_layout_from_file("layout.txt").unwrap_or_else(default_layout);
    
    // If title and description are provided (not empty), override the ones from the layout file
    if !title.is_empty() {
        layout.set_title(&title);
    }
    
    if !description.is_empty() {
        layout.set_description(&description);
    }
    
    // Store layout in the thread local to access it in WM_CREATE
    CURRENT_LAYOUT.with(|current| {
        *current.borrow_mut() = layout;
    });
    
    unsafe {
        let class_name = CString::new("MyInputBox").unwrap();
        let h_instance = GetModuleHandleA(null_mut());

        let wnd_class = WNDCLASSA {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wnd_proc),
            hInstance: h_instance,
            lpszClassName: class_name.as_ptr(),
            hIcon: LoadIconW(0 as _, IDI_APPLICATION),
            hCursor: LoadCursorW(0 as _, IDC_ARROW),
            hbrBackground: COLOR_BTNFACE as _, // Use dialog color for background
            lpszMenuName: null_mut(),
            cbClsExtra: 0,
            cbWndExtra: 0,
        };

        RegisterClassA(&wnd_class);

        let title_str = layout.get_title();
        let desc_str = layout.get_description();
        let desc_cstring = CString::new(desc_str).unwrap();
        
        let hwnd = CreateWindowExA(
            WS_EX_TOPMOST | WS_EX_DLGMODALFRAME, // Use dialog modal frame for cleaner appearance
            class_name.as_ptr(),
            CString::new(title_str).unwrap().as_ptr(),
            WS_CAPTION | WS_SYSMENU | WS_POPUP,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            layout.window_width,
            layout.window_height,
            null_mut(),
            null_mut(),
            h_instance,
            desc_cstring.as_ptr() as _,
        );

        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);

        let mut msg = std::mem::zeroed();
        while GetMessageA(&mut msg, null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageA(&msg);
        }

        // Get the input text from the thread local
        let mut result = String::new();
        INPUT_TEXT.with(|text| {
            let buffer = text.borrow();
            result = std::str::from_utf8(&buffer[..])
                .unwrap_or("")
                .trim_end_matches('\0')
                .to_string();
        });
        
        result
    }
}

// Window procedure for input box
unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            unsafe {
                let create_struct = lparam as *const CREATESTRUCTA;
                let desc_ptr = (*create_struct).lpCreateParams as *const i8;
                let label_text = if desc_ptr.is_null() {
                    CStr::from_bytes_with_nul(b"Enter value:\0").unwrap()
                } else {
                    CStr::from_ptr(desc_ptr)
                };

                // Get layout from thread local
                let layout = CURRENT_LAYOUT.with(|current| *current.borrow());

                // Don't create the background panel if height is set to 0
                if layout.bg_height > 0 {
                    // Label/description background - same color as window background
                    let h_static_bg = CreateWindowExA(
                        0,
                        CString::new("STATIC").unwrap().as_ptr(),
                        null_mut(),
                        WS_CHILD | WS_VISIBLE,  // Remove SS_SUNKEN for flat appearance
                        layout.bg_x,
                        layout.bg_y,
                        layout.bg_width,
                        layout.bg_height,
                        hwnd,
                        null_mut(),
                        null_mut(),
                        null_mut(),
                    );
                    
                    // Set background color to match system dialog
                    let h_brush = CreateSolidBrush(RGB(240, 240, 240));
                    SendMessageA(h_static_bg, WM_CTLCOLORSTATIC as u32, h_brush as WPARAM, h_static_bg as LPARAM);
                }

                // Label/description with multi-line support
                CreateWindowExA(
                    0,
                    CString::new("STATIC").unwrap().as_ptr(),
                    label_text.as_ptr(),
                    WS_CHILD | WS_VISIBLE | SS_LEFT, // Add SS_LEFT for left alignment
                    layout.label_x,
                    layout.label_y,
                    layout.label_width,
                    layout.label_height,
                    hwnd,
                    null_mut(),
                    null_mut(),
                    null_mut(),
                );

                // Input box
                let h_edit = CreateWindowExA(
                    WS_EX_CLIENTEDGE,
                    CString::new("EDIT").unwrap().as_ptr(),
                    null_mut(),
                    WS_CHILD | WS_VISIBLE | ES_AUTOHSCROLL,
                    layout.edit_x,
                    layout.edit_y,
                    layout.edit_width,
                    layout.edit_height,
                    hwnd,
                    1 as HMENU,
                    null_mut(),
                    null_mut(),
                );

                SendMessageA(h_edit, EM_LIMITTEXT as u32, 255, 0);

                // OK Button - positioned at the top right
                CreateWindowExA(
                    0,
                    CString::new("BUTTON").unwrap().as_ptr(),
                    CString::new("OK").unwrap().as_ptr(),
                    WS_CHILD | WS_VISIBLE | BS_DEFPUSHBUTTON,
                    layout.ok_x,
                    layout.ok_y,
                    layout.ok_width,
                    layout.ok_height,
                    hwnd,
                    2 as HMENU,
                    null_mut(),
                    null_mut(),
                );
                
                // Cancel Button - positioned below OK
                CreateWindowExA(
                    0,
                    CString::new("BUTTON").unwrap().as_ptr(),
                    CString::new("Cancel").unwrap().as_ptr(),
                    WS_CHILD | WS_VISIBLE,
                    layout.cancel_x,
                    layout.cancel_y,
                    layout.cancel_width,
                    layout.cancel_height,
                    hwnd,
                    3 as HMENU,
                    null_mut(),
                    null_mut(),
                );
            }
        }
        WM_COMMAND => {
            unsafe {
                let wm_id = LOWORD(wparam as DWORD) as usize;
                match wm_id {
                    2 => {
                        // OK clicked
                        let h_edit = GetDlgItem(hwnd, 1);
                        let mut buffer = [0u8; 256];
                        GetWindowTextA(h_edit, buffer.as_mut_ptr() as *mut i8, 256);
                        
                        // Store in the thread local
                        INPUT_TEXT.with(|text| {
                            *text.borrow_mut() = buffer;
                        });
                        
                        DestroyWindow(hwnd);
                    }
                    3 => {
                        // Cancel clicked
                        // Clear input text
                        INPUT_TEXT.with(|text| {
                            *text.borrow_mut() = [0; 256];
                        });
                        DestroyWindow(hwnd);
                    }
                    _ => {}
                }
            }
        }
        WM_DESTROY => {
            unsafe {
                PostQuitMessage(0);
            }
        }
        _ => {}
    }
    unsafe {
        DefWindowProcA(hwnd, msg, wparam, lparam)
    }
}

// Helper: extract low word
fn LOWORD(l: DWORD) -> WORD {
    (l & 0xffff) as WORD
}

// Function to handle input commands
fn handle_input_command(data: InputBoxData) -> String {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let result = show_input_box(data.title, data.message);
        let _ = tx.send(result);
    });

    rx.recv().unwrap_or_default()
}

// Create a sample layout file if it doesn't exist
fn create_sample_layout_file() {
    if let Ok(_) = std::fs::metadata("layout.txt") {
        // File already exists, don't overwrite
        return;
    }

    let layout = default_layout();
    let contents = format!(
        "# Input Box Layout Configuration\n\n\
        # Window title and message\n\
        title={}\n\
        description={}\n\n\
        # Window dimensions\n\
        window_width={}\n\
        window_height={}\n\n\
        # Background panel\n\
        bg_x={}\n\
        bg_y={}\n\
        bg_width={}\n\
        bg_height={}\n\n\
        # Label/description area\n\
        label_x={}\n\
        label_y={}\n\
        label_width={}\n\
        label_height={}\n\n\
        # Edit/input field\n\
        edit_x={}\n\
        edit_y={}\n\
        edit_width={}\n\
        edit_height={}\n\n\
        # Buttons\n\
        cancel_x={}\n\
        cancel_y={}\n\
        cancel_width={}\n\
        cancel_height={}\n\
        ok_x={}\n\
        ok_y={}\n\
        ok_width={}\n\
        ok_height={}",
        layout.get_title(), layout.get_description(),
        layout.window_width, layout.window_height,
        layout.bg_x, layout.bg_y, layout.bg_width, layout.bg_height,
        layout.label_x, layout.label_y, layout.label_width, layout.label_height,
        layout.edit_x, layout.edit_y, layout.edit_width, layout.edit_height,
        layout.cancel_x, layout.cancel_y, layout.cancel_width, layout.cancel_height,
        layout.ok_x, layout.ok_y, layout.ok_width, layout.ok_height
    );

    if let Ok(mut file) = File::create("layout.txt") {
        let _ = file.write_all(contents.as_bytes());
    }
}

fn main() {
    unsafe {
        // FIX DPI ISSUES
        SetProcessDPIAware();
    }

    // Create a sample layout file if it doesn't exist
    create_sample_layout_file();
    
    // Create tokio runtime for async tasks
    let _rt = Runtime::new().unwrap();
    
    loop {
        println!("Press Enter to show input box (Ctrl+C to exit)");
        
        // Wait for Enter key
        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer).unwrap();
        
        // Read layout for title and description
        let layout = read_layout_from_file("layout.txt").unwrap_or_else(default_layout);
        
        // Display input box with title and description from layout file
        let input = handle_input_command(InputBoxData {
            title: layout.get_title(),
            message: layout.get_description(),
        });
        
        if !input.is_empty() {
            println!("You entered: {}", input);
        } else {
            println!("No input provided or canceled");
        }
    }
}
