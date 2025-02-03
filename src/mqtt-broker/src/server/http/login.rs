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

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use axum::extract::State;
use common_base::http_response::success_response;
use protocol::mqtt::common::Login;

use crate::security::login::plaintext::Plaintext;

use super::server::HttpServerState;

use axum::{extract::Json, http::StatusCode, routing::post, Router};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    token: String,
}

pub async fn http_login(
    State(state): State<Arc<HttpServerState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let login = Login {
        username: payload.username.clone(),
        password: payload.password.clone(),
    };
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    match state
        .auth_driver
        .check_login_auth(&Some(login), &None, &socket)
        .await
    {
        Ok(flag) => {
            if flag {
                let token = state
                    .jwt_manager
                    .generate(payload.username.clone())
                    .await
                    .unwrap()
                    .access_token;
                return Ok(Json(LoginResponse { token }));
            } else {
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
        Err(e) => {
            return Err(StatusCode::UNAUTHORIZED);
        }
    }
}
