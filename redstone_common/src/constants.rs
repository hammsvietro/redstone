pub const IPC_SOCKET_PATH: &'static str = "/tmp/redstone.sock";

#[cfg(target_os = "linux")]
pub const IPC_BUFFER_SIZE: usize = 16384;

#[cfg(target_os = "macos")]
pub const IPC_BUFFER_SIZE: usize = 8192;

pub const TCP_FILE_CHUNK_SIZE: usize = 1_048_576; // 1MB
