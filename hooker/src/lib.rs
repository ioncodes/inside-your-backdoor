use std::ffi::c_void;
use std::mem;
use retour::static_detour;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::consoleapi::AllocConsole;
use winapi::um::winnt::DLL_PROCESS_ATTACH;
use winapi::shared::minwindef::*;

static_detour! {
    /// void __stdcall PlayerStats::ServerTakeDamage(PlayerStats_o *this, float damage, const MethodInfo *method)
    static PSServerTakeDamage: unsafe extern "system" fn(c_void, f32, c_void) -> c_void;
    /// void __stdcall PlayerStats::Update(PlayerStats_o *this, const MethodInfo *method)
    static PSUpdate: unsafe extern "system" fn(c_void, c_void) -> c_void;
}

type FnPSServerTakeDamage = unsafe extern "system" fn(c_void, f32, c_void) -> c_void;
type FnPSUpdate = unsafe extern "system" fn(c_void, c_void) -> c_void;

fn server_take_damage(this: c_void, damage: f32, method: c_void) -> c_void {
    println!("Called PlayerStats::ServerTakeDamage(void*, {}, void*).", damage);
    unsafe { PSServerTakeDamage.call(this, 0.0, method) }
}

fn update(this: c_void, method: c_void) -> c_void {
    println!("Lmao!!!");
    unsafe { PSUpdate.call(this, method) }
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
        }
    }

    1
}