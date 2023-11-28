/*
 * Copyright (c) 2023 RobustMQ Team
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

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RobustConfig {
    pub addr: String,
    pub broker: Broker,
    pub admin: Admin
}

#[derive(Debug, Deserialize)]
pub struct Broker {
    pub port: Option<u16>,
    pub work_thread: Option<u16>,
}

#[derive(Debug, Deserialize)]
pub struct Admin {
    pub port: Option<u16>,
    pub work_thread: Option<u16>,
}