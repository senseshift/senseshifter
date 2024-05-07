#![allow(dead_code)]

use uuid::Uuid;

/// Main service for communication: `6e400001-b5a3-f393-e0a9-e50e24dcca9e`.
///
/// It is literally Nordic UART service.
pub const SERVICE_MOTOR: Uuid = Uuid::from_u128(0x6e400001_b5a3_f393_e0a9_e50e24dcca9e);

/// Legacy characteristic for writing motor commands: `6e400002-b5a3-f393-e0a9-e50e24dcca9e`.
///
/// It is literally Nordic UART TX characteristic, but used to send motor commands.
pub const CHAR_MOTOR_WRITE: Uuid = Uuid::from_u128(0x6e400002_b5a3_f393_e0a9_e50e24dcca9e);

/// Device Serial Number characteristic: `6e400003-b5a3-f393-e0a9-e50e24dcca9e`.
///
/// It is literally Nordic UART RX characteristic, but used to receive device serial number.
/// ¯\_(ツ)_/¯
pub const CHAR_SN: Uuid = Uuid::from_u128(0x6e400003_b5a3_f393_e0a9_e50e24dcca9e);

/// Device Config characteristic: `6e400005-b5a3-f393-e0a9-e50e24dcca9e`.
///
/// Used to update glow color.
pub const CHAR_CONFIG: Uuid = Uuid::from_u128(0x6e400005_b5a3_f393_e0a9_e50e24dcca9e);

/// Device Version characteristic: `6e400007-b5a3-f393-e0a9-e50e24dcca9e`.
///
/// Used to get device firmware version.
pub const CHAR_VERSION: Uuid = Uuid::from_u128(0x6e400007_b5a3_f393_e0a9_e50e24dcca9e);

/// Device Battery characteristic: `6e400008-b5a3-f393-e0a9-e50e24dcca9e`.
///
/// Used to get device battery level.
pub const CHAR_BATTERY: Uuid = Uuid::from_u128(0x6e400008_b5a3_f393_e0a9_e50e24dcca9e);

/// Current characteristic for writing motor commands: `6e40000a-b5a3-f393-e0a9-e50e24dcca9e`.
pub const CHAR_MOTOR_WRITE_STABLE: Uuid = Uuid::from_u128(0x6e40000a_b5a3_f393_e0a9_e50e24dcca9e);
