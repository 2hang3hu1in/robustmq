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

use std::net::SocketAddr;
use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;
use common_base::config::broker_mqtt::broker_mqtt_conf;
use common_base::error::common::CommonError;
use common_jwt::jwt_manager::JwtManager;
use log::info;

use crate::security::login::jwt::load_global_jwt_manager;
use crate::security::AuthDriver;

use super::connection::connection_list;
use super::login::http_login;
use super::prometheus::metrics;
use super::publish::http_publish;

pub const ROUTE_PUBLISH: &str = "/publish";
pub const ROUTE_CONNECTION: &str = "/connection";
pub const ROUTE_METRICS: &str = "/metrics";
pub const ROUTE_LOGIN: &str = "/login";

#[derive(Clone)]
pub struct HttpServerState {
    pub jwt_manager: Arc<dyn JwtManager + Send + Sync>,
    pub auth_driver: Arc<AuthDriver>,
}

impl HttpServerState {
    pub fn new(auth_driver: Arc<AuthDriver>) -> Self {
        Self {
            jwt_manager: load_global_jwt_manager(),
            auth_driver,
        }
    }
}

pub async fn start_http_server(state: Arc<HttpServerState>) -> Result<(), CommonError> {
    let config = broker_mqtt_conf();
    let ip: SocketAddr = format!("0.0.0.0:{}", config.http_port).parse()?;
    let app = routes_v1(state);
    let listener = tokio::net::TcpListener::bind(ip).await?;
    info!(
        "Broker HTTP Server start success. bind addr:{}",
        config.http_port
    );
    axum::serve(listener, app).await?;
    Ok(())
}

fn routes_v1(state: Arc<HttpServerState>) -> Router {
    Router::<Arc<HttpServerState>>::new()
        .route(ROUTE_PUBLISH, get(http_publish))
        .route(ROUTE_CONNECTION, get(connection_list))
        .route(ROUTE_METRICS, get(metrics))
        .route(ROUTE_LOGIN, post(http_login))
        .with_state(state)
}
