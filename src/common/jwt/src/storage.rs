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

use crate::json_web_token::RevokedAccessToken;
use axum::async_trait;
use common_base::{error::common::CommonError, utils::expiry};
use rocksdb_engine::RocksDBEngine;
use std::sync::Arc;

const TOKEN_COLUMN_FAMILY: &str = "tokendb";

#[async_trait]
pub trait TokenStorage: Send + Sync {
    async fn save_revoked_access_token(&self, token: RevokedAccessToken)
        -> Result<(), CommonError>;
    async fn delete_revoked_access_tokens(&self, id: String) -> Result<(), CommonError>;
    async fn load_all_revoked_access_tokens(&self) -> Result<Vec<RevokedAccessToken>, CommonError>;
}

pub struct RokcsDBTokenStorageAdapter {
    pub db: Arc<RocksDBEngine>,
}

#[async_trait]
impl TokenStorage for RokcsDBTokenStorageAdapter {
    async fn load_all_revoked_access_tokens(&self) -> Result<Vec<RevokedAccessToken>, CommonError> {
        let cf = self.db.cf_handle(TOKEN_COLUMN_FAMILY).unwrap();
        self.db
            .read_all_by_cf(cf)?
            .into_iter()
            .map(|x| {
                let id = x.0;
                let expiry_str = String::from_utf8(x.1).unwrap();
                let expiry = expiry_str.parse::<u64>().unwrap();
                Ok(RevokedAccessToken { id, expiry })
            })
            .collect()
    }

    async fn save_revoked_access_token(
        &self,
        token: RevokedAccessToken,
    ) -> Result<(), CommonError> {
        let cf = self.db.cf_handle(TOKEN_COLUMN_FAMILY).unwrap();
        self.db.write(cf, &token.id, &token.expiry)
    }

    async fn delete_revoked_access_tokens(&self, id: String) -> Result<(), CommonError> {
        let cf = self.db.cf_handle(TOKEN_COLUMN_FAMILY).unwrap();
        self.db.delete(cf, id.as_ref())
    }
}

impl RokcsDBTokenStorageAdapter {
    pub fn new(path: impl AsRef<str>) -> Self {
        RokcsDBTokenStorageAdapter {
            db: Arc::new(RocksDBEngine::new(
                path.as_ref(),
                1,
                vec![TOKEN_COLUMN_FAMILY.to_string()],
            )),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use common_base::tools::unique_id;

    #[tokio::test]
    async fn concurrency_test() {
        let db_path = format!("/tmp/robustmq_{}", unique_id());
        {
            let storage_adapter = RokcsDBTokenStorageAdapter::new(db_path.as_str());
            let id = "7355608".to_string();
            let token = RevokedAccessToken {
                id: id.clone(),
                expiry: 0,
            };
            storage_adapter
                .save_revoked_access_token(token)
                .await
                .unwrap();
            let tokens = storage_adapter
                .load_all_revoked_access_tokens()
                .await
                .unwrap();

            assert_eq!(tokens.len(), 1);

            storage_adapter
                .delete_revoked_access_tokens(id.clone())
                .await
                .unwrap();
            let tokens = storage_adapter
                .load_all_revoked_access_tokens()
                .await
                .unwrap();

            assert_eq!(tokens.len(), 0);
        }
        let _ = std::fs::remove_dir_all(&db_path);
    }
}
