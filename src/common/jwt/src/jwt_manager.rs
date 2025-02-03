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

use crate::json_web_token::{GeneratedToken, JwtClaims, RevokedAccessToken};
use crate::storage::{RokcsDBTokenStorageAdapter, TokenStorage};
use axum::async_trait;
use common_base::error::common::CommonError;
use common_base::tools::unique_id;
use common_base::utils::{
    duration::{RobustMQDuration, SEC_IN_MICRO},
    expiry::RobustMQExpiry,
    time_util::RobustMQTimestamp,
};
use dashmap::DashMap;
use jsonwebtoken::{encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation};
use std::collections::HashMap;
use std::marker::Send;
use std::sync::Arc;

#[async_trait]
pub trait JwtManager: Send + Sync {
    async fn load_revoked_tokens(&self) -> Result<(), CommonError>;
    async fn generate(&self, username: String) -> Result<GeneratedToken, CommonError>;
    async fn decode(
        &self,
        token: &str,
        algorithm: Algorithm,
    ) -> Result<TokenData<JwtClaims>, CommonError>;
    async fn revoke_token(&self, token_id: &str, expiry: u64) -> Result<(), CommonError>;
    //This method should be implemented,
    //Read the expired revocation token and delete it together,
    //This way, there is no need to use an additional thread to maintain a list of the latest revocation tokens
    async fn is_token_revoked(&self, token_id: &str) -> bool;
}

pub struct IssuerOptions {
    pub issuer: String,
    pub audience: String,
    pub access_token_expiry: RobustMQExpiry,
    pub not_before: RobustMQDuration,
    pub key: EncodingKey,
    pub algorithm: Algorithm,
}

pub struct ValidatorOptions {
    pub valid_audiences: Vec<String>,
    pub valid_issuers: Vec<String>,
    pub clock_skew: RobustMQDuration,
    pub key: DecodingKey,
}

pub struct JwtManagerAdapter<S: TokenStorage + Send + Sync + 'static> {
    issuer: IssuerOptions,
    validator: ValidatorOptions,
    tokens_storage: Arc<S>,
    revoked_tokens: DashMap<String, u64>,
    validations: HashMap<Algorithm, Validation>,
}

#[async_trait]
impl<S> JwtManager for JwtManagerAdapter<S>
where
    S: TokenStorage + Send + Sync + 'static,
{
    async fn load_revoked_tokens(&self) -> Result<(), CommonError> {
        let revoked_tokens = self.tokens_storage.load_all_revoked_access_tokens().await?;
        for token in revoked_tokens {
            self.revoked_tokens.insert(token.id, token.expiry);
        }
        Ok(())
    }

    async fn generate(&self, username: String) -> Result<GeneratedToken, CommonError> {
        let header = Header::new(self.issuer.algorithm);
        let now = RobustMQTimestamp::now().to_secs();
        let iat = now;
        let exp = iat
            + (match self.issuer.access_token_expiry {
                RobustMQExpiry::NeverExpire => 1_000_000_000,
                RobustMQExpiry::ServerDefault => 0,
                RobustMQExpiry::ExpireDuration(duration) => duration.as_secs(),
            }) as u64;
        let nbf = iat + self.issuer.not_before.as_secs() as u64;
        let claims = JwtClaims {
            jti: unique_id(),
            sub: username.clone(),
            aud: self.issuer.audience.to_string(),
            iss: self.issuer.issuer.to_string(),
            iat,
            exp,
            nbf,
        };

        let access_token = encode::<JwtClaims>(&header, &claims, &self.issuer.key);
        if let Err(err) = access_token {
            return Err(CommonError::CommonError(format!(
                "Cannot generate JWT token. Error: {}",
                err
            )));
        }
        Ok(GeneratedToken {
            user_id: username.clone(),
            access_token: access_token.unwrap(),
            access_token_expiry: exp,
        })
    }

    async fn decode(
        &self,
        token: &str,
        algorithm: Algorithm,
    ) -> Result<TokenData<JwtClaims>, CommonError> {
        let validation = self.validations.get(&algorithm);
        if validation.is_none() {
            return Err(CommonError::CommonError(format!(
                "Invalid algorithm: {}",
                JwtManagerAdapter::<S>::map_algorithm_to_string(algorithm)
            )));
        }

        let validation = validation.unwrap();
        match jsonwebtoken::decode::<JwtClaims>(token, &self.validator.key, validation) {
            Ok(claims) => Ok(claims),
            _ => Err(CommonError::CommonError("Invalid token".to_string())),
        }
    }

    async fn revoke_token(&self, token_id: &str, expiry: u64) -> Result<(), CommonError> {
        self.revoked_tokens.insert(token_id.to_string(), expiry);
        let revoked_token = RevokedAccessToken {
            id: token_id.to_string(),
            expiry,
        };
        self.tokens_storage
            .save_revoked_access_token(revoked_token)
            .await
    }

    async fn is_token_revoked(&self, token_id: &str) -> bool {
        if self.revoked_tokens.contains_key(token_id) {
            if self.revoked_tokens.get(token_id).unwrap().value()
                <= &RobustMQTimestamp::now().to_secs()
            {
                self.revoked_tokens.remove(token_id);
                self.tokens_storage
                    .delete_revoked_access_tokens(token_id.to_string())
                    .await
                    .unwrap();
            }
            true
        } else {
            false
        }
    }
}

impl<S> JwtManagerAdapter<S>
where
    S: TokenStorage,
{
    pub fn new(
        token_storage: Arc<S>,
        issuer: IssuerOptions,
        validator: ValidatorOptions,
    ) -> Result<Self, CommonError> {
        let validation = JwtManagerAdapter::<S>::create_validation(
            issuer.algorithm,
            &validator.valid_issuers,
            &validator.valid_audiences,
            validator.clock_skew,
        );

        Ok(Self {
            validations: vec![(issuer.algorithm, validation)].into_iter().collect(),
            issuer,
            validator,
            tokens_storage: token_storage,
            revoked_tokens: DashMap::new(),
        })
    }

    fn create_validation(
        algorithm: Algorithm,
        issuers: &[String],
        audiences: &[String],
        clock_skew: RobustMQDuration,
    ) -> Validation {
        let mut validator = Validation::new(algorithm);
        validator.set_issuer(issuers);
        validator.set_audience(audiences);
        validator.leeway = clock_skew.as_secs() as u64;
        validator
    }

    pub async fn refresh_token(&self, token: &str) -> Result<GeneratedToken, CommonError> {
        if token.is_empty() {
            return Err(CommonError::CommonError("token is empty".to_string()));
        }
        let token_header = jsonwebtoken::decode_header(token)
            .map_err(|_| CommonError::CommonError("invalid token".to_string()))?;
        let jwt_claims = self.decode(token, token_header.alg).await?;
        let id = jwt_claims.claims.jti;
        let expiry = jwt_claims.claims.exp;
        self.revoked_tokens.insert(id.clone(), expiry);
        self.tokens_storage
            .save_revoked_access_token(RevokedAccessToken {
                id: id.clone(),
                expiry,
            })
            .await?;
        self.generate(jwt_claims.claims.sub).await
    }

    pub async fn delete_expired_revoked_tokens(&self, now: u64) -> Result<(), CommonError> {
        let mut tokens_to_delete = Vec::new();
        for entry in self.revoked_tokens.iter() {
            if entry.value() <= &now {
                tokens_to_delete.push(entry.key().clone());
            }
        }
        if tokens_to_delete.is_empty() {
            return Ok(());
        }
        for token_id in tokens_to_delete {
            self.revoked_tokens.remove(&token_id);
            self.tokens_storage
                .delete_revoked_access_tokens(token_id)
                .await?;
        }
        Ok(())
    }

    fn map_algorithm_to_string(algorithm: Algorithm) -> String {
        match algorithm {
            Algorithm::HS256 => "HS256",
            Algorithm::HS384 => "HS384",
            Algorithm::HS512 => "HS512",
            Algorithm::RS256 => "RS256",
            Algorithm::RS384 => "RS384",
            Algorithm::RS512 => "RS512",
            _ => "Unknown",
        }
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::RokcsDBTokenStorageAdapter;
    use common_base::utils::duration::SEC_IN_MICRO;
    use jsonwebtoken::Algorithm;
    use std::vec;
    #[tokio::test]
    async fn token_generate() {
        let db_path = format!("/tmp/robustmq_{}", unique_id());
        let algorithm = Algorithm::HS256;
        let issuer: IssuerOptions = IssuerOptions {
            issuer: "robustmq.com".to_string(),
            audience: "robustmq_clients".to_string(),
            access_token_expiry: RobustMQExpiry::ExpireDuration(RobustMQDuration::from(
                SEC_IN_MICRO * 60 * 60,
            )),
            not_before: RobustMQDuration::from(0),
            key: EncodingKey::from_secret("hellorobustmq7355608".as_bytes()),
            algorithm,
        };
        let validator = ValidatorOptions {
            valid_audiences: vec!["robustmq_clients".to_string()],
            valid_issuers: vec!["robustmq.com".to_string()],
            clock_skew: RobustMQDuration::from(5),
            key: DecodingKey::from_secret("hellorobustmq7355608".as_bytes()),
        };
        let storage = Arc::new(RokcsDBTokenStorageAdapter::new(db_path.clone()));
        let jwt_manager = JwtManagerAdapter::new(storage, issuer, validator).unwrap();

        let token = jwt_manager.generate("admin".to_string()).await.unwrap();
        assert!(token.access_token.len() > 0);

        let token_header = jsonwebtoken::decode_header(&token.access_token).unwrap();
        assert_eq!(token_header.alg, Algorithm::HS256);

        let token_id = jwt_manager
            .decode(token.access_token.as_str(), algorithm)
            .await
            .unwrap()
            .claims
            .jti;
        jwt_manager
            .revoke_token(token_id.as_str(), token.access_token_expiry)
            .await
            .unwrap();
        assert!(jwt_manager.is_token_revoked(token_id.as_str()).await);

        drop(jwt_manager);

        let _ = std::fs::remove_dir_all(&db_path);
    }
}
