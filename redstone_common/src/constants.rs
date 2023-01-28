pub const IPC_SOCKET_PATH: &str = "/tmp/redstone.sock";

#[cfg(target_os = "linux")]
pub const IPC_BUFFER_SIZE: usize = 16384;

#[cfg(target_os = "macos")]
pub const IPC_BUFFER_SIZE: usize = 8192;

pub const TCP_FILE_CHUNK_SIZE: usize = 1024 * 1024 * 14; // 14MB
