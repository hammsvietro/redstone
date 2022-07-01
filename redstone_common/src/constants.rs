pub const IPC_SOCKET_PATH: &'static str = "/tmp/redstone.sock";

#[cfg(target_os = "linux")]
pub const IPC_BUFFER_SIZE: usize = 16384;

#[cfg(target_os = "macos")]
pub const IPC_BUFFER_SIZE: usize = 8192;

// 2^14 -> BitTorrent block size
pub const BLOCK_SIZE: usize = 16384;
