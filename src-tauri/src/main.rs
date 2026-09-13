// Previene la consola extra en Windows en release, sin afectar dev.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tflclient_lib::run()
}
