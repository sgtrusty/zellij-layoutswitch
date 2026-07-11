#[test]
fn host_run_plugin_command_stub_is_callable() {
    #[cfg(not(target_arch = "wasm32"))]
    unsafe {
        crate::host_run_plugin_command();
    }
}
