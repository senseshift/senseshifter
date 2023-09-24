#[cfg_attr(
    feature = "serde",
    derive(::serde::Serialize, ::serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "specta", derive(::specta::Type))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RawDevice {
    #[prost(oneof = "raw_device::Type", tags = "21, 22, 23")]
    pub r#type: ::core::option::Option<raw_device::Type>,
}
/// Nested message and enum types in `RawDevice`.
pub mod raw_device {
    #[cfg_attr(
        feature = "serde",
        derive(::serde::Serialize, ::serde::Deserialize),
        serde(rename_all = "snake_case")
    )]
    #[cfg_attr(feature = "specta", derive(::specta::Type))]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Serial {
        #[prost(string, tag = "1")]
        pub port: ::prost::alloc::string::String,
    }
    #[cfg_attr(
        feature = "serde",
        derive(::serde::Serialize, ::serde::Deserialize),
        serde(rename_all = "snake_case")
    )]
    #[cfg_attr(feature = "specta", derive(::specta::Type))]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct RFComm {
        /// The MAC address of the device.
        #[prost(string, tag = "1")]
        pub address: ::prost::alloc::string::String,
        /// The name of the device.
        #[prost(string, tag = "2")]
        pub name: ::prost::alloc::string::String,
    }
    #[cfg_attr(
        feature = "serde",
        derive(::serde::Serialize, ::serde::Deserialize),
        serde(rename_all = "snake_case")
    )]
    #[cfg_attr(feature = "specta", derive(::specta::Type))]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct BluetoothLE {
        /// The MAC address of the device.
        #[prost(string, tag = "1")]
        pub address: ::prost::alloc::string::String,
        /// The name of the device.
        #[prost(string, tag = "2")]
        pub name: ::prost::alloc::string::String,
    }
    #[cfg_attr(
        feature = "serde",
        derive(::serde::Serialize, ::serde::Deserialize),
        serde(rename_all = "snake_case")
    )]
    #[cfg_attr(feature = "specta", derive(::specta::Type))]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Type {
        #[prost(message, tag = "21")]
        Serial(Serial),
        #[prost(message, tag = "22")]
        RFComm(RFComm),
        #[prost(message, tag = "23")]
        BluetoothLE(BluetoothLE),
    }
}
#[cfg_attr(
    feature = "serde",
    derive(::serde::Serialize, ::serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "specta", derive(::specta::Type))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Device {
    #[prost(string, tag = "1")]
    pub device_id: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "2")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(enumeration = "device::Status", tag = "3")]
    pub status: i32,
    #[prost(bool, tag = "4")]
    pub connectible: bool,
    #[prost(message, optional, tag = "51")]
    pub info: ::core::option::Option<device::Info>,
    #[prost(message, optional, tag = "52")]
    pub properties: ::core::option::Option<device::Properties>,
}
/// Nested message and enum types in `Device`.
pub mod device {
    #[cfg_attr(
        feature = "serde",
        derive(::serde::Serialize, ::serde::Deserialize),
        serde(rename_all = "snake_case")
    )]
    #[cfg_attr(feature = "specta", derive(::specta::Type))]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Info {
        #[prost(string, tag = "1")]
        pub manufacturer: ::prost::alloc::string::String,
        #[prost(string, tag = "2")]
        pub model: ::prost::alloc::string::String,
    }
    #[cfg_attr(
        feature = "serde",
        derive(::serde::Serialize, ::serde::Deserialize),
        serde(rename_all = "snake_case")
    )]
    #[cfg_attr(feature = "specta", derive(::specta::Type))]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Properties {
        #[prost(string, optional, tag = "1")]
        pub serial_number: ::core::option::Option<::prost::alloc::string::String>,
        #[prost(string, optional, tag = "2")]
        pub firmware_version: ::core::option::Option<::prost::alloc::string::String>,
        #[prost(message, optional, tag = "3")]
        pub battery: ::core::option::Option<properties::Battery>,
    }
    /// Nested message and enum types in `Properties`.
    pub mod properties {
        #[cfg_attr(
            feature = "serde",
            derive(::serde::Serialize, ::serde::Deserialize),
            serde(rename_all = "snake_case")
        )]
        #[cfg_attr(feature = "specta", derive(::specta::Type))]
        #[allow(clippy::derive_partial_eq_without_eq)]
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct Battery {
            #[prost(enumeration = "battery::State", optional, tag = "1")]
            pub state: ::core::option::Option<i32>,
            #[prost(float, repeated, tag = "2")]
            pub levels: ::prost::alloc::vec::Vec<f32>,
        }
        /// Nested message and enum types in `Battery`.
        pub mod battery {
            #[cfg_attr(
                feature = "serde",
                derive(::serde::Serialize, ::serde::Deserialize),
                serde(rename_all = "snake_case")
            )]
            #[cfg_attr(feature = "specta", derive(::specta::Type))]
            #[derive(
                Clone,
                Copy,
                Debug,
                PartialEq,
                Eq,
                Hash,
                PartialOrd,
                Ord,
                ::prost::Enumeration
            )]
            #[repr(i32)]
            pub enum State {
                Unknown = 0,
                Charging = 1,
                Discharging = 2,
                NotCharging = 3,
                Full = 4,
            }
            impl State {
                /// String value of the enum field names used in the ProtoBuf definition.
                ///
                /// The values are not transformed in any way and thus are considered stable
                /// (if the ProtoBuf definition does not change) and safe for programmatic use.
                pub fn as_str_name(&self) -> &'static str {
                    match self {
                        State::Unknown => "STATE_UNKNOWN",
                        State::Charging => "STATE_CHARGING",
                        State::Discharging => "STATE_DISCHARGING",
                        State::NotCharging => "STATE_NOT_CHARGING",
                        State::Full => "STATE_FULL",
                    }
                }
                /// Creates an enum from field names used in the ProtoBuf definition.
                pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
                    match value {
                        "STATE_UNKNOWN" => Some(Self::Unknown),
                        "STATE_CHARGING" => Some(Self::Charging),
                        "STATE_DISCHARGING" => Some(Self::Discharging),
                        "STATE_NOT_CHARGING" => Some(Self::NotCharging),
                        "STATE_FULL" => Some(Self::Full),
                        _ => None,
                    }
                }
            }
        }
    }
    #[cfg_attr(
        feature = "serde",
        derive(::serde::Serialize, ::serde::Deserialize),
        serde(rename_all = "snake_case")
    )]
    #[cfg_attr(feature = "specta", derive(::specta::Type))]
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum Transport {
        Unknown = 0,
        Ble = 1,
        Serial = 2,
        RFComm = 3,
    }
    impl Transport {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Transport::Unknown => "TRANSPORT_UNKNOWN",
                Transport::Ble => "TRANSPORT_BLE",
                Transport::Serial => "TRANSPORT_SERIAL",
                Transport::RFComm => "TRANSPORT_RFCOMM",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "TRANSPORT_UNKNOWN" => Some(Self::Unknown),
                "TRANSPORT_BLE" => Some(Self::Ble),
                "TRANSPORT_SERIAL" => Some(Self::Serial),
                "TRANSPORT_RFCOMM" => Some(Self::RFComm),
                _ => None,
            }
        }
    }
    #[cfg_attr(
        feature = "serde",
        derive(::serde::Serialize, ::serde::Deserialize),
        serde(rename_all = "snake_case")
    )]
    #[cfg_attr(feature = "specta", derive(::specta::Type))]
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum Status {
        Unknown = 0,
        Disconnected = 1,
        Connecting = 2,
        Connected = 3,
        Disconnecting = 4,
    }
    impl Status {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Status::Unknown => "STATUS_UNKNOWN",
                Status::Disconnected => "STATUS_DISCONNECTED",
                Status::Connecting => "STATUS_CONNECTING",
                Status::Connected => "STATUS_CONNECTED",
                Status::Disconnecting => "STATUS_DISCONNECTING",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "STATUS_UNKNOWN" => Some(Self::Unknown),
                "STATUS_DISCONNECTED" => Some(Self::Disconnected),
                "STATUS_CONNECTING" => Some(Self::Connecting),
                "STATUS_CONNECTED" => Some(Self::Connected),
                "STATUS_DISCONNECTING" => Some(Self::Disconnecting),
                _ => None,
            }
        }
    }
    #[cfg_attr(
        feature = "serde",
        derive(::serde::Serialize, ::serde::Deserialize),
        serde(rename_all = "snake_case")
    )]
    #[cfg_attr(feature = "specta", derive(::specta::Type))]
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum Chirality {
        Unknown = 0,
        Left = 1,
        Right = 2,
    }
    impl Chirality {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Chirality::Unknown => "CHIRALITY_UNKNOWN",
                Chirality::Left => "CHIRALITY_LEFT",
                Chirality::Right => "CHIRALITY_RIGHT",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "CHIRALITY_UNKNOWN" => Some(Self::Unknown),
                "CHIRALITY_LEFT" => Some(Self::Left),
                "CHIRALITY_RIGHT" => Some(Self::Right),
                _ => None,
            }
        }
    }
}
#[cfg_attr(
    feature = "serde",
    derive(::serde::Serialize, ::serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "specta", derive(::specta::Type))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeviceMessage {
    #[prost(oneof = "device_message::Type", tags = "121, 122, 151, 152, 153")]
    pub r#type: ::core::option::Option<device_message::Type>,
}
/// Nested message and enum types in `DeviceMessage`.
pub mod device_message {
    #[cfg_attr(
        feature = "serde",
        derive(::serde::Serialize, ::serde::Deserialize),
        serde(rename_all = "snake_case")
    )]
    #[cfg_attr(feature = "specta", derive(::specta::Type))]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ScanStarted {}
    #[cfg_attr(
        feature = "serde",
        derive(::serde::Serialize, ::serde::Deserialize),
        serde(rename_all = "snake_case")
    )]
    #[cfg_attr(feature = "specta", derive(::specta::Type))]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ScanStopped {}
    #[cfg_attr(
        feature = "serde",
        derive(::serde::Serialize, ::serde::Deserialize),
        serde(rename_all = "snake_case")
    )]
    #[cfg_attr(feature = "specta", derive(::specta::Type))]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct DeviceDiscovered {
        #[prost(string, tag = "1")]
        pub device_id: ::prost::alloc::string::String,
        #[prost(message, optional, tag = "2")]
        pub device: ::core::option::Option<super::Device>,
    }
    #[cfg_attr(
        feature = "serde",
        derive(::serde::Serialize, ::serde::Deserialize),
        serde(rename_all = "snake_case")
    )]
    #[cfg_attr(feature = "specta", derive(::specta::Type))]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct DeviceConnected {
        #[prost(string, tag = "1")]
        pub device_id: ::prost::alloc::string::String,
        #[prost(message, optional, tag = "2")]
        pub device: ::core::option::Option<super::Device>,
    }
    #[cfg_attr(
        feature = "serde",
        derive(::serde::Serialize, ::serde::Deserialize),
        serde(rename_all = "snake_case")
    )]
    #[cfg_attr(feature = "specta", derive(::specta::Type))]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct DeviceUpdated {
        #[prost(string, tag = "1")]
        pub device_id: ::prost::alloc::string::String,
        #[prost(message, optional, tag = "2")]
        pub device: ::core::option::Option<super::Device>,
    }
    #[cfg_attr(
        feature = "serde",
        derive(::serde::Serialize, ::serde::Deserialize),
        serde(rename_all = "snake_case")
    )]
    #[cfg_attr(feature = "specta", derive(::specta::Type))]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Type {
        #[prost(message, tag = "121")]
        ScanStarted(ScanStarted),
        #[prost(message, tag = "122")]
        ScanStopped(ScanStopped),
        #[prost(message, tag = "151")]
        DeviceDiscovered(DeviceDiscovered),
        #[prost(message, tag = "152")]
        DeviceConnected(DeviceConnected),
        #[prost(message, tag = "153")]
        DeviceUpdated(DeviceUpdated),
    }
}
#[cfg_attr(
    feature = "serde",
    derive(::serde::Serialize, ::serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "specta", derive(::specta::Type))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventStreamRequest {}
#[cfg_attr(
    feature = "serde",
    derive(::serde::Serialize, ::serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "specta", derive(::specta::Type))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventStreamResponse {
    #[prost(message, optional, tag = "1")]
    pub message: ::core::option::Option<DeviceMessage>,
}
#[cfg_attr(
    feature = "serde",
    derive(::serde::Serialize, ::serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "specta", derive(::specta::Type))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ScanStartRequest {}
#[cfg_attr(
    feature = "serde",
    derive(::serde::Serialize, ::serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "specta", derive(::specta::Type))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ScanStartResponse {}
#[cfg_attr(
    feature = "serde",
    derive(::serde::Serialize, ::serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "specta", derive(::specta::Type))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ScanStopRequest {}
#[cfg_attr(
    feature = "serde",
    derive(::serde::Serialize, ::serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "specta", derive(::specta::Type))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ScanStopResponse {}
#[cfg_attr(
    feature = "serde",
    derive(::serde::Serialize, ::serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "specta", derive(::specta::Type))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeviceConnectRequest {
    #[prost(string, tag = "1")]
    pub device_id: ::prost::alloc::string::String,
}
#[cfg_attr(
    feature = "serde",
    derive(::serde::Serialize, ::serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "specta", derive(::specta::Type))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeviceConnectResponse {
    #[prost(message, optional, tag = "1")]
    pub device: ::core::option::Option<Device>,
}
#[cfg_attr(
    feature = "serde",
    derive(::serde::Serialize, ::serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "specta", derive(::specta::Type))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeviceDisconnectRequest {
    #[prost(string, tag = "1")]
    pub device_id: ::prost::alloc::string::String,
}
#[cfg_attr(
    feature = "serde",
    derive(::serde::Serialize, ::serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "specta", derive(::specta::Type))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeviceDisconnectResponse {
    #[prost(message, optional, tag = "1")]
    pub device: ::core::option::Option<Device>,
}
#[cfg_attr(
    feature = "serde",
    derive(::serde::Serialize, ::serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "specta", derive(::specta::Type))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeviceAddRequest {
    #[prost(oneof = "device_add_request::Type", tags = "11")]
    pub r#type: ::core::option::Option<device_add_request::Type>,
}
/// Nested message and enum types in `DeviceAddRequest`.
pub mod device_add_request {
    #[cfg_attr(
        feature = "serde",
        derive(::serde::Serialize, ::serde::Deserialize),
        serde(rename_all = "snake_case")
    )]
    #[cfg_attr(feature = "specta", derive(::specta::Type))]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct OpenGlove {
        #[prost(oneof = "openglove::Transport", tags = "21, 22, 23")]
        pub transport: ::core::option::Option<openglove::Transport>,
    }
    /// Nested message and enum types in `OpenGlove`.
    pub mod openglove {
        #[cfg_attr(
            feature = "serde",
            derive(::serde::Serialize, ::serde::Deserialize),
            serde(rename_all = "snake_case")
        )]
        #[cfg_attr(feature = "specta", derive(::specta::Type))]
        #[allow(clippy::derive_partial_eq_without_eq)]
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct SerialPort {
            #[prost(string, tag = "1")]
            pub port: ::prost::alloc::string::String,
            #[prost(int32, optional, tag = "2")]
            pub baud_rate: ::core::option::Option<i32>,
        }
        #[cfg_attr(
            feature = "serde",
            derive(::serde::Serialize, ::serde::Deserialize),
            serde(rename_all = "snake_case")
        )]
        #[cfg_attr(feature = "specta", derive(::specta::Type))]
        #[allow(clippy::derive_partial_eq_without_eq)]
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct RFComm {
            /// The MAC address of the device.
            #[prost(string, tag = "1")]
            pub address: ::prost::alloc::string::String,
        }
        #[cfg_attr(
            feature = "serde",
            derive(::serde::Serialize, ::serde::Deserialize),
            serde(rename_all = "snake_case")
        )]
        #[cfg_attr(feature = "specta", derive(::specta::Type))]
        #[allow(clippy::derive_partial_eq_without_eq)]
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct BluetoothLE {
            /// The MAC address of the device.
            #[prost(string, tag = "1")]
            pub address: ::prost::alloc::string::String,
            #[prost(string, optional, tag = "11")]
            pub service_uuid: ::core::option::Option<::prost::alloc::string::String>,
            #[prost(string, optional, tag = "12")]
            pub rx_uuid: ::core::option::Option<::prost::alloc::string::String>,
            #[prost(string, optional, tag = "13")]
            pub tx_uuid: ::core::option::Option<::prost::alloc::string::String>,
        }
        #[cfg_attr(
            feature = "serde",
            derive(::serde::Serialize, ::serde::Deserialize),
            serde(rename_all = "snake_case")
        )]
        #[cfg_attr(feature = "specta", derive(::specta::Type))]
        #[allow(clippy::derive_partial_eq_without_eq)]
        #[derive(Clone, PartialEq, ::prost::Oneof)]
        pub enum Transport {
            #[prost(message, tag = "21")]
            SerialPort(SerialPort),
            #[prost(message, tag = "22")]
            RFComm(RFComm),
            #[prost(message, tag = "23")]
            BluetoothLE(BluetoothLE),
        }
    }
    #[cfg_attr(
        feature = "serde",
        derive(::serde::Serialize, ::serde::Deserialize),
        serde(rename_all = "snake_case")
    )]
    #[cfg_attr(feature = "specta", derive(::specta::Type))]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Type {
        #[prost(message, tag = "11")]
        OpenGlove(OpenGlove),
    }
}
#[cfg_attr(
    feature = "serde",
    derive(::serde::Serialize, ::serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "specta", derive(::specta::Type))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeviceAddResponse {
    #[prost(message, optional, tag = "1")]
    pub device: ::core::option::Option<Device>,
}
#[cfg_attr(
    feature = "serde",
    derive(::serde::Serialize, ::serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "specta", derive(::specta::Type))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RawDeviceRequest {
    #[prost(enumeration = "device::Transport", optional, tag = "1")]
    pub transport: ::core::option::Option<i32>,
}
#[cfg_attr(
    feature = "serde",
    derive(::serde::Serialize, ::serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "specta", derive(::specta::Type))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RawDeviceResponse {
    #[prost(message, repeated, tag = "1")]
    pub raw_devices: ::prost::alloc::vec::Vec<RawDevice>,
}
/// Generated client implementations.
#[cfg(feature = "tonic-client")]
pub mod device_manager_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct DeviceManagerClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl DeviceManagerClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> DeviceManagerClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> DeviceManagerClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            DeviceManagerClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        pub async fn event_stream(
            &mut self,
            request: impl tonic::IntoRequest<super::EventStreamRequest>,
        ) -> std::result::Result<
            tonic::Response<tonic::codec::Streaming<super::EventStreamResponse>>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/xrconnect.devices.v1alpha1.DeviceManager/EventStream",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "xrconnect.devices.v1alpha1.DeviceManager",
                        "EventStream",
                    ),
                );
            self.inner.server_streaming(req, path, codec).await
        }
        pub async fn scan_start(
            &mut self,
            request: impl tonic::IntoRequest<super::ScanStartRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ScanStartResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/xrconnect.devices.v1alpha1.DeviceManager/ScanStart",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "xrconnect.devices.v1alpha1.DeviceManager",
                        "ScanStart",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn scan_stop(
            &mut self,
            request: impl tonic::IntoRequest<super::ScanStopRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ScanStopResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/xrconnect.devices.v1alpha1.DeviceManager/ScanStop",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "xrconnect.devices.v1alpha1.DeviceManager",
                        "ScanStop",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn device_connect(
            &mut self,
            request: impl tonic::IntoRequest<super::DeviceConnectRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeviceConnectResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/xrconnect.devices.v1alpha1.DeviceManager/DeviceConnect",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "xrconnect.devices.v1alpha1.DeviceManager",
                        "DeviceConnect",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn device_disconnect(
            &mut self,
            request: impl tonic::IntoRequest<super::DeviceDisconnectRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeviceDisconnectResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/xrconnect.devices.v1alpha1.DeviceManager/DeviceDisconnect",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "xrconnect.devices.v1alpha1.DeviceManager",
                        "DeviceDisconnect",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn device_add(
            &mut self,
            request: impl tonic::IntoRequest<super::DeviceAddRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeviceAddResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/xrconnect.devices.v1alpha1.DeviceManager/DeviceAdd",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "xrconnect.devices.v1alpha1.DeviceManager",
                        "DeviceAdd",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_raw_devices(
            &mut self,
            request: impl tonic::IntoRequest<super::RawDeviceRequest>,
        ) -> std::result::Result<
            tonic::Response<super::RawDeviceResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/xrconnect.devices.v1alpha1.DeviceManager/GetRawDevices",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "xrconnect.devices.v1alpha1.DeviceManager",
                        "GetRawDevices",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
#[cfg(feature = "tonic-server")]
pub mod device_manager_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with DeviceManagerServer.
    #[async_trait]
    pub trait DeviceManager: Send + Sync + 'static {
        /// Server streaming response type for the EventStream method.
        type EventStreamStream: tonic::codegen::tokio_stream::Stream<
                Item = std::result::Result<super::EventStreamResponse, tonic::Status>,
            >
            + Send
            + 'static;
        async fn event_stream(
            &self,
            request: tonic::Request<super::EventStreamRequest>,
        ) -> std::result::Result<
            tonic::Response<Self::EventStreamStream>,
            tonic::Status,
        >;
        async fn scan_start(
            &self,
            request: tonic::Request<super::ScanStartRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ScanStartResponse>,
            tonic::Status,
        >;
        async fn scan_stop(
            &self,
            request: tonic::Request<super::ScanStopRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ScanStopResponse>,
            tonic::Status,
        >;
        async fn device_connect(
            &self,
            request: tonic::Request<super::DeviceConnectRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeviceConnectResponse>,
            tonic::Status,
        >;
        async fn device_disconnect(
            &self,
            request: tonic::Request<super::DeviceDisconnectRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeviceDisconnectResponse>,
            tonic::Status,
        >;
        async fn device_add(
            &self,
            request: tonic::Request<super::DeviceAddRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeviceAddResponse>,
            tonic::Status,
        >;
        async fn get_raw_devices(
            &self,
            request: tonic::Request<super::RawDeviceRequest>,
        ) -> std::result::Result<
            tonic::Response<super::RawDeviceResponse>,
            tonic::Status,
        >;
    }
    #[derive(Debug)]
    pub struct DeviceManagerServer<T: DeviceManager> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: DeviceManager> DeviceManagerServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for DeviceManagerServer<T>
    where
        T: DeviceManager,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/xrconnect.devices.v1alpha1.DeviceManager/EventStream" => {
                    #[allow(non_camel_case_types)]
                    struct EventStreamSvc<T: DeviceManager>(pub Arc<T>);
                    impl<
                        T: DeviceManager,
                    > tonic::server::ServerStreamingService<super::EventStreamRequest>
                    for EventStreamSvc<T> {
                        type Response = super::EventStreamResponse;
                        type ResponseStream = T::EventStreamStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::EventStreamRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as DeviceManager>::event_stream(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = EventStreamSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/xrconnect.devices.v1alpha1.DeviceManager/ScanStart" => {
                    #[allow(non_camel_case_types)]
                    struct ScanStartSvc<T: DeviceManager>(pub Arc<T>);
                    impl<
                        T: DeviceManager,
                    > tonic::server::UnaryService<super::ScanStartRequest>
                    for ScanStartSvc<T> {
                        type Response = super::ScanStartResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ScanStartRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as DeviceManager>::scan_start(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ScanStartSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/xrconnect.devices.v1alpha1.DeviceManager/ScanStop" => {
                    #[allow(non_camel_case_types)]
                    struct ScanStopSvc<T: DeviceManager>(pub Arc<T>);
                    impl<
                        T: DeviceManager,
                    > tonic::server::UnaryService<super::ScanStopRequest>
                    for ScanStopSvc<T> {
                        type Response = super::ScanStopResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ScanStopRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as DeviceManager>::scan_stop(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ScanStopSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/xrconnect.devices.v1alpha1.DeviceManager/DeviceConnect" => {
                    #[allow(non_camel_case_types)]
                    struct DeviceConnectSvc<T: DeviceManager>(pub Arc<T>);
                    impl<
                        T: DeviceManager,
                    > tonic::server::UnaryService<super::DeviceConnectRequest>
                    for DeviceConnectSvc<T> {
                        type Response = super::DeviceConnectResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeviceConnectRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as DeviceManager>::device_connect(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeviceConnectSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/xrconnect.devices.v1alpha1.DeviceManager/DeviceDisconnect" => {
                    #[allow(non_camel_case_types)]
                    struct DeviceDisconnectSvc<T: DeviceManager>(pub Arc<T>);
                    impl<
                        T: DeviceManager,
                    > tonic::server::UnaryService<super::DeviceDisconnectRequest>
                    for DeviceDisconnectSvc<T> {
                        type Response = super::DeviceDisconnectResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeviceDisconnectRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as DeviceManager>::device_disconnect(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeviceDisconnectSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/xrconnect.devices.v1alpha1.DeviceManager/DeviceAdd" => {
                    #[allow(non_camel_case_types)]
                    struct DeviceAddSvc<T: DeviceManager>(pub Arc<T>);
                    impl<
                        T: DeviceManager,
                    > tonic::server::UnaryService<super::DeviceAddRequest>
                    for DeviceAddSvc<T> {
                        type Response = super::DeviceAddResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeviceAddRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as DeviceManager>::device_add(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DeviceAddSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/xrconnect.devices.v1alpha1.DeviceManager/GetRawDevices" => {
                    #[allow(non_camel_case_types)]
                    struct GetRawDevicesSvc<T: DeviceManager>(pub Arc<T>);
                    impl<
                        T: DeviceManager,
                    > tonic::server::UnaryService<super::RawDeviceRequest>
                    for GetRawDevicesSvc<T> {
                        type Response = super::RawDeviceResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RawDeviceRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as DeviceManager>::get_raw_devices(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetRawDevicesSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: DeviceManager> Clone for DeviceManagerServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    impl<T: DeviceManager> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: DeviceManager> tonic::server::NamedService for DeviceManagerServer<T> {
        const NAME: &'static str = "xrconnect.devices.v1alpha1.DeviceManager";
    }
}
