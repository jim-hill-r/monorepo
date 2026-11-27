#![allow(non_camel_case_types)]

use serde::Serialize;

#[derive(Serialize)]
struct ProviderGithubData_ {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    alias: std::option::Option<std::string::String>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    base_url: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    cache_path: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    insecure: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    legacy_client: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    max_per_page: std::option::Option<terrars::PrimField<f64>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    max_retries: std::option::Option<terrars::PrimField<f64>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    organization: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    owner: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    parallel_requests: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    read_delay_ms: std::option::Option<terrars::PrimField<f64>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    retry_delay_ms: std::option::Option<terrars::PrimField<f64>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    retryable_errors: std::option::Option<terrars::ListField<terrars::PrimField<f64>>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    token: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    write_delay_ms: std::option::Option<terrars::PrimField<f64>>,
}

struct ProviderGithub_ {
    data: std::cell::RefCell<ProviderGithubData_>,
}

pub struct ProviderGithub(std::rc::Rc<ProviderGithub_>);

impl ProviderGithub {
    pub fn provider_ref(&self) -> std::string::String {
        let data = self.0.data.borrow();
        if let std::option::Option::Some(alias) = &data.alias {
            format!("{}.{}", "github", alias)
        } else {
            "github".into()
        }
    }

    pub fn set_alias(self, alias: impl std::string::ToString) -> Self {
        self.0.data.borrow_mut().alias = std::option::Option::Some(alias.to_string());
        self
    }

    #[doc = "Set the field `base_url`.\nThe base URL for the GitHub API; this defaults to the GitHub API URL. If you are using GitHub Enterprise Server (GHES) or GitHub Enterprise Cloud with Data Residency (GHEC-DR), this is required. This can also be set by the `GITHUB_BASE_URL` environment variable."]
    pub fn set_base_url(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().base_url = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `cache_path`.\nThe path to the cache directory for persisting GitHub API requests between runs; if not set there will be no caching between runs. This can also be set by the `GITHUB_CACHE_PATH` environment variable."]
    pub fn set_cache_path(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().cache_path = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `insecure`.\nAllow insecure server connections when using SSL."]
    pub fn set_insecure(self, v: impl std::convert::Into<terrars::PrimField<bool>>) -> Self {
        self.0.data.borrow_mut().insecure = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `legacy_client`.\nUse the legacy GitHub client implementation; if set to `false`, the new client implementation is used. This can also be set by the `GITHUB_LEGACY_CLIENT` environment variable."]
    pub fn set_legacy_client(self, v: impl std::convert::Into<terrars::PrimField<bool>>) -> Self {
        self.0.data.borrow_mut().legacy_client = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `max_per_page`.\nThe maximum number of results per page for paginated API requests; this defaults to `100`. This can also be set by the `GITHUB_MAX_PER_PAGE` environment variable."]
    pub fn set_max_per_page(self, v: impl std::convert::Into<terrars::PrimField<f64>>) -> Self {
        self.0.data.borrow_mut().max_per_page = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `max_retries`.\nThe maximum number of retries for failed requests; this defaults to `3`."]
    pub fn set_max_retries(self, v: impl std::convert::Into<terrars::PrimField<f64>>) -> Self {
        self.0.data.borrow_mut().max_retries = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `organization`.\nGitHub organization to manage. This can also be set by the `GITHUB_ORGANIZATION` environment variable."]
    pub fn set_organization(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().organization = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `owner`.\nGitHub organization or user account to manage; this is required when authenticating using a GitHub App. If the owner is not provided and a token is provided, the provider will attempt to auto-detect the owner associated with the token. This can also be set by the `GITHUB_OWNER` environment variable."]
    pub fn set_owner(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().owner = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `parallel_requests`.\nAllow the provider to make parallel API calls; this is experimental and may cause concurrency and rate limiting issues. This is ignored for the REST API when `legacy_client` is `false` since the new client implementation is designed to safely handle parallel requests."]
    pub fn set_parallel_requests(
        self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.0.data.borrow_mut().parallel_requests = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `read_delay_ms`.\nThe delay in milliseconds between read operations; this defaults to `0`. This can be used to mitigate rate limiting issues when performing a large number of read operations. This is ignored for the REST API when `legacy_client` is `false` since the new client implementation is GitHub rate limit aware."]
    pub fn set_read_delay_ms(self, v: impl std::convert::Into<terrars::PrimField<f64>>) -> Self {
        self.0.data.borrow_mut().read_delay_ms = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `retry_delay_ms`.\nThe delay in milliseconds between retry attempts; this defaults to `1000`. This setting only applies when `max_retries` is greater than `0`."]
    pub fn set_retry_delay_ms(self, v: impl std::convert::Into<terrars::PrimField<f64>>) -> Self {
        self.0.data.borrow_mut().retry_delay_ms = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `retryable_errors`.\nList of HTTP status codes that should be retried; if not set this uses the provider defaults. This setting only applies when `max_retries` is greater than `0`. This is ignored for the REST API when `legacy_client` is `false` since the new client implementation handles the retry logic."]
    pub fn set_retryable_errors(
        self,
        v: impl std::convert::Into<terrars::ListField<terrars::PrimField<f64>>>,
    ) -> Self {
        self.0.data.borrow_mut().retryable_errors = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `token`.\nGitHub OAuth or Personal Access Token (PAT) to use for authentication. This can also be set by the `GITHUB_TOKEN` environment variable."]
    pub fn set_token(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().token = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `write_delay_ms`.\nThe delay in milliseconds between write operations; this defaults to `1000`. This is used to mitigate the GitHub API's abuse rate limits when writing. Note that **ALL** requests to the GraphQL API are implemented as `POST` requests under the hood, so this setting affects those calls as well. This is ignored for the REST API when `legacy_client` is `false` since the new client implementation is GitHub rate limit aware."]
    pub fn set_write_delay_ms(self, v: impl std::convert::Into<terrars::PrimField<f64>>) -> Self {
        self.0.data.borrow_mut().write_delay_ms = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::Provider for ProviderGithub_ {
    fn extract_type_tf_id(&self) -> std::string::String {
        "github".into()
    }

    fn extract_provider_type(&self) -> serde_json::Value {
        serde_json::json!({
            "source": "integrations/github",
            "version": "6.13.0",
        })
    }

    fn extract_provider(&self) -> serde_json::Value {
        serde_json::to_value(&self.data).unwrap()
    }
}

pub struct BuildProviderGithub {}

impl BuildProviderGithub {
    pub fn build(self, stack: &mut terrars::Stack) -> ProviderGithub {
        let out = ProviderGithub(std::rc::Rc::new(ProviderGithub_ {
            data: std::cell::RefCell::new(ProviderGithubData_ {
                alias: std::option::Option::None,
                base_url: core::default::Default::default(),
                cache_path: core::default::Default::default(),
                insecure: core::default::Default::default(),
                legacy_client: core::default::Default::default(),
                max_per_page: core::default::Default::default(),
                max_retries: core::default::Default::default(),
                organization: core::default::Default::default(),
                owner: core::default::Default::default(),
                parallel_requests: core::default::Default::default(),
                read_delay_ms: core::default::Default::default(),
                retry_delay_ms: core::default::Default::default(),
                retryable_errors: core::default::Default::default(),
                token: core::default::Default::default(),
                write_delay_ms: core::default::Default::default(),
            }),
        }));
        stack.add_provider(out.0.clone());
        out
    }
}
