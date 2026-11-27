#![allow(non_camel_case_types)]

use serde::Serialize;

#[derive(Serialize)]
struct ProviderAuth0Data_ {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    alias: std::option::Option<std::string::String>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    api_token: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    audience: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    cli_login: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    client_assertion_private_key: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    client_assertion_signing_alg: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    client_id: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    client_secret: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    custom_domain_header: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    debug: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    domain: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    dynamic_credentials: std::option::Option<terrars::PrimField<bool>>,
}

struct ProviderAuth0_ {
    data: std::cell::RefCell<ProviderAuth0Data_>,
}

pub struct ProviderAuth0(std::rc::Rc<ProviderAuth0_>);

impl ProviderAuth0 {
    pub fn provider_ref(&self) -> std::string::String {
        let data = self.0.data.borrow();
        if let std::option::Option::Some(alias) = &data.alias {
            format!("{}.{}", "auth0", alias)
        } else {
            "auth0".into()
        }
    }

    pub fn set_alias(self, alias: impl std::string::ToString) -> Self {
        self.0.data.borrow_mut().alias = std::option::Option::Some(alias.to_string());
        self
    }

    #[doc = "Set the field `api_token`.\nYour Auth0 [management api access token](https://auth0.com/docs/security/tokens/access-tokens/management-api-access-tokens). It can also be sourced from the `AUTH0_API_TOKEN` environment variable. It can be used instead of `client_id` + `client_secret`. If both are specified, `api_token` will be used over `client_id` + `client_secret` fields."]
    pub fn set_api_token(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().api_token = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `audience`.\nYour Auth0 audience when using a custom domain. It can also be sourced from the `AUTH0_AUDIENCE` environment variable."]
    pub fn set_audience(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().audience = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `cli_login`.\nWhile toggled on, the API token gets fetched from the keyring for the given domain"]
    pub fn set_cli_login(self, v: impl std::convert::Into<terrars::PrimField<bool>>) -> Self {
        self.0.data.borrow_mut().cli_login = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `client_assertion_private_key`.\nThe private key used to sign the client assertion JWT. It can also be sourced from the `AUTH0_CLIENT_ASSERTION_PRIVATE_KEY` environment variable. "]
    pub fn set_client_assertion_private_key(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().client_assertion_private_key = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `client_assertion_signing_alg`.\nThe algorithm used to sign the client assertion JWT. It can also be sourced from the `AUTH0_CLIENT_ASSERTION_SIGNING_ALG` environment variable. "]
    pub fn set_client_assertion_signing_alg(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().client_assertion_signing_alg = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `client_id`.\nYour Auth0 client ID. It can also be sourced from the `AUTH0_CLIENT_ID` environment variable."]
    pub fn set_client_id(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().client_id = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `client_secret`.\nYour Auth0 client secret. It can also be sourced from the `AUTH0_CLIENT_SECRET` environment variable."]
    pub fn set_client_secret(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().client_secret = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `custom_domain_header`.\nWhen specified, this header is added to requests targeting a set of pre-defined whitelisted URLs Global setting overrides all resource specific `custom_domain_header` value"]
    pub fn set_custom_domain_header(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().custom_domain_header = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `debug`.\nEnables HTTP request and response logging when TF_LOG=DEBUG is set. It can also be sourced from the `AUTH0_DEBUG` environment variable."]
    pub fn set_debug(self, v: impl std::convert::Into<terrars::PrimField<bool>>) -> Self {
        self.0.data.borrow_mut().debug = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `domain`.\nYour Auth0 domain name. It can also be sourced from the `AUTH0_DOMAIN` environment variable."]
    pub fn set_domain(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().domain = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `dynamic_credentials`.\nIndicates whether credentials will be dynamically passed to the provider from other terraform resources."]
    pub fn set_dynamic_credentials(
        self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.0.data.borrow_mut().dynamic_credentials = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::Provider for ProviderAuth0_ {
    fn extract_type_tf_id(&self) -> std::string::String {
        "auth0".into()
    }

    fn extract_provider_type(&self) -> serde_json::Value {
        serde_json::json!({
            "source": "auth0/auth0",
            "version": "1.63.0",
        })
    }

    fn extract_provider(&self) -> serde_json::Value {
        serde_json::to_value(&self.data).unwrap()
    }
}

pub struct BuildProviderAuth0 {}

impl BuildProviderAuth0 {
    pub fn build(self, stack: &mut terrars::Stack) -> ProviderAuth0 {
        let out = ProviderAuth0(std::rc::Rc::new(ProviderAuth0_ {
            data: std::cell::RefCell::new(ProviderAuth0Data_ {
                alias: std::option::Option::None,
                api_token: core::default::Default::default(),
                audience: core::default::Default::default(),
                cli_login: core::default::Default::default(),
                client_assertion_private_key: core::default::Default::default(),
                client_assertion_signing_alg: core::default::Default::default(),
                client_id: core::default::Default::default(),
                client_secret: core::default::Default::default(),
                custom_domain_header: core::default::Default::default(),
                debug: core::default::Default::default(),
                domain: core::default::Default::default(),
                dynamic_credentials: core::default::Default::default(),
            }),
        }));
        stack.add_provider(out.0.clone());
        out
    }
}
