use dll_syringe::{Syringe, process::OwnedProcess};

fn main() {
    let process = OwnedProcess::find_first_by_name("Inside the Backrooms.exe")
        .expect("Unable to find game process.");
    let syringe = Syringe::for_process(process);
    syringe.inject("hooker.dll")
        .expect("Unable to inject hooker.dll.");
}
