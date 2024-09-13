// Copyright 2023 RobustMQ Team
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::fmt;

use common_base::error::common::CommonError;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Clone)]
pub struct MQTTAcl {
    pub resource_type: MQTTAclResourceType,
    pub resource_name: String,
    pub topic: String,
    pub ip: String,
    pub action: MQTTAclAction,
    pub permission: MQTTAclPermission,
}

impl MQTTAcl {
    pub fn encode(&self) -> Result<Vec<u8>, CommonError> {
        return Ok(serde_json::to_vec(&self)?);
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Clone)]
pub enum MQTTAclResourceType {
    ClientId,
    User,
}

impl fmt::Display for MQTTAclResourceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MQTTAclResourceType::ClientId => "ClientId",
                MQTTAclResourceType::User => "User",
            }
        )
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Clone)]
pub enum MQTTAclAction {
    All,
    Subscribe,
    Publish,
    PubSub,
    Retain,
    Qos,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, PartialOrd, Clone)]
pub enum MQTTAclPermission {
    Allow,
    Deny,
}
