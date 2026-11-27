//! `jimhillr-tofu-state`: OpenTofu's `http` state backend as a Cloudflare
//! Worker, with state in the D1 database `jimhillr-tofu-state` (decision
//! 0009). The protocol is in [`backend`]; this file only connects it to the
//! Workers runtime and D1.
//!
//! Nothing here logs a request body: the bodies are encrypted state, and the
//! Worker never sees the passphrase, but they still stay out of logs.

pub mod backend;

#[cfg(target_arch = "wasm32")]
mod worker_entry {
    use crate::backend::{self, KEEP_VERSIONS, Passwords, StoreError};
    use worker::{
        Context, D1Database, Env, Request, Response, Result, event, wasm_bindgen::JsValue,
    };

    struct D1Store(D1Database);

    fn store_error(e: worker::Error) -> StoreError {
        StoreError(e.to_string())
    }

    impl backend::Store for D1Store {
        async fn state(&self, stack: &str) -> std::result::Result<Option<String>, StoreError> {
            self.0
                .prepare("SELECT body FROM states WHERE stack = ?1 ORDER BY version DESC LIMIT 1")
                .bind(&[JsValue::from_str(stack)])
                .map_err(store_error)?
                .first::<String>(Some("body"))
                .await
                .map_err(store_error)
        }

        async fn put_state(&self, stack: &str, body: &str) -> std::result::Result<(), StoreError> {
            let stack_value = JsValue::from_str(stack);
            let insert = self
                .0
                .prepare(
                    "INSERT INTO states (stack, version, body) \
                     SELECT ?1, COALESCE(MAX(version), 0) + 1, ?2 FROM states WHERE stack = ?1",
                )
                .bind(&[stack_value.clone(), JsValue::from_str(body)])
                .map_err(store_error)?;
            let prune = self
                .0
                .prepare(
                    "DELETE FROM states WHERE stack = ?1 AND version <= \
                     (SELECT MAX(version) FROM states WHERE stack = ?1) - ?2",
                )
                .bind(&[stack_value, JsValue::from_f64(f64::from(KEEP_VERSIONS))])
                .map_err(store_error)?;
            // One batch is one transaction.
            self.0
                .batch(vec![insert, prune])
                .await
                .map_err(store_error)?;
            Ok(())
        }

        async fn lock(
            &self,
            stack: &str,
            id: &str,
            info: &str,
        ) -> std::result::Result<Option<String>, StoreError> {
            let taken = self
                .0
                .prepare(
                    "INSERT INTO locks (stack, id, info) VALUES (?1, ?2, ?3) \
                     ON CONFLICT (stack) DO NOTHING RETURNING id",
                )
                .bind(&[
                    JsValue::from_str(stack),
                    JsValue::from_str(id),
                    JsValue::from_str(info),
                ])
                .map_err(store_error)?
                .first::<String>(Some("id"))
                .await
                .map_err(store_error)?;
            if taken.is_some() {
                return Ok(None);
            }
            // Held by someone else. If they released it in between, report an
            // empty holder; OpenTofu retries and the next LOCK takes it.
            Ok(Some(
                self.holder(stack)
                    .await?
                    .map(|(_, info)| info)
                    .unwrap_or_else(|| "{}".to_string()),
            ))
        }

        async fn holder(
            &self,
            stack: &str,
        ) -> std::result::Result<Option<(String, String)>, StoreError> {
            let rows = self
                .0
                .prepare("SELECT id, info FROM locks WHERE stack = ?1")
                .bind(&[JsValue::from_str(stack)])
                .map_err(store_error)?
                .raw::<String>()
                .await
                .map_err(store_error)?;
            Ok(rows.into_iter().next().and_then(|row| {
                let mut row = row.into_iter();
                Some((row.next()?, row.next()?))
            }))
        }

        async fn unlock(&self, stack: &str, id: &str) -> std::result::Result<(), StoreError> {
            self.0
                .prepare("DELETE FROM locks WHERE stack = ?1 AND id = ?2")
                .bind(&[JsValue::from_str(stack), JsValue::from_str(id)])
                .map_err(store_error)?
                .run()
                .await
                .map_err(store_error)?;
            Ok(())
        }
    }

    fn secret(env: &Env, name: &str) -> String {
        env.secret(name).map(|s| s.to_string()).unwrap_or_default()
    }

    #[event(fetch)]
    async fn fetch(mut req: Request, env: Env, _ctx: Context) -> Result<Response> {
        // `Request::method` maps methods it doesn't know, such as LOCK, to
        // GET, so read the raw one.
        let method = req.inner().method();
        let url = req.url()?;
        let authorization = req.headers().get("Authorization")?;
        let body = req.bytes().await?;
        let read = secret(&env, "TOFU_STATE_READ_PASSWORD");
        let write = secret(&env, "TOFU_STATE_WRITE_PASSWORD");
        let store = D1Store(env.d1("DB")?);
        let response = backend::handle(
            &store,
            Passwords {
                read: &read,
                write: &write,
            },
            backend::Request {
                method: &method,
                path: url.path(),
                query: url.query().unwrap_or(""),
                authorization: authorization.as_deref(),
                body: &body,
            },
        )
        .await;
        if response.status == 500 {
            worker::console_error!("state store error on {method} {}", url.path());
        }
        let out = if response.body.is_empty() {
            Response::empty()?
        } else {
            Response::ok(response.body)?
        };
        let mut out = out.with_status(response.status);
        let content_type = if response.status == 200 || response.status == 409 {
            "application/json"
        } else {
            "text/plain; charset=utf-8"
        };
        out.headers_mut().set("Content-Type", content_type)?;
        out.headers_mut().set("Cache-Control", "no-store")?;
        Ok(out)
    }
}
