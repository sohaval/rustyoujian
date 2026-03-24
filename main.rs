use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::System::Registry::*,
    Win32::UI::WindowsAndMessaging::*,
    Win32::UI::Controls::*,
    Win32::System::LibraryLoader::GetModuleHandleA,
    Win32::Security::*,
    Win32::System::Threading::*,
    Win32::Graphics::Gdi::*,
};

#[link(name = "user32")]
extern "system" {
    fn EnableWindow(hwnd: HWND, benable: bool) -> bool;
}

#[link(name = "kernel32")]
extern "system" {
    fn SetConsoleOutputCP(wCodePageID: u32) -> bool;
}

static mut DIALOG_RESULT: bool = false;
static mut DIALOG_CLOSED: bool = false;
static mut EDIT_KEY: HWND = HWND(0);
static mut EDIT_DISPLAY: HWND = HWND(0);
static mut EDIT_COMMAND: HWND = HWND(0);
static mut EDIT_ICON: HWND = HWND(0);
static mut FORM_PTR: *mut MenuItemForm = std::ptr::null_mut();

extern "system" fn dialog_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_COMMAND => {
                let id = loword(wparam.0 as u32) as i32;
                if id == IDOK {
                    let get_text = |hwnd: HWND| -> String {
                        let len = GetWindowTextLengthW(hwnd);
                        let mut buf = vec![0u16; (len + 1) as usize];
                        let _ = GetWindowTextW(hwnd, &mut buf);
                        String::from_utf16_lossy(&buf[..len as usize])
                    };
                    if !FORM_PTR.is_null() {
                        (*FORM_PTR).key_name = get_text(EDIT_KEY);
                        (*FORM_PTR).display_name = get_text(EDIT_DISPLAY);
                        (*FORM_PTR).command = get_text(EDIT_COMMAND);
                        (*FORM_PTR).icon = get_text(EDIT_ICON);
                    }
                    DIALOG_RESULT = true;
                    DIALOG_CLOSED = true;
                    let _ = DestroyWindow(hwnd);
                    LRESULT(0)
                } else if id == IDCANCEL {
                    DIALOG_RESULT = false;
                    DIALOG_CLOSED = true;
                    let _ = DestroyWindow(hwnd);
                    LRESULT(0)
                } else {
                    DefWindowProcW(hwnd, msg, wparam, lparam)
                }
            }
            WM_CLOSE => {
                DIALOG_RESULT = false;
                DIALOG_CLOSED = true;
                let _ = DestroyWindow(hwnd);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

fn loword(l: u32) -> u16 {
    (l & 0xFFFF) as u16
}

// 窗口控件 ID
const IDC_COMBO_CONTEXT: i32 = 1001;
const IDC_BUTTON_REFRESH: i32 = 1002;
const IDC_LIST_VIEW: i32 = 1003;
const IDC_BUTTON_ADD: i32 = 1004;
const IDC_BUTTON_EDIT: i32 = 1005;
const IDC_BUTTON_DELETE: i32 = 1006;
const IDC_BUTTON_ENABLE_DISABLE: i32 = 1007;

// 对话框控件 ID
const IDC_EDIT_KEY_NAME: i32 = 2001;
const IDC_EDIT_DISPLAY_NAME: i32 = 2002;
const IDC_EDIT_COMMAND: i32 = 2003;
const IDC_EDIT_ICON: i32 = 2004;
const IDOK: i32 = 1;
const IDCANCEL: i32 = 2;

// 上下文类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContextType {
    File,
    Folder,
    Drive,
    Desktop,
}

impl ContextType {
    fn reg_path(&self) -> &'static str {
        match self {
            ContextType::File => r"*\shell",
            ContextType::Folder => r"Folder\shell",
            ContextType::Drive => r"Drive\shell",
            ContextType::Desktop => r"DesktopBackground\shell",
        }
    }

    fn display_name(&self) -> &'static str {
        match self {
            ContextType::File => "文件 (*)",
            ContextType::Folder => "文件夹",
            ContextType::Drive => "驱动器",
            ContextType::Desktop => "桌面背景",
        }
    }
}

// 菜单项数据结构
#[derive(Debug, Clone)]
struct MenuItem {
    name: String,
    display_name: String,
    command: String,
    icon: Option<String>,
    disabled: bool,
}

// 全局应用状态
struct AppState {
    hwnd: HWND,
    h_list_view: HWND,
    h_combo: HWND,
    current_context: ContextType,
    menu_items: Vec<MenuItem>,
    is_admin: bool,
}

impl AppState {
    fn new(hwnd: HWND) -> Self {
        let is_admin = is_running_as_admin();
        Self {
            hwnd,
            h_list_view: HWND::default(),
            h_combo: HWND::default(),
            current_context: ContextType::File,
            menu_items: Vec::new(),
            is_admin,
        }
    }

    fn refresh_menu_items(&mut self) {
        if !self.is_admin {
            unsafe {
                MessageBoxW(
                    self.hwnd,
                    w!("程序未以管理员身份运行，无法读取/修改注册表。请以管理员权限重新启动。"),
                    w!("错误"),
                    MB_OK | MB_ICONERROR,
                );
            }
            return;
        }

        let path = self.current_context.reg_path();
        match read_menu_items_from_registry(path) {
            Ok(items) => {
                self.menu_items = items;
                self.update_list_view();
            }
            Err(_) => {
                self.menu_items = Vec::new();
                self.update_list_view();
            }
        }
    }

    fn update_list_view(&self) {
        unsafe {
            SendMessageW(self.h_list_view, LVM_DELETEALLITEMS, WPARAM(0), LPARAM(0));
        }
        for (i, item) in self.menu_items.iter().enumerate() {
            let index = i as i32;
            let status = if item.disabled { "已禁用" } else { "已启用" };

            let display_name: Vec<u16> = item.display_name.encode_utf16().chain(Some(0)).collect();
            let key_name: Vec<u16> = item.name.encode_utf16().chain(Some(0)).collect();
            let command: Vec<u16> = item.command.encode_utf16().chain(Some(0)).collect();
            let status_vec: Vec<u16> = status.encode_utf16().chain(Some(0)).collect();

            unsafe {
                let mut lvi = LVITEMW {
                    mask: LVIF_TEXT,
                    iItem: index,
                    iSubItem: 0,
                    pszText: PWSTR(display_name.as_ptr() as *mut _),
                    ..Default::default()
                };
                SendMessageW(self.h_list_view, LVM_INSERTITEMW, WPARAM(0), LPARAM(&mut lvi as *mut _ as isize));

                lvi.iSubItem = 1;
                lvi.pszText = PWSTR(key_name.as_ptr() as *mut _);
                SendMessageW(self.h_list_view, LVM_SETITEMW, WPARAM(index as usize), LPARAM(&mut lvi as *mut _ as isize));

                lvi.iSubItem = 2;
                lvi.pszText = PWSTR(command.as_ptr() as *mut _);
                SendMessageW(self.h_list_view, LVM_SETITEMW, WPARAM(index as usize), LPARAM(&mut lvi as *mut _ as isize));

                lvi.iSubItem = 3;
                lvi.pszText = PWSTR(status_vec.as_ptr() as *mut _);
                SendMessageW(self.h_list_view, LVM_SETITEMW, WPARAM(index as usize), LPARAM(&mut lvi as *mut _ as isize));
            }
        }
    }

    fn get_selected_item_index(&self) -> Option<usize> {
        unsafe {
            let index = SendMessageW(self.h_list_view, LVM_GETNEXTITEM, WPARAM(usize::MAX), LPARAM(LVNI_SELECTED as isize));
            if index.0 == -1 { None } else { Some(index.0 as usize) }
        }
    }

    fn add_menu_item(&mut self) {
        let mut form = MenuItemForm::default();
        if self.show_dialog(&mut form, true) {
            if form.key_name.is_empty() || form.display_name.is_empty() || form.command.is_empty() {
                unsafe {
                    MessageBoxW(self.hwnd, w!("键名、显示名称和命令不能为空"), w!("错误"), MB_OK | MB_ICONERROR);
                }
                return;
            }
            let path = self.current_context.reg_path();
            match create_menu_item_in_registry(path, &form) {
                Ok(()) => {
                    unsafe {
                        MessageBoxW(self.hwnd, w!("菜单项添加成功"), w!("成功"), MB_OK);
                    }
                    self.refresh_menu_items();
                }
                Err(e) => {
                    let msg = format!("添加失败：{}", e);
                    unsafe {
                        MessageBoxW(self.hwnd, &HSTRING::from(msg), w!("错误"), MB_OK | MB_ICONERROR);
                    }
                }
            }
        }
    }

    fn edit_menu_item(&mut self, index: usize) {
        let item = &self.menu_items[index];
        let mut form = MenuItemForm {
            key_name: item.name.clone(),
            display_name: item.display_name.clone(),
            command: item.command.clone(),
            icon: item.icon.clone().unwrap_or_default(),
        };
        if self.show_dialog(&mut form, false) {
            if form.key_name.is_empty() || form.display_name.is_empty() || form.command.is_empty() {
                unsafe {
                    MessageBoxW(self.hwnd, w!("键名、显示名称和命令不能为空"), w!("错误"), MB_OK | MB_ICONERROR);
                }
                return;
            }
            let path = self.current_context.reg_path();
            match update_menu_item_in_registry(path, &item.name, &form) {
                Ok(()) => {
                    unsafe {
                        MessageBoxW(self.hwnd, w!("菜单项更新成功"), w!("成功"), MB_OK);
                    }
                    self.refresh_menu_items();
                }
                Err(e) => {
                    let msg = format!("更新失败：{}", e);
                    unsafe {
                        MessageBoxW(self.hwnd, &HSTRING::from(msg), w!("错误"), MB_OK | MB_ICONERROR);
                    }
                }
            }
        }
    }

    fn delete_menu_item(&mut self, index: usize) {
        let item = &self.menu_items[index];
        let path = self.current_context.reg_path();
        match delete_menu_item_from_registry(path, &item.name) {
            Ok(()) => {
                unsafe {
                    MessageBoxW(self.hwnd, w!("菜单项已删除"), w!("成功"), MB_OK);
                }
                self.refresh_menu_items();
            }
            Err(e) => {
                let msg = format!("删除失败：{}", e);
                unsafe {
                    MessageBoxW(self.hwnd, &HSTRING::from(msg), w!("错误"), MB_OK | MB_ICONERROR);
                }
            }
        }
    }

    fn toggle_disabled(&mut self, index: usize) {
        let item = &self.menu_items[index];
        let new_disabled = !item.disabled;
        let path = self.current_context.reg_path();
        match set_menu_item_disabled(path, &item.name, new_disabled) {
            Ok(()) => {
                let msg = if new_disabled { "已禁用" } else { "已启用" };
                let msg = format!("菜单项{}", msg);
                unsafe {
                    MessageBoxW(self.hwnd, &HSTRING::from(msg), w!("成功"), MB_OK);
                }
                self.refresh_menu_items();
            }
            Err(e) => {
                let msg = format!("操作失败：{}", e);
                unsafe {
                    MessageBoxW(self.hwnd, &HSTRING::from(msg), w!("错误"), MB_OK | MB_ICONERROR);
                }
            }
        }
    }

    fn show_dialog(&self, form: &mut MenuItemForm, is_add: bool) -> bool {
        let title = if is_add { "添加右键菜单项" } else { "编辑右键菜单项" };
        let h_inst = unsafe { GetModuleHandleA(None).unwrap_or_default() };
        let dialog_class_name = w!("RightClickManagerDialog");

        unsafe {
            DIALOG_CLOSED = false;
            DIALOG_RESULT = false;
            FORM_PTR = form as *mut _;
        }

        let hwnd_dialog = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                dialog_class_name,
                &HSTRING::from(title),
                WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                600,
                320,
                self.hwnd,
                HMENU(0),
                h_inst,
                None,
            )
        };
        if hwnd_dialog.0 == 0 {
            return false;
        }

        let create_edit = |x, y, w, h, id, multiline| unsafe {
            let style = if multiline {
                WS_CHILD | WS_VISIBLE | WS_BORDER | WINDOW_STYLE(ES_LEFT as u32 | ES_MULTILINE as u32 | ES_AUTOHSCROLL as u32)
            } else {
                WS_CHILD | WS_VISIBLE | WS_BORDER | WINDOW_STYLE(ES_LEFT as u32 | ES_AUTOHSCROLL as u32)
            };
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("EDIT"),
                w!(""),
                style,
                x,
                y,
                w,
                h,
                hwnd_dialog,
                HMENU(id as isize),
                h_inst,
                None,
            )
        };
        let create_label = |x, y, w, h, text| unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("STATIC"),
                &HSTRING::from(text),
                WS_CHILD | WS_VISIBLE,
                x,
                y,
                w,
                h,
                hwnd_dialog,
                HMENU(0),
                h_inst,
                None,
            )
        };

        let edit_width = 560;
        let label_x = 10;
        let edit_x = 10;
        let _row_height = 25;
        let label_width = 80;
        let edit_height = 22;

        create_label(label_x, 10, label_width, 20, "键名:");
        let edit_key = create_edit(edit_x, 30, edit_width, edit_height, IDC_EDIT_KEY_NAME, false);

        create_label(label_x, 55, label_width, 20, "显示名称:");
        let edit_display = create_edit(edit_x, 75, edit_width, edit_height, IDC_EDIT_DISPLAY_NAME, false);

        create_label(label_x, 100, label_width, 20, "命令:");
        let edit_command = create_edit(edit_x, 120, edit_width, 45, IDC_EDIT_COMMAND, true);

        create_label(label_x, 170, label_width, 20, "图标路径:");
        let edit_icon = create_edit(edit_x, 190, edit_width, edit_height, IDC_EDIT_ICON, false);

        unsafe {
            EDIT_KEY = edit_key;
            EDIT_DISPLAY = edit_display;
            EDIT_COMMAND = edit_command;
            EDIT_ICON = edit_icon;
            let _ = SetWindowTextW(edit_key, &HSTRING::from(&form.key_name));
            let _ = SetWindowTextW(edit_display, &HSTRING::from(&form.display_name));
            let _ = SetWindowTextW(edit_command, &HSTRING::from(&form.command));
            let _ = SetWindowTextW(edit_icon, &HSTRING::from(&form.icon));
        }

        let btn_y = 225;
        let btn_width = 80;
        let btn_height = 28;
        let btn_ok_x = 380;
        let btn_cancel_x = 480;

        let _btn_ok = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("BUTTON"),
                w!("确定"),
                WS_CHILD | WS_VISIBLE | WINDOW_STYLE(BS_PUSHBUTTON as u32),
                btn_ok_x,
                btn_y,
                btn_width,
                btn_height,
                hwnd_dialog,
                HMENU(IDOK as isize),
                h_inst,
                None,
            )
        };
        let _btn_cancel = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("BUTTON"),
                w!("取消"),
                WS_CHILD | WS_VISIBLE | WINDOW_STYLE(BS_PUSHBUTTON as u32),
                btn_cancel_x,
                btn_y,
                btn_width,
                btn_height,
                hwnd_dialog,
                HMENU(IDCANCEL as isize),
                h_inst,
                None,
            )
        };

        let mut msg = MSG::default();
        unsafe {
            EnableWindow(self.hwnd, false);
            while !DIALOG_CLOSED {
                if GetMessageW(&mut msg, HWND::default(), 0, 0).as_bool() {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
            EnableWindow(self.hwnd, true);
            FORM_PTR = std::ptr::null_mut();
        }
        unsafe { DIALOG_RESULT }
    }
}

#[derive(Debug, Clone, Default)]
struct MenuItemForm {
    key_name: String,
    display_name: String,
    command: String,
    icon: String,
}

fn open_key(root: HKEY, subkey: &str, write: bool) -> windows::core::Result<HKEY> {
    let access = if write { KEY_READ | KEY_WRITE | KEY_CREATE_SUB_KEY } else { KEY_READ };
    let mut key = HKEY::default();
    unsafe {
        let result = RegOpenKeyExW(root, &HSTRING::from(subkey), 0, access, &mut key);
        if result.0 != 0 {
            return Err(Error::from_win32());
        }
    }
    Ok(key)
}

fn read_menu_items_from_registry(context_path: &str) -> windows::core::Result<Vec<MenuItem>> {
    let root = HKEY_CLASSES_ROOT;
    let context_key = open_key(root, context_path, false)?;
    let mut items = Vec::new();
    let mut index = 0;
    let mut key_name_buf = [0u16; 256];
    loop {
        let mut len = key_name_buf.len() as u32;
        let status = unsafe {
            RegEnumKeyExW(
                context_key,
                index,
                PWSTR(key_name_buf.as_mut_ptr()),
                &mut len,
                None,
                PWSTR::null(),
                None,
                None,
            )
        };
        if status.0 != 0 { break; }
        let key_name = String::from_utf16_lossy(&key_name_buf[..len as usize]);
        let subkey_path = format!("{}\\{}", context_path, key_name);
        
        let subkey = match open_key(root, &subkey_path, false) {
            Ok(k) => k,
            Err(_) => {
                index += 1;
                continue;
            }
        };

        let display_name;
        {
            let mut value_data = [0u16; 1024];
            let mut data_len = (value_data.len() * 2) as u32;
            let status = unsafe {
                RegQueryValueExW(subkey, PCWSTR::null(), None, None, Some(value_data.as_mut_ptr() as *mut u8), Some(&mut data_len))
            };
            if status.0 == 0 {
                display_name = String::from_utf16_lossy(&value_data[..(data_len as usize / 2)]);
            } else {
                display_name = key_name.clone();
            }
        }

        let command_path = format!("{}\\{}", subkey_path, "command");
        let command = match open_key(root, &command_path, false) {
            Ok(command_key) => {
                let mut value_data = [0u16; 1024];
                let mut data_len = (value_data.len() * 2) as u32;
                let status = unsafe {
                    RegQueryValueExW(command_key, PCWSTR::null(), None, None, Some(value_data.as_mut_ptr() as *mut u8), Some(&mut data_len))
                };
                if status.0 == 0 {
                    String::from_utf16_lossy(&value_data[..(data_len as usize / 2)])
                } else {
                    String::new()
                }
            }
            Err(_) => String::new(),
        };

        let mut icon = None;
        {
            let mut value_data = [0u16; 1024];
            let mut data_len = (value_data.len() * 2) as u32;
            let status = unsafe {
                RegQueryValueExW(subkey, &HSTRING::from("Icon"), None, None, Some(value_data.as_mut_ptr() as *mut u8), Some(&mut data_len))
            };
            if status.0 == 0 {
                let icon_str = String::from_utf16_lossy(&value_data[..(data_len as usize / 2)]);
                if !icon_str.is_empty() { icon = Some(icon_str); }
            }
        }

        let mut disabled = false;
        let status = unsafe {
            RegQueryValueExW(subkey, &HSTRING::from("LegacyDisable"), None, None, None, None)
        };
        if status.0 == 0 { disabled = true; }

        items.push(MenuItem {
            name: key_name,
            display_name,
            command,
            icon,
            disabled,
        });
        index += 1;
    }
    Ok(items)
}

fn set_reg_sz_value(key: HKEY, name: PCWSTR, value: &str) -> windows::core::Result<()> {
    let wide: Vec<u16> = value.encode_utf16().chain(Some(0)).collect();
    let bytes = unsafe { std::slice::from_raw_parts(wide.as_ptr() as *const u8, wide.len() * 2) };
    unsafe {
        RegSetValueExW(key, name, 0, REG_SZ, Some(bytes)).ok()?;
    }
    Ok(())
}

fn create_menu_item_in_registry(context_path: &str, form: &MenuItemForm) -> windows::core::Result<()> {
    let root = HKEY_CLASSES_ROOT;
    let key_path = format!("{}\\{}", context_path, form.key_name);
    let key = open_key(root, &key_path, true)?;

    set_reg_sz_value(key, PCWSTR::null(), &form.display_name)?;

    if !form.icon.is_empty() {
        set_reg_sz_value(key, PCWSTR(HSTRING::from("Icon").as_ptr()), &form.icon)?;
    }

    let command_path = format!("{}\\command", key_path);
    let command_key = open_key(root, &command_path, true)?;
    set_reg_sz_value(command_key, PCWSTR::null(), &form.command)?;
    Ok(())
}

fn update_menu_item_in_registry(context_path: &str, old_key_name: &str, form: &MenuItemForm) -> windows::core::Result<()> {
    if old_key_name != form.key_name {
        delete_menu_item_from_registry(context_path, old_key_name)?;
        create_menu_item_in_registry(context_path, form)?;
    } else {
        let root = HKEY_CLASSES_ROOT;
        let key_path = format!("{}\\{}", context_path, form.key_name);
        let key = open_key(root, &key_path, true)?;

        set_reg_sz_value(key, PCWSTR::null(), &form.display_name)?;

        if !form.icon.is_empty() {
            set_reg_sz_value(key, PCWSTR(HSTRING::from("Icon").as_ptr()), &form.icon)?;
        } else {
            unsafe { let _ = RegDeleteValueW(key, &HSTRING::from("Icon")); }
        }

        let command_path = format!("{}\\command", key_path);
        let command_key = open_key(root, &command_path, true)?;
        set_reg_sz_value(command_key, PCWSTR::null(), &form.command)?;
    }
    Ok(())
}

fn delete_menu_item_from_registry(context_path: &str, key_name: &str) -> windows::core::Result<()> {
    let root = HKEY_CLASSES_ROOT;
    let full_path = format!("{}\\{}", context_path, key_name);
    let full_path_wide: Vec<u16> = full_path.encode_utf16().chain(Some(0)).collect();
    unsafe {
        let _ = RegDeleteTreeW(root, PCWSTR(full_path_wide.as_ptr()));
    }
    Ok(())
}

fn set_menu_item_disabled(context_path: &str, key_name: &str, disabled: bool) -> windows::core::Result<()> {
    let root = HKEY_CLASSES_ROOT;
    let key_path = format!("{}\\{}", context_path, key_name);
    let key = open_key(root, &key_path, true)?;
    if disabled {
        let value: Vec<u16> = vec![0];
        let bytes = unsafe { std::slice::from_raw_parts(value.as_ptr() as *const u8, value.len() * 2) };
        unsafe {
            let _ = RegSetValueExW(key, &HSTRING::from("LegacyDisable"), 0, REG_SZ, Some(bytes));
        }
    } else {
        unsafe { let _ = RegDeleteValueW(key, &HSTRING::from("LegacyDisable")); }
    }
    Ok(())
}

fn is_running_as_admin() -> bool {
    unsafe {
        let mut token = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }
        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut size = 0;
        let result = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut size
        );
        let _ = CloseHandle(token);
        if result.is_ok() && elevation.TokenIsElevated != 0 {
            return true;
        }
        false
    }
}

extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
        let state = if state_ptr != 0 { (state_ptr as *mut AppState).as_mut() } else { None };

        match msg {
            WM_CREATE => {
                let createstruct = lparam.0 as *const CREATESTRUCTW;
                let state = (*createstruct).lpCreateParams as *mut AppState;
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, state as isize);

                let icc = INITCOMMONCONTROLSEX {
                    dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
                    dwICC: ICC_LISTVIEW_CLASSES,
                };
                InitCommonControlsEx(&icc);

                let state = &mut *state;

                state.h_combo = CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    w!("COMBOBOX"),
                    w!(""),
                    WS_CHILD | WS_VISIBLE | WINDOW_STYLE(CBS_DROPDOWNLIST as u32 | CBS_HASSTRINGS as u32),
                    10,
                    10,
                    150,
                    200,
                    hwnd,
                    HMENU(IDC_COMBO_CONTEXT as isize),
                    GetModuleHandleA(None).unwrap_or_default(),
                    None,
                );

                for (i, ctx) in [ContextType::File, ContextType::Folder, ContextType::Drive, ContextType::Desktop].iter().enumerate() {
                    let display_name = HSTRING::from(ctx.display_name());
                    SendMessageW(state.h_combo, CB_ADDSTRING, WPARAM(0), LPARAM(display_name.as_ptr() as _));
                    SendMessageW(state.h_combo, CB_SETITEMDATA, WPARAM(i), LPARAM(*ctx as isize));
                }
                SendMessageW(state.h_combo, CB_SETCURSEL, WPARAM(0), LPARAM(0));

                state.h_list_view = CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    w!("SysListView32"),
                    w!(""),
                    WS_CHILD | WS_VISIBLE | WS_BORDER | WINDOW_STYLE(LVS_REPORT as u32) | WINDOW_STYLE(LVS_SINGLESEL as u32),
                    10,
                    40,
                    560,
                    300,
                    hwnd,
                    HMENU(IDC_LIST_VIEW as isize),
                    GetModuleHandleA(None).unwrap_or_default(),
                    None,
                );

                let columns: Vec<HSTRING> = vec![
                    HSTRING::from("显示名称"),
                    HSTRING::from("键名"),
                    HSTRING::from("命令"),
                    HSTRING::from("状态"),
                ];
                for (i, col) in columns.iter().enumerate() {
                    let mut lvc = LVCOLUMNW {
                        mask: LVCF_TEXT | LVCF_WIDTH,
                        iSubItem: i as i32,
                        cx: 140,
                        pszText: PWSTR(col.as_ptr() as *mut _),
                        ..Default::default()
                    };
                    SendMessageW(state.h_list_view, LVM_INSERTCOLUMNW, WPARAM(i), LPARAM(&mut lvc as *mut _ as _));
                }

                let _btn_refresh = CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    w!("BUTTON"),
                    w!("刷新"),
                    WS_CHILD | WS_VISIBLE | WINDOW_STYLE(BS_PUSHBUTTON as u32),
                    170,
                    10,
                    60,
                    24,
                    hwnd,
                    HMENU(IDC_BUTTON_REFRESH as isize),
                    GetModuleHandleA(None).unwrap_or_default(),
                    None,
                );

                let _btn_add = CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    w!("BUTTON"),
                    w!("添加"),
                    WS_CHILD | WS_VISIBLE | WINDOW_STYLE(BS_PUSHBUTTON as u32),
                    10,
                    350,
                    80,
                    24,
                    hwnd,
                    HMENU(IDC_BUTTON_ADD as isize),
                    GetModuleHandleA(None).unwrap_or_default(),
                    None,
                );

                let _btn_edit = CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    w!("BUTTON"),
                    w!("编辑"),
                    WS_CHILD | WS_VISIBLE | WINDOW_STYLE(BS_PUSHBUTTON as u32),
                    100,
                    350,
                    80,
                    24,
                    hwnd,
                    HMENU(IDC_BUTTON_EDIT as isize),
                    GetModuleHandleA(None).unwrap_or_default(),
                    None,
                );

                let _btn_delete = CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    w!("BUTTON"),
                    w!("删除"),
                    WS_CHILD | WS_VISIBLE | WINDOW_STYLE(BS_PUSHBUTTON as u32),
                    190,
                    350,
                    80,
                    24,
                    hwnd,
                    HMENU(IDC_BUTTON_DELETE as isize),
                    GetModuleHandleA(None).unwrap_or_default(),
                    None,
                );

                let _btn_enable_disable = CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    w!("BUTTON"),
                    w!("启用/禁用"),
                    WS_CHILD | WS_VISIBLE | WINDOW_STYLE(BS_PUSHBUTTON as u32),
                    280,
                    350,
                    100,
                    24,
                    hwnd,
                    HMENU(IDC_BUTTON_ENABLE_DISABLE as isize),
                    GetModuleHandleA(None).unwrap_or_default(),
                    None,
                );

                if !state.is_admin {
                    let _ = EnableWindow(GetDlgItem(hwnd, IDC_BUTTON_ADD as i32), false);
                    let _ = EnableWindow(GetDlgItem(hwnd, IDC_BUTTON_EDIT as i32), false);
                    let _ = EnableWindow(GetDlgItem(hwnd, IDC_BUTTON_DELETE as i32), false);
                    let _ = EnableWindow(GetDlgItem(hwnd, IDC_BUTTON_ENABLE_DISABLE as i32), false);
                }

                return LRESULT(0);
            }
            WM_COMMAND => {
                if let Some(state) = state {
                    let id = loword(wparam.0 as u32) as i32;
                    match id {
                        IDC_BUTTON_REFRESH => {
                            state.refresh_menu_items();
                        }
                        IDC_BUTTON_ADD => {
                            state.add_menu_item();
                        }
                        IDC_BUTTON_EDIT => {
                            if let Some(index) = state.get_selected_item_index() {
                                state.edit_menu_item(index);
                            }
                        }
                        IDC_BUTTON_DELETE => {
                            if let Some(index) = state.get_selected_item_index() {
                                state.delete_menu_item(index);
                            }
                        }
                        IDC_BUTTON_ENABLE_DISABLE => {
                            if let Some(index) = state.get_selected_item_index() {
                                state.toggle_disabled(index);
                            }
                        }
                        IDC_COMBO_CONTEXT => {
                            let sel = SendMessageW(state.h_combo, CB_GETCURSEL, WPARAM(0), LPARAM(0));
                            if sel.0 != CB_ERR as isize {
                                let ctx = SendMessageW(state.h_combo, CB_GETITEMDATA, WPARAM(sel.0 as usize), LPARAM(0));
                                match ctx.0 as i8 {
                                    0 => state.current_context = ContextType::File,
                                    1 => state.current_context = ContextType::Folder,
                                    2 => state.current_context = ContextType::Drive,
                                    3 => state.current_context = ContextType::Desktop,
                                    _ => {}
                                }
                                state.refresh_menu_items();
                            }
                        }
                        _ => {}
                    }
                }
            }
            WM_DESTROY => {
                PostQuitMessage(0);
            }
            _ => {}
        }
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}

fn main() -> windows::core::Result<()> {
    unsafe {
        SetConsoleOutputCP(65001);
        let h_instance = GetModuleHandleA(None)?;
        let class_name = w!("RightClickManager");

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wnd_proc),
            hInstance: h_instance.into(),
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hbrBackground: HBRUSH(COLOR_WINDOW.0 as isize + 1),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        if RegisterClassExW(&wc) == 0 {
            return Err(Error::from_win32());
        }

        let dialog_class_name = w!("RightClickManagerDialog");
        let wc_dialog = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(dialog_proc),
            hInstance: h_instance.into(),
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hbrBackground: HBRUSH(COLOR_WINDOW.0 as isize + 1),
            lpszClassName: PCWSTR(dialog_class_name.as_ptr()),
            ..Default::default()
        };

        if RegisterClassExW(&wc_dialog) == 0 {
            return Err(Error::from_win32());
        }

        let mut state = AppState::new(HWND::default());
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class_name,
            w!("右键菜单管理器"),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            600,
            420,
            None,
            None,
            h_instance,
            Some(&mut state as *mut _ as *const _),
        );

        if hwnd.0 == 0 {
            return Err(Error::from_win32());
        }

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    Ok(())
}
