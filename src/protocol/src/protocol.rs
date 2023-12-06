/*
 * Copyright (c) 2023 robustmq team 
 * 
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 * 
 *     http://www.apache.org/licenses/LICENSE-2.0
 * 
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */


use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::{io, str::Utf8Error, string::FromUtf8Error};

/// This module is the place where all the protocal specifics gets abstracted
/// out and creates structures which are common across protocols. Since, MQTT
/// is the core protocol that this broker supports, a lot of structs closely
/// map to what MQTT specifies in its protocol

// TODO: Handle the cases when there are no properties using Inner struct, so
// handling of properties can be made simplier internally

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Packet {
    Connect(
        Connect,
        Option<ConnectProperties>,
        Option<LastWill>,
        Option<LastWillProperties>,
        Option<Login>,
    ),
    ConnAck(ConnAck, Option<ConnAckProperties>),
    Publish(Publish, Option<PublishProperties>),
    PubAck(PubAck, Option<PubAckProperties>),
    PubRec(PubRec, Option<PubRecProperties>),
    PubRel(PubRel, Option<PubRelProperties>),
    PubComp(PubComp, Option<PubCompProperties>),
}

/// Connection packet initialized by the client
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Connect {
    /// mqtt keep alive interval
    pub keep_alive: u16,
    /// Client ID
    pub client_id: String,
    /// Clean session. Ask the broker to clear previous session
    pub clean_session: bool,
}

/// ConnectProperties can be used in MQTT Version 5
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectProperties {
    /// Expiry interval property after loosing connection
    pub session_expiry_interval: Option<u32>,
    /// Maximum simultaneous packets
    pub receive_maximum: Option<u16>,
    /// Maximum packet size
    pub max_packet_size: Option<u32>,
    /// Maximum mapping integer for a topic
    pub topic_alias_max: Option<u16>,
    pub request_response_info: Option<u8>,
    pub request_problem_info: Option<u8>,
    /// List of user properties
    pub user_properties: Vec<(String, String)>,
    /// Method of authentication
    pub authentication_method: Option<String>,
    /// Authentication data
    pub authentication_data: Option<Bytes>,
}

/// LastWill that broker forwards on behalf of the client
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastWill {
    pub topic: Bytes,
    pub message: Bytes,
    pub qos: QoS,
    pub retain: bool,
}

/// LastWillProperties can be used in MQTT Version 5
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastWillProperties {
    pub delay_interval: Option<u32>,
    pub payload_format_indicator: Option<u8>,
    pub message_expiry_interval: Option<u32>,
    pub content_type: Option<String>,
    pub response_topic: Option<String>,
    pub correlation_data: Option<Bytes>,
    pub user_properties: Vec<(String, String)>,
}

/// Quality of service
#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq, Default, Copy, PartialOrd)]
#[allow(clippy::enum_variant_names)]
pub enum QoS {
    #[default]
    AtMostOnce = 0,
    AtLeastOnce = 1,
    ExactlyOnce = 2,
}

/// Maps a number to QoS
pub fn qos(num: u8) -> Option<QoS> {
    match num {
        0 => Some(QoS::AtMostOnce),
        1 => Some(QoS::AtLeastOnce),
        2 => Some(QoS::ExactlyOnce),
        qos => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Login {
    pub username: String,
    pub password: String,
}

//-----------------------------ConnectAck packet----------------
/// Return code in connack
/// This contains return codes for both MQTT v4(3.11) and v5
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectReturnCode {
    Success,
    RefusedProtocolVersion,
    BadClientId,
    ServiceUnavailable,
    UnspecifiedError,
    MalformedPacket,
    ProtocolError,
    ImplementationSpecificError,
    UnsupportedProtocolVersion,
    ClientIdentifierNotValid,
    BadUserNamePassword,
    NotAuthorized,
    ServerUnavailable,
    ServerBusy,
    Banned,
    BadAuthenticationMethod,
    TopicNameInvalid,
    PacketTooLarge,
    QuotaExceeded,
    PayloadFormatInvalid,
    RetainNotSupported,
    QoSNotSupported,
    UseAnotherServer,
    ServerMoved,
    ConnectionRateExceeded,
}

/// Acknowledgement to connect packet
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnAck {
    pub session_present: bool,
    pub code: ConnectReturnCode,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConnAckProperties {
    pub session_expiry_interval: Option<u32>,
    pub receive_max: Option<u16>,
    pub max_qos: Option<u8>,
    pub retain_available: Option<u8>,
    pub max_packet_size: Option<u32>,
    pub assigned_client_identifier: Option<String>,
    pub topic_alias_max: Option<u16>,
    pub reason_string: Option<String>,
    pub user_properties: Vec<(String, String)>,
    pub wildcard_subscription_available: Option<u8>,
    pub subscription_identifiers_available: Option<u8>,
    pub shared_subscription_available: Option<u8>,
    pub server_keep_alive: Option<u16>,
    pub response_information: Option<String>,
    pub server_reference: Option<String>,
    pub authentication_method: Option<String>,
    pub authentication_data: Option<Bytes>,
}

//----------------------Publish Packet --------------------------------

/// Publish packet
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Publish{
    pub(crate) dup: bool,
    pub(crate) qos: QoS,
    pub(crate) pkid: u16,
    pub retain: bool,
    pub topic: Bytes,
    pub payload: Bytes,
}

impl Publish {
    //Constructor of Publish
    pub fn new<T: Into<Bytes>>(topic: T, payload: T, retain: bool) -> Publish{
        Publish { 
            dup: false, 
            qos: QoS::AtMostOnce,
            pkid: 0, 
            retain, 
            topic: topic.into(),
            payload: payload.into(),
         }
    }

    pub fn is_empty(&self) -> bool {
        false
    }

    pub fn len(&self) -> usize {
        let len = 2 + self.topic.len() + self.payload.len();
        match self.qos == QoS::AtMostOnce{
            true => len,
            false => len + 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PublishProperties {
    pub payload_format_indicator: Option<u8>,
    pub message_expiry_interval: Option<u32>,
    pub topic_alias: Option<u16>, 
    pub response_topic: Option<String>,
    pub correlation_data: Option<Bytes>,
    pub user_properties: Vec<(String, String)>,
    pub subscription_identifiers: Vec<usize>,
    pub content_type: Option<String>,
}

/// Acknowledgement to QoS 1 publish packet
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PubAck {
    pub pkid : u16, 
    pub reason: PubAckReason,
}


/// Return code in puback
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PubAckReason {
    Success, 
    NoMatchingSubscribers,
    UnspecifiedError,
    ImplementationSpecificError,
    NotAuthorized,
    TopicNameInvalid,
    PacketIdentifierInUse,
    QuotaExceeded,
    PayloadFormatInvalid,
}
//-----------------------------PubRec packet---------------------------------
/// Acknowledgement as part 1 to QoS 2 publish packet
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PubRec {
    pub pkid : u16,
    pub reason: PubRecReason,
}

/// Return code in pubrec packet
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PubRecReason {
    Success, 
    NoMatchingSubscribers,
    UnspecifiedError,
    ImplementationSpecificError,
    NotAuthorized,
    TopicNameInvalid,
    PacketIdentifierInUse,
    QuotaExceeded,
    PayloadFormatInvalid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PubRecProperties {
    pub reason_string: Option<String>,
    pub user_properties: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PubAckProperties {
    pub reason_string: Option<String>,
    pub user_properties: Vec<(String, String)>,

}

//--------------------------- PubRel packet -------------------------------

/// Publish release in response to PubRec packet as QoS 2
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PubRel{
    pub pkid: u16,
    pub reason: PubRelReason,
}
/// Return code in pubrel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PubRelReason{
    Success,
    PacketIdentifierNotFound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PubRelProperties {
    pub reason_string: Option<String>,
    pub user_properties: Vec<(String, String)>,

}

//--------------------------- PubComp packet -------------------------------

/// Asssured publish complete as QoS 2 in response to PubRel packet
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PubComp {
    pub pkid: u16,
    pub reason: PubCompReason,
}
/// Return code in pubcomp pcaket
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PubCompReason {
    Success, 
    PacketIdentifierNotFound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PubCompProperties {
    pub reason_string: Option<String>,
    pub user_properties: Vec<(String, String)>,
}

/// Error during serialization and deserialization
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("Invalid return code received as response fro connect = {0}")]
    InvalidConnectReturnCode(u8),
    #[error("Invalid reason = {0}")]
    InvalidReason(u8),
    #[error("Invalid reason = {0}")]
    InvalidRemainingLength(usize),
    #[error("Invalid protocol used")]
    InvalidProtocol,
    #[error("Invalid protocol level {0}. Make sure right port is being used.")]
    InvalidProtocolLevel(u8),
    #[error("Invalid packet format")]
    IncorrectPacketFormat,
    #[error("Invalid packet type = {0}")]
    InvalidPacketType(u8),
    #[error("Invalid retain forward rule = {0}")]
    InvalidRetainForwardRule(u8),
    #[error("Invalid QoS level = {0}")]
    InvalidQoS(u8),
    #[error("Payload is too long")]
    PayloadTooLong,
    #[error("Payload is required = {0}")]
    PayloadNotUtf8(#[from] Utf8Error),
    #[error("Promised boundary crossed, contains {0} bytes")]
    BoundaryCrossed(usize),
    #[error("Packet is malformed")]
    MalformedPacket,
    #[error("Remaining length is malformed")]
    MalformedRemainingLength,
    #[error("Insufficient number of bytes to frame packet, {0} more bytes required")]
    InsufficientBytes(usize),
    #[error("Packet recieved has id Zero")]
    PacketIdZero,
    #[error("Payload size has been exceeded by {0} bytes")]
    PayloadSizeLimitExceeded(usize)

}

pub trait Protocol {

    fn read_mut(&mut self, stream: &mut BytesMut, max_size: usize) -> Result<Packet, Error>;
    fn write(&self, packet: Packet, write: &mut BytesMut) -> Result<usize, Error>; 
}
