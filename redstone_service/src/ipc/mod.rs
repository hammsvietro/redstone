use redstone_common::constants::IPC_SOCKET_PATH;

pub mod clone;
pub mod pull;
pub mod push;
pub mod socket_loop;
pub mod track;

pub fn assert_socket_is_available() {
    let _ = std::fs::remove_file(IPC_SOCKET_PATH);
}
