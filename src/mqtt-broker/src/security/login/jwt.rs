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

use common_base::utils::duration::{RobustMQDuration, SEC_IN_MICRO};
use common_base::utils::expiry::RobustMQExpiry;
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
use common_base::config::broker_mqtt::broker_mqtt_conf;
use common_jwt::jwt_manager::JwtManager;
use common_jwt::jwt_manager::{IssuerOptions, JwtManagerAdapter, ValidatorOptions};
use common_jwt::storage::RokcsDBTokenStorageAdapter;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey};
use std::sync::{Arc, LazyLock};

pub const ROKCSDBTYPE: &str = "rocksdb";

// 用 trait object 存储不同类型的 JwtManager
pub static JWTMANAGER: LazyLock<Arc<dyn JwtManager + Send + Sync>> = LazyLock::new(|| {
    let jwt_config = &broker_mqtt_conf().jwt;
    match jwt_config.storage_type.as_str() {
        ROKCSDBTYPE => {
            let storage = Arc::new(RokcsDBTokenStorageAdapter::new(
                jwt_config.storage_path.clone(),
            ));

            let issuer: IssuerOptions = IssuerOptions {
                issuer: jwt_config.issuer.clone(),
                audience: jwt_config.audience.clone(),
                access_token_expiry: RobustMQExpiry::ExpireDuration(RobustMQDuration::from(
                    SEC_IN_MICRO * 60 * 60,
                )),
                not_before: jwt_config.not_before.parse().unwrap(),
                key: EncodingKey::from_secret(jwt_config.encoding_secret.as_bytes()),
                algorithm: str_to_algorithm(jwt_config.algorithm.as_str()),
            };
            let validator = ValidatorOptions {
                valid_audiences: jwt_config.valid_audiences.clone(),
                valid_issuers: jwt_config.valid_issuers.clone(),
                clock_skew: jwt_config.clock_skew.parse().unwrap(),
                key: DecodingKey::from_secret(jwt_config.decoding_secret.as_bytes()),
            };

            let jwt_manager = JwtManagerAdapter::new(storage, issuer, validator).unwrap();
            Arc::new(jwt_manager)
        }
        _ => {
            todo!("load jwt manager from config")
        }
    }
});

fn str_to_algorithm(alg: &str) -> Algorithm {
    match alg {
        "HS256" => Algorithm::HS256,
        "HS384" => Algorithm::HS384,
        "HS512" => Algorithm::HS512,
        _ => panic!("unknown algorithm"),
    }
}

pub fn load_global_jwt_manager() -> Arc<dyn JwtManager + Send + Sync> {
    JWTMANAGER.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use common_base::config::broker_mqtt::init_broker_mqtt_conf_by_path;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_jwt_manager() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../../example/mqtt-cluster/mqtt-server/node-1.toml");
        init_broker_mqtt_conf_by_path(path.to_str().unwrap());
        let jwt_manager = load_global_jwt_manager();
        let token = jwt_manager.generate("admin".to_string()).await.unwrap();
        assert!(token.access_token.len() > 0);

        let token_header = jsonwebtoken::decode_header(&token.access_token).unwrap();
        assert_eq!(token_header.alg, Algorithm::HS256);

        let token_id = jwt_manager
            .decode(token.access_token.as_str(), Algorithm::HS256)
            .await
            .unwrap()
            .claims
            .jti;
        jwt_manager
            .revoke_token(token_id.as_str(), token.access_token_expiry)
            .await
            .unwrap();
        assert!(jwt_manager.is_token_revoked(token_id.as_str()).await);
    }
}
