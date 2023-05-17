use std::ffi::c_void;
use std::mem;
use retour::static_detour;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::consoleapi::AllocConsole;
use winapi::um::winnt::DLL_PROCESS_ATTACH;
use winapi::shared::minwindef::*;

#[repr(C)]
struct UnityString {
    pub _class: *const c_void,
    pub _monitor: *const c_void,
    pub length: u32,
    pub buffer: [u16; 1000]
}

static_detour! {
    /// void __stdcall PlayerStats::ServerTakeDamage(PlayerStats_o *this, float damage, const MethodInfo *method)
    static PSServerTakeDamage: unsafe extern "system" fn(*mut c_void, f32, *const c_void) -> c_void;
    /// void __stdcall PlayerStats::Update(PlayerStats_o *this, const MethodInfo *method)
    static PSUpdate: unsafe extern "system" fn(*mut c_void, *const c_void) -> c_void;
    /// void __stdcall Elevator__set_Networkm_InternalElevatorCode(Elevator_o *this, System_String_o *value, const MethodInfo *method)
    static EInternalElevatorCode: unsafe extern "system" fn(*mut c_void, *const UnityString, *const c_void) -> c_void;
}

type FnPSServerTakeDamage = unsafe extern "system" fn(*mut c_void, f32, *const c_void) -> c_void;
type FnPSUpdate = unsafe extern "system" fn(*mut c_void, *const c_void) -> c_void;
type FnEInternalElevatorCode = unsafe extern "system" fn(*mut c_void, *const UnityString, *const c_void) -> c_void;


fn server_take_damage(this: *mut c_void, damage: f32, method: *const c_void) -> c_void {
    println!("Called PlayerStats::ServerTakeDamage(void*, {}, void*).", damage);
    unsafe { PSServerTakeDamage.call(this, 0.0, method) }
}

fn update(this: *mut c_void, method: *const c_void) -> c_void {
    unsafe { PSUpdate.call(this, method) }
}

fn internal_elevator_code(this: *mut c_void, string: *const UnityString, method: *const c_void) -> c_void {
    let length = unsafe { (*string).length };
    println!("Elevator code length: {}", length);
    let buffer: Vec<u16> = unsafe {
        let mut buffer = Vec::<u16>::new();
        for idx in 0..length {
            buffer.push((*string).buffer[idx as usize]);
        }
        buffer.push(0);
        buffer
    };
    println!("Elevator code: {}", String::from_utf16_lossy(&buffer));
    unsafe { EInternalElevatorCode.call(this, string, method) }
}

#[no_mangle]
extern "system" fn DllMain(_: HINSTANCE, fdwReason: DWORD, _: LPVOID) -> BOOL {
    if fdwReason == DLL_PROCESS_ATTACH {
        unsafe { AllocConsole(); }

        let module = "GameAssembly.dll"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect::<Vec<u16>>();
        let base = unsafe { GetModuleHandleW(module.as_ptr()) };
        let server_take_damage_addr: FnPSServerTakeDamage = unsafe { mem::transmute(base as usize + 0x6536A0) };
        let update_addr: FnPSUpdate = unsafe { mem::transmute(base as usize + 0x653E00) };
        let elevator_code_addr: FnEInternalElevatorCode = unsafe { mem::transmute(base as usize + 0x69E470) };
        unsafe { 
            PSServerTakeDamage
                .initialize(server_take_damage_addr, server_take_damage)
                .unwrap()
                .enable()
                .unwrap();
            PSUpdate
                .initialize(update_addr, update)
                .unwrap()
                .enable()
                .unwrap();
            EInternalElevatorCode
                .initialize(elevator_code_addr, internal_elevator_code)
                .unwrap()
                .enable()
                .unwrap();
        }
    }

    1
}