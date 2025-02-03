// Copyright 2023 RobustMQ Team
//
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

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct Identity {
    pub token_id: String,
    pub token_expiry: u64,
    pub user_id: String,
    pub ip_address: SocketAddr,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub jti: String, // JWT ID
    pub iss: String, // Issuer
    pub aud: String, // Audience
    pub sub: String, // Subject
    pub iat: u64,    // Issued At
    pub exp: u64,    // Expiration Time
    pub nbf: u64,    // Not Before
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RevokedAccessToken {
    pub id: String,
    pub expiry: u64,
}

#[derive(Debug)]
pub struct GeneratedToken {
    pub user_id: String,
    pub access_token: String,
    pub access_token_expiry: u64,
}
