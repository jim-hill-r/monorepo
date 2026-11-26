#![allow(non_camel_case_types)]

use super::provider::ProviderAuth0;
use serde::Serialize;
use terrars::{Ref as _, Referable as _, Resource_ as _};

#[derive(Serialize)]
struct ClientData_ {
    #[serde(skip_serializing_if = "std::vec::Vec::is_empty")]
    depends_on: std::vec::Vec<std::string::String>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    provider: std::option::Option<std::string::String>,
    #[serde(skip_serializing_if = "terrars::SerdeSkipDefault::is_default")]
    lifecycle: terrars::ResourceLifecycle,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    for_each: std::option::Option<std::string::String>,
    #[serde(skip_serializing_if = "std::vec::Vec::is_empty")]
    provisioner: std::vec::Vec<terrars::Provisioner>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    agent_id: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    allowed_clients:
        std::option::Option<terrars::ListField<terrars::PrimField<std::string::String>>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    allowed_logout_urls:
        std::option::Option<terrars::ListField<terrars::PrimField<std::string::String>>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    allowed_origins:
        std::option::Option<terrars::ListField<terrars::PrimField<std::string::String>>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    app_type: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    async_approval_notification_channels:
        std::option::Option<terrars::ListField<terrars::PrimField<std::string::String>>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    callbacks: std::option::Option<terrars::ListField<terrars::PrimField<std::string::String>>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    client_aliases:
        std::option::Option<terrars::ListField<terrars::PrimField<std::string::String>>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    client_id: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    client_metadata:
        std::option::Option<terrars::RecField<terrars::PrimField<std::string::String>>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    compliance_level: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    cross_origin_auth: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    cross_origin_loc: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    custom_login_page: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    custom_login_page_on: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    description: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    encryption_key: std::option::Option<terrars::RecField<terrars::PrimField<std::string::String>>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    form_template: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    grant_types: std::option::Option<terrars::ListField<terrars::PrimField<std::string::String>>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    id: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    initiate_login_uri: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    is_first_party: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    is_token_endpoint_ip_header_trusted: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    logo_uri: std::option::Option<terrars::PrimField<std::string::String>>,
    name: terrars::PrimField<std::string::String>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    oidc_backchannel_logout_urls:
        std::option::Option<terrars::SetField<terrars::PrimField<std::string::String>>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    oidc_conformant: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    organization_discovery_methods:
        std::option::Option<terrars::ListField<terrars::PrimField<std::string::String>>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    organization_require_behavior: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    organization_usage: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    redirection_policy: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    require_proof_of_possession: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    require_pushed_authorization_requests: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    resource_server_identifier: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    skip_non_verifiable_callback_uri_confirmation_prompt:
        std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    sso: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    sso_disabled: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    third_party_security_mode: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    web_origins: std::option::Option<terrars::ListField<terrars::PrimField<std::string::String>>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    addons: std::option::Option<std::vec::Vec<Client_Addons>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    anonymous_sessions: std::option::Option<std::vec::Vec<Client_AnonymousSessions>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    b2b_integration_configuration:
        std::option::Option<std::vec::Vec<Client_B2bIntegrationConfiguration>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    default_organization: std::option::Option<std::vec::Vec<Client_DefaultOrganization>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    express_configuration: std::option::Option<std::vec::Vec<Client_ExpressConfiguration>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    fedcm_login: std::option::Option<std::vec::Vec<Client_FedcmLogin>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    identity_assertion_authorization_grant:
        std::option::Option<std::vec::Vec<Client_IdentityAssertionAuthorizationGrant>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    jwt_configuration: std::option::Option<std::vec::Vec<Client_JwtConfiguration>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    mobile: std::option::Option<std::vec::Vec<Client_Mobile>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    my_organization_configuration:
        std::option::Option<std::vec::Vec<Client_MyOrganizationConfiguration>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    native_social_login: std::option::Option<std::vec::Vec<Client_NativeSocialLogin>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    oidc_logout: std::option::Option<std::vec::Vec<Client_OidcLogout>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    oidc_support: std::option::Option<std::vec::Vec<Client_OidcSupport>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    refresh_token: std::option::Option<std::vec::Vec<Client_RefreshToken>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    session_transfer: std::option::Option<std::vec::Vec<Client_SessionTransfer>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    token_exchange: std::option::Option<std::vec::Vec<Client_TokenExchange>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    token_quota: std::option::Option<std::vec::Vec<Client_TokenQuota>>,
    dynamic: ClientDynamic,
}

struct Client_ {
    shared: terrars::StackShared,
    tf_id: std::string::String,
    data: std::cell::RefCell<ClientData_>,
}

#[derive(Clone)]
pub struct Client(std::rc::Rc<Client_>);

impl Client {
    fn shared(&self) -> &terrars::StackShared {
        &self.0.shared
    }

    pub fn depends_on(self, dep: &impl terrars::Referable) -> Self {
        self.0.data.borrow_mut().depends_on.push(dep.extract_ref());
        self
    }

    pub fn set_provider(self, provider: &ProviderAuth0) -> Self {
        self.0.data.borrow_mut().provider = std::option::Option::Some(provider.provider_ref());
        self
    }

    pub fn set_create_before_destroy(self, v: bool) -> Self {
        self.0.data.borrow_mut().lifecycle.create_before_destroy = v;
        self
    }

    pub fn set_prevent_destroy(self, v: bool) -> Self {
        self.0.data.borrow_mut().lifecycle.prevent_destroy = v;
        self
    }

    pub fn ignore_changes_to_all(self) -> Self {
        self.0.data.borrow_mut().lifecycle.ignore_changes =
            std::option::Option::Some(terrars::IgnoreChanges::All(terrars::IgnoreChangesAll::All));
        self
    }

    pub fn ignore_changes_to_attr(self, attr: impl std::string::ToString) -> Self {
        {
            let mut d = self.0.data.borrow_mut();
            if match &mut d.lifecycle.ignore_changes {
                std::option::Option::Some(i) => match i {
                    terrars::IgnoreChanges::All(_) => true,
                    terrars::IgnoreChanges::Refs(r) => {
                        r.push(attr.to_string());
                        false
                    }
                },
                std::option::Option::None => true,
            } {
                d.lifecycle.ignore_changes =
                    std::option::Option::Some(terrars::IgnoreChanges::Refs(vec![attr.to_string()]));
            }
        }
        self
    }

    pub fn replace_triggered_by_resource(self, r: &impl terrars::Resource) -> Self {
        self.0
            .data
            .borrow_mut()
            .lifecycle
            .replace_triggered_by
            .push(r.extract_ref());
        self
    }

    pub fn replace_triggered_by_attr(self, attr: impl std::string::ToString) -> Self {
        self.0
            .data
            .borrow_mut()
            .lifecycle
            .replace_triggered_by
            .push(attr.to_string());
        self
    }

    #[doc = r" Add a provisioner (ex: `local-exec`, `remote-exec`, `file`) to this resource."]
    #[doc = r" Provisioners run in the order they're added."]
    pub fn add_provisioner(self, v: impl std::convert::Into<terrars::Provisioner>) -> Self {
        self.0.data.borrow_mut().provisioner.push(v.into());
        self
    }

    pub fn add_lifecycle_precondition(
        self,
        condition: impl std::convert::Into<terrars::PrimField<bool>>,
        error_message: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0
            .data
            .borrow_mut()
            .lifecycle
            .precondition
            .push(terrars::LifecycleCondition {
                condition: condition.into(),
                error_message: error_message.into(),
            });
        self
    }

    pub fn add_lifecycle_postcondition(
        self,
        condition: impl std::convert::Into<terrars::PrimField<bool>>,
        error_message: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0
            .data
            .borrow_mut()
            .lifecycle
            .postcondition
            .push(terrars::LifecycleCondition {
                condition: condition.into(),
                error_message: error_message.into(),
            });
        self
    }

    #[doc = "Set the field `agent_id`.\nThe ID of the agent (`auth0_agent`) this client is linked to (EA Only). Reference it as `auth0_agent.<name>.id` so that the client is unlinked before the agent is deleted. Removing the attribute unlinks the client, including a link made outside Terraform. Requires the `update:clients` and `update:agents` scopes."]
    pub fn set_agent_id(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().agent_id = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `allowed_clients`.\nList of applications ID's that will be allowed to make delegation request. By default, all applications will be allowed."]
    pub fn set_allowed_clients(
        self,
        v: impl std::convert::Into<terrars::ListField<terrars::PrimField<std::string::String>>>,
    ) -> Self {
        self.0.data.borrow_mut().allowed_clients = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `allowed_logout_urls`.\nURLs that Auth0 may redirect to after logout."]
    pub fn set_allowed_logout_urls(
        self,
        v: impl std::convert::Into<terrars::ListField<terrars::PrimField<std::string::String>>>,
    ) -> Self {
        self.0.data.borrow_mut().allowed_logout_urls = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `allowed_origins`.\nURLs that represent valid origins for cross-origin resource sharing. By default, all your callback URLs will be allowed."]
    pub fn set_allowed_origins(
        self,
        v: impl std::convert::Into<terrars::ListField<terrars::PrimField<std::string::String>>>,
    ) -> Self {
        self.0.data.borrow_mut().allowed_origins = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `app_type`.\nType of application the client represents. Possible values are: `native`, `spa`, `regular_web`, `non_interactive`, `resource_server`,`sso_integration`. Specific SSO integrations types accepted as well are: `rms`, `box`, `cloudbees`, `concur`, `dropbox`, `mscrm`, `echosign`, `egnyte`, `newrelic`, `office365`, `salesforce`, `sentry`, `sharepoint`, `slack`, `springcm`, `zendesk`, `zoom`, `express_configuration`"]
    pub fn set_app_type(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().app_type = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `async_approval_notification_channels`.\nList of notification channels enabled for CIBA (Client-Initiated Backchannel Authentication) requests initiated by this client. Valid values are `guardian-push`, `email`, and `my-account`.The order is significant as this is the order in which notification channels will be evaluated."]
    pub fn set_async_approval_notification_channels(
        self,
        v: impl std::convert::Into<terrars::ListField<terrars::PrimField<std::string::String>>>,
    ) -> Self {
        self.0
            .data
            .borrow_mut()
            .async_approval_notification_channels = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `callbacks`.\nURLs that Auth0 may call back to after a user authenticates for the client. Make sure to specify the protocol (https://) otherwise the callback may fail in some cases. With the exception of custom URI schemes for native clients, all callbacks should use protocol https://."]
    pub fn set_callbacks(
        self,
        v: impl std::convert::Into<terrars::ListField<terrars::PrimField<std::string::String>>>,
    ) -> Self {
        self.0.data.borrow_mut().callbacks = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `client_aliases`.\nList of audiences/realms for SAML protocol. Used by the wsfed addon."]
    pub fn set_client_aliases(
        self,
        v: impl std::convert::Into<terrars::ListField<terrars::PrimField<std::string::String>>>,
    ) -> Self {
        self.0.data.borrow_mut().client_aliases = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `client_id`.\nThe ID of the client. If not provided, Auth0 will generate one automatically. Use this to specify a custom client ID for migration or tenant-copy scenarios. Requires feature flag to be enabled on the tenant."]
    pub fn set_client_id(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().client_id = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `client_metadata`.\nMetadata associated with the client, in the form of an object with string values (max 255 chars). Maximum of 10 metadata properties allowed. Field names (max 255 chars) are alphanumeric and may only include the following special characters: `:,-+=_*?\"/\\()<>@ [Tab] [Space]`."]
    pub fn set_client_metadata(
        self,
        v: impl std::convert::Into<terrars::RecField<terrars::PrimField<std::string::String>>>,
    ) -> Self {
        self.0.data.borrow_mut().client_metadata = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `compliance_level`.\nDefines the compliance level for this client, which may restrict it's capabilities. Can be one of `none`, `fapi1_adv_pkj_par`, `fapi1_adv_mtls_par`."]
    pub fn set_compliance_level(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().compliance_level = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `cross_origin_auth`.\nWhether this client can be used to make cross-origin authentication requests (`true`) or it is not allowed to make such requests (`false`)."]
    pub fn set_cross_origin_auth(
        self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.0.data.borrow_mut().cross_origin_auth = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `cross_origin_loc`.\nURL of the location in your site where the cross-origin verification takes place for the cross-origin auth flow when performing authentication in your own domain instead of Auth0 Universal Login page."]
    pub fn set_cross_origin_loc(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().cross_origin_loc = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `custom_login_page`.\nThe content (HTML, CSS, JS) of the custom login page."]
    pub fn set_custom_login_page(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().custom_login_page = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `custom_login_page_on`.\nIndicates whether a custom login page is to be used."]
    pub fn set_custom_login_page_on(
        self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.0.data.borrow_mut().custom_login_page_on = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `description`.\nDescription of the purpose of the client."]
    pub fn set_description(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().description = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `encryption_key`.\nEncryption used for WS-Fed responses with this client."]
    pub fn set_encryption_key(
        self,
        v: impl std::convert::Into<terrars::RecField<terrars::PrimField<std::string::String>>>,
    ) -> Self {
        self.0.data.borrow_mut().encryption_key = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `form_template`.\nHTML form template to be used for WS-Federation."]
    pub fn set_form_template(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().form_template = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `grant_types`.\nTypes of grants that this client is authorized to use."]
    pub fn set_grant_types(
        self,
        v: impl std::convert::Into<terrars::ListField<terrars::PrimField<std::string::String>>>,
    ) -> Self {
        self.0.data.borrow_mut().grant_types = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `id`.\n"]
    pub fn set_id(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().id = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `initiate_login_uri`.\nInitiate login URI. Must be HTTPS or an empty string. May contain Auth0 dynamic login URI placeholders such as `{organization.metadata.public_login_host}` or `{custom_domain.metadata.public_app_host}`, which are resolved by Auth0 at request time. See https://auth0.com/docs/get-started/applications/application-settings."]
    pub fn set_initiate_login_uri(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().initiate_login_uri = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `is_first_party`.\nIndicates whether this client is a first-party client."]
    pub fn set_is_first_party(self, v: impl std::convert::Into<terrars::PrimField<bool>>) -> Self {
        self.0.data.borrow_mut().is_first_party = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `is_token_endpoint_ip_header_trusted`.\nIndicates whether the token endpoint IP header is trusted. Requires the authentication method to be set to `client_secret_post` or `client_secret_basic`. Setting this property when creating the resource, will default the authentication method to `client_secret_post`. To change the authentication method to `client_secret_basic` use the `auth0_client_credentials` resource."]
    pub fn set_is_token_endpoint_ip_header_trusted(
        self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.0.data.borrow_mut().is_token_endpoint_ip_header_trusted =
            std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `logo_uri`.\nURL of the logo for the client. Recommended size is 150px x 150px. If none is set, the default badge for the application type will be shown."]
    pub fn set_logo_uri(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().logo_uri = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `oidc_backchannel_logout_urls`.\nSet of URLs that are valid to call back from Auth0 for OIDC backchannel logout. Currently only one URL is allowed."]
    pub fn set_oidc_backchannel_logout_urls(
        self,
        v: impl std::convert::Into<terrars::SetField<terrars::PrimField<std::string::String>>>,
    ) -> Self {
        self.0.data.borrow_mut().oidc_backchannel_logout_urls = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `oidc_conformant`.\nIndicates whether this client will conform to strict OIDC specifications."]
    pub fn set_oidc_conformant(self, v: impl std::convert::Into<terrars::PrimField<bool>>) -> Self {
        self.0.data.borrow_mut().oidc_conformant = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `organization_discovery_methods`.\nMethods for discovering organizations during the pre_login_prompt. Can include `email` (allows users to find their organization by entering their email address) and/or `organization_name` (requires users to enter the organization name directly). These methods can be combined. Setting this property requires that `organization_require_behavior` is set to `pre_login_prompt`. For clients that set `b2b_integration_configuration`, server-side defaults the values when this is not specified; Set to `[]` (empty array) to clear the values."]
    pub fn set_organization_discovery_methods(
        self,
        v: impl std::convert::Into<terrars::ListField<terrars::PrimField<std::string::String>>>,
    ) -> Self {
        self.0.data.borrow_mut().organization_discovery_methods =
            std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `organization_require_behavior`.\nDefines how to proceed during an authentication transaction when `organization_usage = \"require\"`. Can be `no_prompt` (default), `pre_login_prompt` or  `post_login_prompt`."]
    pub fn set_organization_require_behavior(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().organization_require_behavior =
            std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `organization_usage`.\nDefines how to proceed during an authentication transaction with regards to an organization. Can be `deny` (default), `allow` or `require`."]
    pub fn set_organization_usage(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().organization_usage = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `redirection_policy`.\nControls whether Auth0 redirects users to the application's callback URL on authentication errors or in email verification flows.Allowed values: `allow_always` or `open_redirect_protection`."]
    pub fn set_redirection_policy(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().redirection_policy = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `require_proof_of_possession`.\nMakes the use of Proof-of-Possession mandatory for this client."]
    pub fn set_require_proof_of_possession(
        self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.0.data.borrow_mut().require_proof_of_possession = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `require_pushed_authorization_requests`.\nMakes the use of Pushed Authorization Requests mandatory for this client. This feature currently needs to be enabled on the tenant in order to make use of it."]
    pub fn set_require_pushed_authorization_requests(
        self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.0
            .data
            .borrow_mut()
            .require_pushed_authorization_requests = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `resource_server_identifier`.\nThe identifier of a resource server that client is associated withThis property can be sent only when app_type=resource_server.This property can not be changed, once the client is created."]
    pub fn set_resource_server_identifier(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().resource_server_identifier = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `skip_non_verifiable_callback_uri_confirmation_prompt`.\nIndicates whether the confirmation prompt appears when using non-verifiable callback URIs. Set to true to skip the prompt, false to show it, or null to unset. Accepts (true/false/null) or (\"true\"/\"false\"/\"null\") "]
    pub fn set_skip_non_verifiable_callback_uri_confirmation_prompt(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0
            .data
            .borrow_mut()
            .skip_non_verifiable_callback_uri_confirmation_prompt =
            std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `sso`.\nApplies only to SSO clients and determines whether Auth0 will handle Single Sign-On (true) or whether the identity provider will (false)."]
    pub fn set_sso(self, v: impl std::convert::Into<terrars::PrimField<bool>>) -> Self {
        self.0.data.borrow_mut().sso = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `sso_disabled`.\nIndicates whether or not SSO is disabled."]
    pub fn set_sso_disabled(self, v: impl std::convert::Into<terrars::PrimField<bool>>) -> Self {
        self.0.data.borrow_mut().sso_disabled = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `third_party_security_mode`.\nSecurity mode for third-party clients. Allowed values: `strict` or `permissive`."]
    pub fn set_third_party_security_mode(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().third_party_security_mode = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `web_origins`.\nURLs that represent valid web origins for use with web message response mode."]
    pub fn set_web_origins(
        self,
        v: impl std::convert::Into<terrars::ListField<terrars::PrimField<std::string::String>>>,
    ) -> Self {
        self.0.data.borrow_mut().web_origins = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `addons`.\n"]
    pub fn set_addons(
        self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_Addons>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.0.data.borrow_mut().addons = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.0.data.borrow_mut().dynamic.addons = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `anonymous_sessions`.\n"]
    pub fn set_anonymous_sessions(
        self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_AnonymousSessions>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.0.data.borrow_mut().anonymous_sessions = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.0.data.borrow_mut().dynamic.anonymous_sessions = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `b2b_integration_configuration`.\n"]
    pub fn set_b2b_integration_configuration(
        self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_B2bIntegrationConfiguration>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.0.data.borrow_mut().b2b_integration_configuration =
                    std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.0
                    .data
                    .borrow_mut()
                    .dynamic
                    .b2b_integration_configuration = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `default_organization`.\n"]
    pub fn set_default_organization(
        self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_DefaultOrganization>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.0.data.borrow_mut().default_organization = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.0.data.borrow_mut().dynamic.default_organization =
                    std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `express_configuration`.\n"]
    pub fn set_express_configuration(
        self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_ExpressConfiguration>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.0.data.borrow_mut().express_configuration = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.0.data.borrow_mut().dynamic.express_configuration =
                    std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `fedcm_login`.\n"]
    pub fn set_fedcm_login(
        self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_FedcmLogin>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.0.data.borrow_mut().fedcm_login = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.0.data.borrow_mut().dynamic.fedcm_login = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `identity_assertion_authorization_grant`.\n"]
    pub fn set_identity_assertion_authorization_grant(
        self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_IdentityAssertionAuthorizationGrant>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.0
                    .data
                    .borrow_mut()
                    .identity_assertion_authorization_grant = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.0
                    .data
                    .borrow_mut()
                    .dynamic
                    .identity_assertion_authorization_grant = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `jwt_configuration`.\n"]
    pub fn set_jwt_configuration(
        self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_JwtConfiguration>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.0.data.borrow_mut().jwt_configuration = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.0.data.borrow_mut().dynamic.jwt_configuration = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `mobile`.\n"]
    pub fn set_mobile(
        self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_Mobile>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.0.data.borrow_mut().mobile = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.0.data.borrow_mut().dynamic.mobile = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `my_organization_configuration`.\n"]
    pub fn set_my_organization_configuration(
        self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_MyOrganizationConfiguration>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.0.data.borrow_mut().my_organization_configuration =
                    std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.0
                    .data
                    .borrow_mut()
                    .dynamic
                    .my_organization_configuration = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `native_social_login`.\n"]
    pub fn set_native_social_login(
        self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_NativeSocialLogin>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.0.data.borrow_mut().native_social_login = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.0.data.borrow_mut().dynamic.native_social_login = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `oidc_logout`.\n"]
    pub fn set_oidc_logout(
        self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_OidcLogout>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.0.data.borrow_mut().oidc_logout = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.0.data.borrow_mut().dynamic.oidc_logout = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `oidc_support`.\n"]
    pub fn set_oidc_support(
        self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_OidcSupport>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.0.data.borrow_mut().oidc_support = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.0.data.borrow_mut().dynamic.oidc_support = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `refresh_token`.\n"]
    pub fn set_refresh_token(
        self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_RefreshToken>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.0.data.borrow_mut().refresh_token = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.0.data.borrow_mut().dynamic.refresh_token = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `session_transfer`.\n"]
    pub fn set_session_transfer(
        self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_SessionTransfer>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.0.data.borrow_mut().session_transfer = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.0.data.borrow_mut().dynamic.session_transfer = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `token_exchange`.\n"]
    pub fn set_token_exchange(
        self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_TokenExchange>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.0.data.borrow_mut().token_exchange = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.0.data.borrow_mut().dynamic.token_exchange = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `token_quota`.\n"]
    pub fn set_token_quota(
        self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_TokenQuota>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.0.data.borrow_mut().token_quota = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.0.data.borrow_mut().dynamic.token_quota = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Get a reference to the value of field `agent_id` after provisioning.\nThe ID of the agent (`auth0_agent`) this client is linked to (EA Only). Reference it as `auth0_agent.<name>.id` so that the client is unlinked before the agent is deleted. Removing the attribute unlinks the client, including a link made outside Terraform. Requires the `update:clients` and `update:agents` scopes."]
    pub fn agent_id(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.agent_id", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `allowed_clients` after provisioning.\nList of applications ID's that will be allowed to make delegation request. By default, all applications will be allowed."]
    pub fn allowed_clients(&self) -> terrars::ListRef<terrars::PrimExpr<std::string::String>> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.allowed_clients", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `allowed_logout_urls` after provisioning.\nURLs that Auth0 may redirect to after logout."]
    pub fn allowed_logout_urls(&self) -> terrars::ListRef<terrars::PrimExpr<std::string::String>> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.allowed_logout_urls", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `allowed_origins` after provisioning.\nURLs that represent valid origins for cross-origin resource sharing. By default, all your callback URLs will be allowed."]
    pub fn allowed_origins(&self) -> terrars::ListRef<terrars::PrimExpr<std::string::String>> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.allowed_origins", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `app_type` after provisioning.\nType of application the client represents. Possible values are: `native`, `spa`, `regular_web`, `non_interactive`, `resource_server`,`sso_integration`. Specific SSO integrations types accepted as well are: `rms`, `box`, `cloudbees`, `concur`, `dropbox`, `mscrm`, `echosign`, `egnyte`, `newrelic`, `office365`, `salesforce`, `sentry`, `sharepoint`, `slack`, `springcm`, `zendesk`, `zoom`, `express_configuration`"]
    pub fn app_type(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.app_type", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `async_approval_notification_channels` after provisioning.\nList of notification channels enabled for CIBA (Client-Initiated Backchannel Authentication) requests initiated by this client. Valid values are `guardian-push`, `email`, and `my-account`.The order is significant as this is the order in which notification channels will be evaluated."]
    pub fn async_approval_notification_channels(
        &self,
    ) -> terrars::ListRef<terrars::PrimExpr<std::string::String>> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!(
                "{}.async_approval_notification_channels",
                self.extract_ref()
            ),
        )
    }

    #[doc = "Get a reference to the value of field `callbacks` after provisioning.\nURLs that Auth0 may call back to after a user authenticates for the client. Make sure to specify the protocol (https://) otherwise the callback may fail in some cases. With the exception of custom URI schemes for native clients, all callbacks should use protocol https://."]
    pub fn callbacks(&self) -> terrars::ListRef<terrars::PrimExpr<std::string::String>> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.callbacks", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `client_aliases` after provisioning.\nList of audiences/realms for SAML protocol. Used by the wsfed addon."]
    pub fn client_aliases(&self) -> terrars::ListRef<terrars::PrimExpr<std::string::String>> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.client_aliases", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `client_id` after provisioning.\nThe ID of the client. If not provided, Auth0 will generate one automatically. Use this to specify a custom client ID for migration or tenant-copy scenarios. Requires feature flag to be enabled on the tenant."]
    pub fn client_id(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.client_id", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `client_metadata` after provisioning.\nMetadata associated with the client, in the form of an object with string values (max 255 chars). Maximum of 10 metadata properties allowed. Field names (max 255 chars) are alphanumeric and may only include the following special characters: `:,-+=_*?\"/\\()<>@ [Tab] [Space]`."]
    pub fn client_metadata(&self) -> terrars::RecRef<terrars::PrimExpr<std::string::String>> {
        terrars::RecRef::new(
            self.shared().clone(),
            format!("{}.client_metadata", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `compliance_level` after provisioning.\nDefines the compliance level for this client, which may restrict it's capabilities. Can be one of `none`, `fapi1_adv_pkj_par`, `fapi1_adv_mtls_par`."]
    pub fn compliance_level(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.compliance_level", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `cross_origin_auth` after provisioning.\nWhether this client can be used to make cross-origin authentication requests (`true`) or it is not allowed to make such requests (`false`)."]
    pub fn cross_origin_auth(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.cross_origin_auth", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `cross_origin_loc` after provisioning.\nURL of the location in your site where the cross-origin verification takes place for the cross-origin auth flow when performing authentication in your own domain instead of Auth0 Universal Login page."]
    pub fn cross_origin_loc(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.cross_origin_loc", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `custom_login_page` after provisioning.\nThe content (HTML, CSS, JS) of the custom login page."]
    pub fn custom_login_page(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.custom_login_page", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `custom_login_page_on` after provisioning.\nIndicates whether a custom login page is to be used."]
    pub fn custom_login_page_on(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.custom_login_page_on", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `description` after provisioning.\nDescription of the purpose of the client."]
    pub fn description(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.description", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `encryption_key` after provisioning.\nEncryption used for WS-Fed responses with this client."]
    pub fn encryption_key(&self) -> terrars::RecRef<terrars::PrimExpr<std::string::String>> {
        terrars::RecRef::new(
            self.shared().clone(),
            format!("{}.encryption_key", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `external_client_id` after provisioning.\nThe URL of the Client ID Metadata Document. Only present for CIMD-registered clients."]
    pub fn external_client_id(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.external_client_id", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `external_metadata_created_by` after provisioning.\nWho created the external metadata client: `admin` (via Management API), `client` (self-registered), or `unknown`."]
    pub fn external_metadata_created_by(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.external_metadata_created_by", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `external_metadata_status` after provisioning.\nWhether Auth0 can resolve the Client ID Metadata Document from its cache or the document URL: `available` or `unavailable` (EA only). Only present for CIMD-registered clients."]
    pub fn external_metadata_status(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.external_metadata_status", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `external_metadata_type` after provisioning.\nType of external metadata. Value is `cimd` for CIMD-registered clients."]
    pub fn external_metadata_type(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.external_metadata_type", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `form_template` after provisioning.\nHTML form template to be used for WS-Federation."]
    pub fn form_template(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.form_template", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `grant_types` after provisioning.\nTypes of grants that this client is authorized to use."]
    pub fn grant_types(&self) -> terrars::ListRef<terrars::PrimExpr<std::string::String>> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.grant_types", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `id` after provisioning.\n"]
    pub fn id(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.id", self.extract_ref()))
    }

    #[doc = "Get a reference to the value of field `initiate_login_uri` after provisioning.\nInitiate login URI. Must be HTTPS or an empty string. May contain Auth0 dynamic login URI placeholders such as `{organization.metadata.public_login_host}` or `{custom_domain.metadata.public_app_host}`, which are resolved by Auth0 at request time. See https://auth0.com/docs/get-started/applications/application-settings."]
    pub fn initiate_login_uri(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.initiate_login_uri", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `is_first_party` after provisioning.\nIndicates whether this client is a first-party client."]
    pub fn is_first_party(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.is_first_party", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `is_token_endpoint_ip_header_trusted` after provisioning.\nIndicates whether the token endpoint IP header is trusted. Requires the authentication method to be set to `client_secret_post` or `client_secret_basic`. Setting this property when creating the resource, will default the authentication method to `client_secret_post`. To change the authentication method to `client_secret_basic` use the `auth0_client_credentials` resource."]
    pub fn is_token_endpoint_ip_header_trusted(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.is_token_endpoint_ip_header_trusted", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `jwks_uri` after provisioning.\nURL for the JSON Web Key Set (JWKS) containing the public keys used for `private_key_jwt` authentication. Only present for CIMD clients using `private_key_jwt` authentication."]
    pub fn jwks_uri(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.jwks_uri", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `logo_uri` after provisioning.\nURL of the logo for the client. Recommended size is 150px x 150px. If none is set, the default badge for the application type will be shown."]
    pub fn logo_uri(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.logo_uri", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `name` after provisioning.\nName of the client."]
    pub fn name(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.name", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `oidc_backchannel_logout_urls` after provisioning.\nSet of URLs that are valid to call back from Auth0 for OIDC backchannel logout. Currently only one URL is allowed."]
    pub fn oidc_backchannel_logout_urls(
        &self,
    ) -> terrars::SetRef<terrars::PrimExpr<std::string::String>> {
        terrars::SetRef::new(
            self.shared().clone(),
            format!("{}.oidc_backchannel_logout_urls", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `oidc_conformant` after provisioning.\nIndicates whether this client will conform to strict OIDC specifications."]
    pub fn oidc_conformant(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.oidc_conformant", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `organization_discovery_methods` after provisioning.\nMethods for discovering organizations during the pre_login_prompt. Can include `email` (allows users to find their organization by entering their email address) and/or `organization_name` (requires users to enter the organization name directly). These methods can be combined. Setting this property requires that `organization_require_behavior` is set to `pre_login_prompt`. For clients that set `b2b_integration_configuration`, server-side defaults the values when this is not specified; Set to `[]` (empty array) to clear the values."]
    pub fn organization_discovery_methods(
        &self,
    ) -> terrars::ListRef<terrars::PrimExpr<std::string::String>> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.organization_discovery_methods", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `organization_require_behavior` after provisioning.\nDefines how to proceed during an authentication transaction when `organization_usage = \"require\"`. Can be `no_prompt` (default), `pre_login_prompt` or  `post_login_prompt`."]
    pub fn organization_require_behavior(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.organization_require_behavior", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `organization_usage` after provisioning.\nDefines how to proceed during an authentication transaction with regards to an organization. Can be `deny` (default), `allow` or `require`."]
    pub fn organization_usage(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.organization_usage", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `redirection_policy` after provisioning.\nControls whether Auth0 redirects users to the application's callback URL on authentication errors or in email verification flows.Allowed values: `allow_always` or `open_redirect_protection`."]
    pub fn redirection_policy(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.redirection_policy", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `require_proof_of_possession` after provisioning.\nMakes the use of Proof-of-Possession mandatory for this client."]
    pub fn require_proof_of_possession(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.require_proof_of_possession", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `require_pushed_authorization_requests` after provisioning.\nMakes the use of Pushed Authorization Requests mandatory for this client. This feature currently needs to be enabled on the tenant in order to make use of it."]
    pub fn require_pushed_authorization_requests(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!(
                "{}.require_pushed_authorization_requests",
                self.extract_ref()
            ),
        )
    }

    #[doc = "Get a reference to the value of field `resource_server_identifier` after provisioning.\nThe identifier of a resource server that client is associated withThis property can be sent only when app_type=resource_server.This property can not be changed, once the client is created."]
    pub fn resource_server_identifier(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.resource_server_identifier", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `signing_keys` after provisioning.\nList containing a map of the public cert of the signing key and the public cert of the signing key in PKCS7."]
    pub fn signing_keys(
        &self,
    ) -> terrars::ListRef<terrars::RecRef<terrars::PrimExpr<std::string::String>>> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.signing_keys", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `skip_non_verifiable_callback_uri_confirmation_prompt` after provisioning.\nIndicates whether the confirmation prompt appears when using non-verifiable callback URIs. Set to true to skip the prompt, false to show it, or null to unset. Accepts (true/false/null) or (\"true\"/\"false\"/\"null\") "]
    pub fn skip_non_verifiable_callback_uri_confirmation_prompt(
        &self,
    ) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!(
                "{}.skip_non_verifiable_callback_uri_confirmation_prompt",
                self.extract_ref()
            ),
        )
    }

    #[doc = "Get a reference to the value of field `sso` after provisioning.\nApplies only to SSO clients and determines whether Auth0 will handle Single Sign-On (true) or whether the identity provider will (false)."]
    pub fn sso(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.sso", self.extract_ref()))
    }

    #[doc = "Get a reference to the value of field `sso_disabled` after provisioning.\nIndicates whether or not SSO is disabled."]
    pub fn sso_disabled(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.sso_disabled", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `third_party_security_mode` after provisioning.\nSecurity mode for third-party clients. Allowed values: `strict` or `permissive`."]
    pub fn third_party_security_mode(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.third_party_security_mode", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `web_origins` after provisioning.\nURLs that represent valid web origins for use with web message response mode."]
    pub fn web_origins(&self) -> terrars::ListRef<terrars::PrimExpr<std::string::String>> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.web_origins", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `addons` after provisioning.\n"]
    pub fn addons(&self) -> terrars::ListRef<Client_AddonsRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.addons", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `anonymous_sessions` after provisioning.\n"]
    pub fn anonymous_sessions(&self) -> terrars::ListRef<Client_AnonymousSessionsRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.anonymous_sessions", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `b2b_integration_configuration` after provisioning.\n"]
    pub fn b2b_integration_configuration(
        &self,
    ) -> terrars::ListRef<Client_B2bIntegrationConfigurationRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.b2b_integration_configuration", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `default_organization` after provisioning.\n"]
    pub fn default_organization(&self) -> terrars::ListRef<Client_DefaultOrganizationRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.default_organization", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `express_configuration` after provisioning.\n"]
    pub fn express_configuration(&self) -> terrars::ListRef<Client_ExpressConfigurationRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.express_configuration", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `fedcm_login` after provisioning.\n"]
    pub fn fedcm_login(&self) -> terrars::ListRef<Client_FedcmLoginRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.fedcm_login", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `identity_assertion_authorization_grant` after provisioning.\n"]
    pub fn identity_assertion_authorization_grant(
        &self,
    ) -> terrars::ListRef<Client_IdentityAssertionAuthorizationGrantRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!(
                "{}.identity_assertion_authorization_grant",
                self.extract_ref()
            ),
        )
    }

    #[doc = "Get a reference to the value of field `jwt_configuration` after provisioning.\n"]
    pub fn jwt_configuration(&self) -> terrars::ListRef<Client_JwtConfigurationRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.jwt_configuration", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `mobile` after provisioning.\n"]
    pub fn mobile(&self) -> terrars::ListRef<Client_MobileRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.mobile", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `my_organization_configuration` after provisioning.\n"]
    pub fn my_organization_configuration(
        &self,
    ) -> terrars::ListRef<Client_MyOrganizationConfigurationRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.my_organization_configuration", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `native_social_login` after provisioning.\n"]
    pub fn native_social_login(&self) -> terrars::ListRef<Client_NativeSocialLoginRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.native_social_login", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `oidc_logout` after provisioning.\n"]
    pub fn oidc_logout(&self) -> terrars::ListRef<Client_OidcLogoutRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.oidc_logout", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `oidc_support` after provisioning.\n"]
    pub fn oidc_support(&self) -> terrars::ListRef<Client_OidcSupportRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.oidc_support", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `refresh_token` after provisioning.\n"]
    pub fn refresh_token(&self) -> terrars::ListRef<Client_RefreshTokenRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.refresh_token", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `session_transfer` after provisioning.\n"]
    pub fn session_transfer(&self) -> terrars::ListRef<Client_SessionTransferRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.session_transfer", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `token_exchange` after provisioning.\n"]
    pub fn token_exchange(&self) -> terrars::ListRef<Client_TokenExchangeRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.token_exchange", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `token_quota` after provisioning.\n"]
    pub fn token_quota(&self) -> terrars::ListRef<Client_TokenQuotaRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.token_quota", self.extract_ref()),
        )
    }
}

impl terrars::Referable for Client {
    fn extract_ref(&self) -> std::string::String {
        format!(
            "{}.{}",
            self.0.extract_resource_type(),
            self.0.extract_tf_id()
        )
    }
}

impl terrars::Resource for Client {}

impl terrars::ToListMappable for Client {
    type O = terrars::ListRef<ClientRef>;

    fn do_map(self, base: std::string::String) -> Self::O {
        self.0.data.borrow_mut().for_each = std::option::Option::Some(format!("${{{}}}", base));
        terrars::ListRef::new(self.0.shared.clone(), self.extract_ref())
    }
}

impl terrars::Resource_ for Client_ {
    fn extract_resource_type(&self) -> std::string::String {
        "auth0_client".into()
    }

    fn extract_tf_id(&self) -> std::string::String {
        self.tf_id.clone()
    }

    fn extract_value(&self) -> serde_json::Value {
        serde_json::to_value(&self.data).unwrap()
    }
}

pub struct BuildClient {
    pub tf_id: std::string::String,
    #[doc = "Name of the client."]
    pub name: terrars::PrimField<std::string::String>,
}

impl BuildClient {
    pub fn build(self, stack: &mut terrars::Stack) -> Client {
        let out = Client(std::rc::Rc::new(Client_ {
            shared: stack.shared.clone(),
            tf_id: self.tf_id,
            data: std::cell::RefCell::new(ClientData_ {
                depends_on: core::default::Default::default(),
                provider: std::option::Option::None,
                lifecycle: core::default::Default::default(),
                for_each: std::option::Option::None,
                provisioner: core::default::Default::default(),
                agent_id: core::default::Default::default(),
                allowed_clients: core::default::Default::default(),
                allowed_logout_urls: core::default::Default::default(),
                allowed_origins: core::default::Default::default(),
                app_type: core::default::Default::default(),
                async_approval_notification_channels: core::default::Default::default(),
                callbacks: core::default::Default::default(),
                client_aliases: core::default::Default::default(),
                client_id: core::default::Default::default(),
                client_metadata: core::default::Default::default(),
                compliance_level: core::default::Default::default(),
                cross_origin_auth: core::default::Default::default(),
                cross_origin_loc: core::default::Default::default(),
                custom_login_page: core::default::Default::default(),
                custom_login_page_on: core::default::Default::default(),
                description: core::default::Default::default(),
                encryption_key: core::default::Default::default(),
                form_template: core::default::Default::default(),
                grant_types: core::default::Default::default(),
                id: core::default::Default::default(),
                initiate_login_uri: core::default::Default::default(),
                is_first_party: core::default::Default::default(),
                is_token_endpoint_ip_header_trusted: core::default::Default::default(),
                logo_uri: core::default::Default::default(),
                name: self.name,
                oidc_backchannel_logout_urls: core::default::Default::default(),
                oidc_conformant: core::default::Default::default(),
                organization_discovery_methods: core::default::Default::default(),
                organization_require_behavior: core::default::Default::default(),
                organization_usage: core::default::Default::default(),
                redirection_policy: core::default::Default::default(),
                require_proof_of_possession: core::default::Default::default(),
                require_pushed_authorization_requests: core::default::Default::default(),
                resource_server_identifier: core::default::Default::default(),
                skip_non_verifiable_callback_uri_confirmation_prompt:
                    core::default::Default::default(),
                sso: core::default::Default::default(),
                sso_disabled: core::default::Default::default(),
                third_party_security_mode: core::default::Default::default(),
                web_origins: core::default::Default::default(),
                addons: core::default::Default::default(),
                anonymous_sessions: core::default::Default::default(),
                b2b_integration_configuration: core::default::Default::default(),
                default_organization: core::default::Default::default(),
                express_configuration: core::default::Default::default(),
                fedcm_login: core::default::Default::default(),
                identity_assertion_authorization_grant: core::default::Default::default(),
                jwt_configuration: core::default::Default::default(),
                mobile: core::default::Default::default(),
                my_organization_configuration: core::default::Default::default(),
                native_social_login: core::default::Default::default(),
                oidc_logout: core::default::Default::default(),
                oidc_support: core::default::Default::default(),
                refresh_token: core::default::Default::default(),
                session_transfer: core::default::Default::default(),
                token_exchange: core::default::Default::default(),
                token_quota: core::default::Default::default(),
                dynamic: core::default::Default::default(),
            }),
        }));
        stack.add_resource(out.0.clone());
        out
    }
}

pub struct ClientRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for ClientRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Self {
        Self {
            shared: shared,
            base: base,
        }
    }
}

impl ClientRef {
    fn extract_ref(&self) -> std::string::String {
        self.base.clone()
    }

    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `agent_id` after provisioning.\nThe ID of the agent (`auth0_agent`) this client is linked to (EA Only). Reference it as `auth0_agent.<name>.id` so that the client is unlinked before the agent is deleted. Removing the attribute unlinks the client, including a link made outside Terraform. Requires the `update:clients` and `update:agents` scopes."]
    pub fn agent_id(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.agent_id", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `allowed_clients` after provisioning.\nList of applications ID's that will be allowed to make delegation request. By default, all applications will be allowed."]
    pub fn allowed_clients(&self) -> terrars::ListRef<terrars::PrimExpr<std::string::String>> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.allowed_clients", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `allowed_logout_urls` after provisioning.\nURLs that Auth0 may redirect to after logout."]
    pub fn allowed_logout_urls(&self) -> terrars::ListRef<terrars::PrimExpr<std::string::String>> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.allowed_logout_urls", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `allowed_origins` after provisioning.\nURLs that represent valid origins for cross-origin resource sharing. By default, all your callback URLs will be allowed."]
    pub fn allowed_origins(&self) -> terrars::ListRef<terrars::PrimExpr<std::string::String>> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.allowed_origins", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `app_type` after provisioning.\nType of application the client represents. Possible values are: `native`, `spa`, `regular_web`, `non_interactive`, `resource_server`,`sso_integration`. Specific SSO integrations types accepted as well are: `rms`, `box`, `cloudbees`, `concur`, `dropbox`, `mscrm`, `echosign`, `egnyte`, `newrelic`, `office365`, `salesforce`, `sentry`, `sharepoint`, `slack`, `springcm`, `zendesk`, `zoom`, `express_configuration`"]
    pub fn app_type(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.app_type", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `async_approval_notification_channels` after provisioning.\nList of notification channels enabled for CIBA (Client-Initiated Backchannel Authentication) requests initiated by this client. Valid values are `guardian-push`, `email`, and `my-account`.The order is significant as this is the order in which notification channels will be evaluated."]
    pub fn async_approval_notification_channels(
        &self,
    ) -> terrars::ListRef<terrars::PrimExpr<std::string::String>> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!(
                "{}.async_approval_notification_channels",
                self.extract_ref()
            ),
        )
    }

    #[doc = "Get a reference to the value of field `callbacks` after provisioning.\nURLs that Auth0 may call back to after a user authenticates for the client. Make sure to specify the protocol (https://) otherwise the callback may fail in some cases. With the exception of custom URI schemes for native clients, all callbacks should use protocol https://."]
    pub fn callbacks(&self) -> terrars::ListRef<terrars::PrimExpr<std::string::String>> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.callbacks", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `client_aliases` after provisioning.\nList of audiences/realms for SAML protocol. Used by the wsfed addon."]
    pub fn client_aliases(&self) -> terrars::ListRef<terrars::PrimExpr<std::string::String>> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.client_aliases", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `client_id` after provisioning.\nThe ID of the client. If not provided, Auth0 will generate one automatically. Use this to specify a custom client ID for migration or tenant-copy scenarios. Requires feature flag to be enabled on the tenant."]
    pub fn client_id(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.client_id", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `client_metadata` after provisioning.\nMetadata associated with the client, in the form of an object with string values (max 255 chars). Maximum of 10 metadata properties allowed. Field names (max 255 chars) are alphanumeric and may only include the following special characters: `:,-+=_*?\"/\\()<>@ [Tab] [Space]`."]
    pub fn client_metadata(&self) -> terrars::RecRef<terrars::PrimExpr<std::string::String>> {
        terrars::RecRef::new(
            self.shared().clone(),
            format!("{}.client_metadata", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `compliance_level` after provisioning.\nDefines the compliance level for this client, which may restrict it's capabilities. Can be one of `none`, `fapi1_adv_pkj_par`, `fapi1_adv_mtls_par`."]
    pub fn compliance_level(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.compliance_level", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `cross_origin_auth` after provisioning.\nWhether this client can be used to make cross-origin authentication requests (`true`) or it is not allowed to make such requests (`false`)."]
    pub fn cross_origin_auth(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.cross_origin_auth", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `cross_origin_loc` after provisioning.\nURL of the location in your site where the cross-origin verification takes place for the cross-origin auth flow when performing authentication in your own domain instead of Auth0 Universal Login page."]
    pub fn cross_origin_loc(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.cross_origin_loc", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `custom_login_page` after provisioning.\nThe content (HTML, CSS, JS) of the custom login page."]
    pub fn custom_login_page(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.custom_login_page", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `custom_login_page_on` after provisioning.\nIndicates whether a custom login page is to be used."]
    pub fn custom_login_page_on(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.custom_login_page_on", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `description` after provisioning.\nDescription of the purpose of the client."]
    pub fn description(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.description", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `encryption_key` after provisioning.\nEncryption used for WS-Fed responses with this client."]
    pub fn encryption_key(&self) -> terrars::RecRef<terrars::PrimExpr<std::string::String>> {
        terrars::RecRef::new(
            self.shared().clone(),
            format!("{}.encryption_key", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `external_client_id` after provisioning.\nThe URL of the Client ID Metadata Document. Only present for CIMD-registered clients."]
    pub fn external_client_id(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.external_client_id", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `external_metadata_created_by` after provisioning.\nWho created the external metadata client: `admin` (via Management API), `client` (self-registered), or `unknown`."]
    pub fn external_metadata_created_by(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.external_metadata_created_by", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `external_metadata_status` after provisioning.\nWhether Auth0 can resolve the Client ID Metadata Document from its cache or the document URL: `available` or `unavailable` (EA only). Only present for CIMD-registered clients."]
    pub fn external_metadata_status(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.external_metadata_status", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `external_metadata_type` after provisioning.\nType of external metadata. Value is `cimd` for CIMD-registered clients."]
    pub fn external_metadata_type(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.external_metadata_type", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `form_template` after provisioning.\nHTML form template to be used for WS-Federation."]
    pub fn form_template(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.form_template", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `grant_types` after provisioning.\nTypes of grants that this client is authorized to use."]
    pub fn grant_types(&self) -> terrars::ListRef<terrars::PrimExpr<std::string::String>> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.grant_types", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `id` after provisioning.\n"]
    pub fn id(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.id", self.extract_ref()))
    }

    #[doc = "Get a reference to the value of field `initiate_login_uri` after provisioning.\nInitiate login URI. Must be HTTPS or an empty string. May contain Auth0 dynamic login URI placeholders such as `{organization.metadata.public_login_host}` or `{custom_domain.metadata.public_app_host}`, which are resolved by Auth0 at request time. See https://auth0.com/docs/get-started/applications/application-settings."]
    pub fn initiate_login_uri(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.initiate_login_uri", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `is_first_party` after provisioning.\nIndicates whether this client is a first-party client."]
    pub fn is_first_party(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.is_first_party", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `is_token_endpoint_ip_header_trusted` after provisioning.\nIndicates whether the token endpoint IP header is trusted. Requires the authentication method to be set to `client_secret_post` or `client_secret_basic`. Setting this property when creating the resource, will default the authentication method to `client_secret_post`. To change the authentication method to `client_secret_basic` use the `auth0_client_credentials` resource."]
    pub fn is_token_endpoint_ip_header_trusted(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.is_token_endpoint_ip_header_trusted", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `jwks_uri` after provisioning.\nURL for the JSON Web Key Set (JWKS) containing the public keys used for `private_key_jwt` authentication. Only present for CIMD clients using `private_key_jwt` authentication."]
    pub fn jwks_uri(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.jwks_uri", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `logo_uri` after provisioning.\nURL of the logo for the client. Recommended size is 150px x 150px. If none is set, the default badge for the application type will be shown."]
    pub fn logo_uri(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.logo_uri", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `name` after provisioning.\nName of the client."]
    pub fn name(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.name", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `oidc_backchannel_logout_urls` after provisioning.\nSet of URLs that are valid to call back from Auth0 for OIDC backchannel logout. Currently only one URL is allowed."]
    pub fn oidc_backchannel_logout_urls(
        &self,
    ) -> terrars::SetRef<terrars::PrimExpr<std::string::String>> {
        terrars::SetRef::new(
            self.shared().clone(),
            format!("{}.oidc_backchannel_logout_urls", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `oidc_conformant` after provisioning.\nIndicates whether this client will conform to strict OIDC specifications."]
    pub fn oidc_conformant(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.oidc_conformant", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `organization_discovery_methods` after provisioning.\nMethods for discovering organizations during the pre_login_prompt. Can include `email` (allows users to find their organization by entering their email address) and/or `organization_name` (requires users to enter the organization name directly). These methods can be combined. Setting this property requires that `organization_require_behavior` is set to `pre_login_prompt`. For clients that set `b2b_integration_configuration`, server-side defaults the values when this is not specified; Set to `[]` (empty array) to clear the values."]
    pub fn organization_discovery_methods(
        &self,
    ) -> terrars::ListRef<terrars::PrimExpr<std::string::String>> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.organization_discovery_methods", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `organization_require_behavior` after provisioning.\nDefines how to proceed during an authentication transaction when `organization_usage = \"require\"`. Can be `no_prompt` (default), `pre_login_prompt` or  `post_login_prompt`."]
    pub fn organization_require_behavior(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.organization_require_behavior", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `organization_usage` after provisioning.\nDefines how to proceed during an authentication transaction with regards to an organization. Can be `deny` (default), `allow` or `require`."]
    pub fn organization_usage(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.organization_usage", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `redirection_policy` after provisioning.\nControls whether Auth0 redirects users to the application's callback URL on authentication errors or in email verification flows.Allowed values: `allow_always` or `open_redirect_protection`."]
    pub fn redirection_policy(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.redirection_policy", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `require_proof_of_possession` after provisioning.\nMakes the use of Proof-of-Possession mandatory for this client."]
    pub fn require_proof_of_possession(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.require_proof_of_possession", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `require_pushed_authorization_requests` after provisioning.\nMakes the use of Pushed Authorization Requests mandatory for this client. This feature currently needs to be enabled on the tenant in order to make use of it."]
    pub fn require_pushed_authorization_requests(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!(
                "{}.require_pushed_authorization_requests",
                self.extract_ref()
            ),
        )
    }

    #[doc = "Get a reference to the value of field `resource_server_identifier` after provisioning.\nThe identifier of a resource server that client is associated withThis property can be sent only when app_type=resource_server.This property can not be changed, once the client is created."]
    pub fn resource_server_identifier(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.resource_server_identifier", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `signing_keys` after provisioning.\nList containing a map of the public cert of the signing key and the public cert of the signing key in PKCS7."]
    pub fn signing_keys(
        &self,
    ) -> terrars::ListRef<terrars::RecRef<terrars::PrimExpr<std::string::String>>> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.signing_keys", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `skip_non_verifiable_callback_uri_confirmation_prompt` after provisioning.\nIndicates whether the confirmation prompt appears when using non-verifiable callback URIs. Set to true to skip the prompt, false to show it, or null to unset. Accepts (true/false/null) or (\"true\"/\"false\"/\"null\") "]
    pub fn skip_non_verifiable_callback_uri_confirmation_prompt(
        &self,
    ) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!(
                "{}.skip_non_verifiable_callback_uri_confirmation_prompt",
                self.extract_ref()
            ),
        )
    }

    #[doc = "Get a reference to the value of field `sso` after provisioning.\nApplies only to SSO clients and determines whether Auth0 will handle Single Sign-On (true) or whether the identity provider will (false)."]
    pub fn sso(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.sso", self.extract_ref()))
    }

    #[doc = "Get a reference to the value of field `sso_disabled` after provisioning.\nIndicates whether or not SSO is disabled."]
    pub fn sso_disabled(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.sso_disabled", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `third_party_security_mode` after provisioning.\nSecurity mode for third-party clients. Allowed values: `strict` or `permissive`."]
    pub fn third_party_security_mode(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.third_party_security_mode", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `web_origins` after provisioning.\nURLs that represent valid web origins for use with web message response mode."]
    pub fn web_origins(&self) -> terrars::ListRef<terrars::PrimExpr<std::string::String>> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.web_origins", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `addons` after provisioning.\n"]
    pub fn addons(&self) -> terrars::ListRef<Client_AddonsRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.addons", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `anonymous_sessions` after provisioning.\n"]
    pub fn anonymous_sessions(&self) -> terrars::ListRef<Client_AnonymousSessionsRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.anonymous_sessions", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `b2b_integration_configuration` after provisioning.\n"]
    pub fn b2b_integration_configuration(
        &self,
    ) -> terrars::ListRef<Client_B2bIntegrationConfigurationRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.b2b_integration_configuration", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `default_organization` after provisioning.\n"]
    pub fn default_organization(&self) -> terrars::ListRef<Client_DefaultOrganizationRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.default_organization", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `express_configuration` after provisioning.\n"]
    pub fn express_configuration(&self) -> terrars::ListRef<Client_ExpressConfigurationRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.express_configuration", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `fedcm_login` after provisioning.\n"]
    pub fn fedcm_login(&self) -> terrars::ListRef<Client_FedcmLoginRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.fedcm_login", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `identity_assertion_authorization_grant` after provisioning.\n"]
    pub fn identity_assertion_authorization_grant(
        &self,
    ) -> terrars::ListRef<Client_IdentityAssertionAuthorizationGrantRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!(
                "{}.identity_assertion_authorization_grant",
                self.extract_ref()
            ),
        )
    }

    #[doc = "Get a reference to the value of field `jwt_configuration` after provisioning.\n"]
    pub fn jwt_configuration(&self) -> terrars::ListRef<Client_JwtConfigurationRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.jwt_configuration", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `mobile` after provisioning.\n"]
    pub fn mobile(&self) -> terrars::ListRef<Client_MobileRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.mobile", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `my_organization_configuration` after provisioning.\n"]
    pub fn my_organization_configuration(
        &self,
    ) -> terrars::ListRef<Client_MyOrganizationConfigurationRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.my_organization_configuration", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `native_social_login` after provisioning.\n"]
    pub fn native_social_login(&self) -> terrars::ListRef<Client_NativeSocialLoginRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.native_social_login", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `oidc_logout` after provisioning.\n"]
    pub fn oidc_logout(&self) -> terrars::ListRef<Client_OidcLogoutRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.oidc_logout", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `oidc_support` after provisioning.\n"]
    pub fn oidc_support(&self) -> terrars::ListRef<Client_OidcSupportRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.oidc_support", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `refresh_token` after provisioning.\n"]
    pub fn refresh_token(&self) -> terrars::ListRef<Client_RefreshTokenRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.refresh_token", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `session_transfer` after provisioning.\n"]
    pub fn session_transfer(&self) -> terrars::ListRef<Client_SessionTransferRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.session_transfer", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `token_exchange` after provisioning.\n"]
    pub fn token_exchange(&self) -> terrars::ListRef<Client_TokenExchangeRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.token_exchange", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `token_quota` after provisioning.\n"]
    pub fn token_quota(&self) -> terrars::ListRef<Client_TokenQuotaRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.token_quota", self.extract_ref()),
        )
    }
}

#[derive(Serialize, Default)]
struct ClientDynamic {
    addons: std::option::Option<terrars::DynamicBlock<Client_Addons>>,
    anonymous_sessions: std::option::Option<terrars::DynamicBlock<Client_AnonymousSessions>>,
    b2b_integration_configuration:
        std::option::Option<terrars::DynamicBlock<Client_B2bIntegrationConfiguration>>,
    default_organization: std::option::Option<terrars::DynamicBlock<Client_DefaultOrganization>>,
    express_configuration: std::option::Option<terrars::DynamicBlock<Client_ExpressConfiguration>>,
    fedcm_login: std::option::Option<terrars::DynamicBlock<Client_FedcmLogin>>,
    identity_assertion_authorization_grant:
        std::option::Option<terrars::DynamicBlock<Client_IdentityAssertionAuthorizationGrant>>,
    jwt_configuration: std::option::Option<terrars::DynamicBlock<Client_JwtConfiguration>>,
    mobile: std::option::Option<terrars::DynamicBlock<Client_Mobile>>,
    my_organization_configuration:
        std::option::Option<terrars::DynamicBlock<Client_MyOrganizationConfiguration>>,
    native_social_login: std::option::Option<terrars::DynamicBlock<Client_NativeSocialLogin>>,
    oidc_logout: std::option::Option<terrars::DynamicBlock<Client_OidcLogout>>,
    oidc_support: std::option::Option<terrars::DynamicBlock<Client_OidcSupport>>,
    refresh_token: std::option::Option<terrars::DynamicBlock<Client_RefreshToken>>,
    session_transfer: std::option::Option<terrars::DynamicBlock<Client_SessionTransfer>>,
    token_exchange: std::option::Option<terrars::DynamicBlock<Client_TokenExchange>>,
    token_quota: std::option::Option<terrars::DynamicBlock<Client_TokenQuota>>,
}

#[derive(Serialize)]
pub struct Client_Addons_Aws {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    lifetime_in_seconds: std::option::Option<terrars::PrimField<f64>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    principal: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    role: std::option::Option<terrars::PrimField<std::string::String>>,
}

impl Client_Addons_Aws {
    #[doc = "Set the field `lifetime_in_seconds`.\nAWS token lifetime in seconds."]
    pub fn set_lifetime_in_seconds(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<f64>>,
    ) -> Self {
        self.lifetime_in_seconds = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `principal`.\nAWS principal ARN, for example `arn:aws:iam::010616021751:saml-provider/idpname`."]
    pub fn set_principal(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.principal = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `role`.\nAWS role ARN, for example `arn:aws:iam::010616021751:role/foo`."]
    pub fn set_role(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.role = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for Client_Addons_Aws {
    type O = terrars::BlockAssignable<Client_Addons_Aws>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_Addons_Aws {}

impl BuildClient_Addons_Aws {
    pub fn build(self) -> Client_Addons_Aws {
        Client_Addons_Aws {
            lifetime_in_seconds: core::default::Default::default(),
            principal: core::default::Default::default(),
            role: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_Addons_AwsRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_Addons_AwsRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_Addons_AwsRef {
        Client_Addons_AwsRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_Addons_AwsRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `lifetime_in_seconds` after provisioning.\nAWS token lifetime in seconds."]
    pub fn lifetime_in_seconds(&self) -> terrars::PrimExpr<f64> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.lifetime_in_seconds", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `principal` after provisioning.\nAWS principal ARN, for example `arn:aws:iam::010616021751:saml-provider/idpname`."]
    pub fn principal(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.principal", self.base))
    }

    #[doc = "Get a reference to the value of field `role` after provisioning.\nAWS role ARN, for example `arn:aws:iam::010616021751:role/foo`."]
    pub fn role(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.role", self.base))
    }
}

#[derive(Serialize)]
pub struct Client_Addons_AzureBlob {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    account_name: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    blob_delete: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    blob_name: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    blob_read: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    blob_write: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    container_delete: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    container_list: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    container_name: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    container_read: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    container_write: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    expiration: std::option::Option<terrars::PrimField<f64>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    signed_identifier: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    storage_access_key: std::option::Option<terrars::PrimField<std::string::String>>,
}

impl Client_Addons_AzureBlob {
    #[doc = "Set the field `account_name`.\nYour Azure storage account name. Usually first segment in your Azure storage URL, for example `https://acme-org.blob.core.windows.net` would be the account name `acme-org`."]
    pub fn set_account_name(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.account_name = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `blob_delete`.\nIndicates if the issued token has permission to delete the blob."]
    pub fn set_blob_delete(mut self, v: impl std::convert::Into<terrars::PrimField<bool>>) -> Self {
        self.blob_delete = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `blob_name`.\nEntity to request a token for, such as `my-blob`. If blank the computed SAS will apply to the entire storage container."]
    pub fn set_blob_name(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.blob_name = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `blob_read`.\nIndicates if the issued token has permission to read the content, properties, metadata and block list. Use the blob as the source of a copy operation."]
    pub fn set_blob_read(mut self, v: impl std::convert::Into<terrars::PrimField<bool>>) -> Self {
        self.blob_read = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `blob_write`.\nIndicates if the issued token has permission to create or write content, properties, metadata, or block list. Snapshot or lease the blob. Resize the blob (page blob only). Use the blob as the destination of a copy operation within the same account."]
    pub fn set_blob_write(mut self, v: impl std::convert::Into<terrars::PrimField<bool>>) -> Self {
        self.blob_write = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `container_delete`.\nIndicates if issued token has permission to delete any blob in the container."]
    pub fn set_container_delete(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.container_delete = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `container_list`.\nIndicates if the issued token has permission to list blobs in the container."]
    pub fn set_container_list(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.container_list = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `container_name`.\nContainer to request a token for, such as `my-container`."]
    pub fn set_container_name(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.container_name = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `container_read`.\nIndicates if the issued token has permission to read the content, properties, metadata or block list of any blob in the container. Use any blob in the container as the source of a copy operation."]
    pub fn set_container_read(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.container_read = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `container_write`.\nIndicates that for any blob in the container if the issued token has permission to create or write content, properties, metadata, or block list. Snapshot or lease the blob. Resize the blob (page blob only). Use the blob as the destination of a copy operation within the same account."]
    pub fn set_container_write(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.container_write = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `expiration`.\nExpiration in minutes for the generated token (default of 5 minutes)."]
    pub fn set_expiration(mut self, v: impl std::convert::Into<terrars::PrimField<f64>>) -> Self {
        self.expiration = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `signed_identifier`.\nShared access policy identifier defined in your storage account resource."]
    pub fn set_signed_identifier(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.signed_identifier = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `storage_access_key`.\nAccess key associated with this storage account."]
    pub fn set_storage_access_key(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.storage_access_key = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for Client_Addons_AzureBlob {
    type O = terrars::BlockAssignable<Client_Addons_AzureBlob>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_Addons_AzureBlob {}

impl BuildClient_Addons_AzureBlob {
    pub fn build(self) -> Client_Addons_AzureBlob {
        Client_Addons_AzureBlob {
            account_name: core::default::Default::default(),
            blob_delete: core::default::Default::default(),
            blob_name: core::default::Default::default(),
            blob_read: core::default::Default::default(),
            blob_write: core::default::Default::default(),
            container_delete: core::default::Default::default(),
            container_list: core::default::Default::default(),
            container_name: core::default::Default::default(),
            container_read: core::default::Default::default(),
            container_write: core::default::Default::default(),
            expiration: core::default::Default::default(),
            signed_identifier: core::default::Default::default(),
            storage_access_key: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_Addons_AzureBlobRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_Addons_AzureBlobRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_Addons_AzureBlobRef {
        Client_Addons_AzureBlobRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_Addons_AzureBlobRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `account_name` after provisioning.\nYour Azure storage account name. Usually first segment in your Azure storage URL, for example `https://acme-org.blob.core.windows.net` would be the account name `acme-org`."]
    pub fn account_name(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.account_name", self.base))
    }

    #[doc = "Get a reference to the value of field `blob_delete` after provisioning.\nIndicates if the issued token has permission to delete the blob."]
    pub fn blob_delete(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.blob_delete", self.base))
    }

    #[doc = "Get a reference to the value of field `blob_name` after provisioning.\nEntity to request a token for, such as `my-blob`. If blank the computed SAS will apply to the entire storage container."]
    pub fn blob_name(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.blob_name", self.base))
    }

    #[doc = "Get a reference to the value of field `blob_read` after provisioning.\nIndicates if the issued token has permission to read the content, properties, metadata and block list. Use the blob as the source of a copy operation."]
    pub fn blob_read(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.blob_read", self.base))
    }

    #[doc = "Get a reference to the value of field `blob_write` after provisioning.\nIndicates if the issued token has permission to create or write content, properties, metadata, or block list. Snapshot or lease the blob. Resize the blob (page blob only). Use the blob as the destination of a copy operation within the same account."]
    pub fn blob_write(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.blob_write", self.base))
    }

    #[doc = "Get a reference to the value of field `container_delete` after provisioning.\nIndicates if issued token has permission to delete any blob in the container."]
    pub fn container_delete(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.container_delete", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `container_list` after provisioning.\nIndicates if the issued token has permission to list blobs in the container."]
    pub fn container_list(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.container_list", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `container_name` after provisioning.\nContainer to request a token for, such as `my-container`."]
    pub fn container_name(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.container_name", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `container_read` after provisioning.\nIndicates if the issued token has permission to read the content, properties, metadata or block list of any blob in the container. Use any blob in the container as the source of a copy operation."]
    pub fn container_read(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.container_read", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `container_write` after provisioning.\nIndicates that for any blob in the container if the issued token has permission to create or write content, properties, metadata, or block list. Snapshot or lease the blob. Resize the blob (page blob only). Use the blob as the destination of a copy operation within the same account."]
    pub fn container_write(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.container_write", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `expiration` after provisioning.\nExpiration in minutes for the generated token (default of 5 minutes)."]
    pub fn expiration(&self) -> terrars::PrimExpr<f64> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.expiration", self.base))
    }

    #[doc = "Get a reference to the value of field `signed_identifier` after provisioning.\nShared access policy identifier defined in your storage account resource."]
    pub fn signed_identifier(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.signed_identifier", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `storage_access_key` after provisioning.\nAccess key associated with this storage account."]
    pub fn storage_access_key(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.storage_access_key", self.base),
        )
    }
}

#[derive(Serialize)]
pub struct Client_Addons_AzureSb {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    entity_path: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    expiration: std::option::Option<terrars::PrimField<f64>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    namespace: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    sas_key: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    sas_key_name: std::option::Option<terrars::PrimField<std::string::String>>,
}

impl Client_Addons_AzureSb {
    #[doc = "Set the field `entity_path`.\nEntity you want to request a token for, such as `my-queue`."]
    pub fn set_entity_path(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.entity_path = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `expiration`.\nOptional expiration in minutes for the generated token. Defaults to 5 minutes."]
    pub fn set_expiration(mut self, v: impl std::convert::Into<terrars::PrimField<f64>>) -> Self {
        self.expiration = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `namespace`.\nYour Azure Service Bus namespace. Usually the first segment of your Service Bus URL (for example `https://acme-org.servicebus.windows.net` would be `acme-org`)."]
    pub fn set_namespace(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.namespace = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `sas_key`.\nPrimary Key associated with your shared access policy."]
    pub fn set_sas_key(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.sas_key = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `sas_key_name`.\nYour shared access policy name defined in your Service Bus entity."]
    pub fn set_sas_key_name(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.sas_key_name = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for Client_Addons_AzureSb {
    type O = terrars::BlockAssignable<Client_Addons_AzureSb>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_Addons_AzureSb {}

impl BuildClient_Addons_AzureSb {
    pub fn build(self) -> Client_Addons_AzureSb {
        Client_Addons_AzureSb {
            entity_path: core::default::Default::default(),
            expiration: core::default::Default::default(),
            namespace: core::default::Default::default(),
            sas_key: core::default::Default::default(),
            sas_key_name: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_Addons_AzureSbRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_Addons_AzureSbRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_Addons_AzureSbRef {
        Client_Addons_AzureSbRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_Addons_AzureSbRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `entity_path` after provisioning.\nEntity you want to request a token for, such as `my-queue`."]
    pub fn entity_path(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.entity_path", self.base))
    }

    #[doc = "Get a reference to the value of field `expiration` after provisioning.\nOptional expiration in minutes for the generated token. Defaults to 5 minutes."]
    pub fn expiration(&self) -> terrars::PrimExpr<f64> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.expiration", self.base))
    }

    #[doc = "Get a reference to the value of field `namespace` after provisioning.\nYour Azure Service Bus namespace. Usually the first segment of your Service Bus URL (for example `https://acme-org.servicebus.windows.net` would be `acme-org`)."]
    pub fn namespace(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.namespace", self.base))
    }

    #[doc = "Get a reference to the value of field `sas_key` after provisioning.\nPrimary Key associated with your shared access policy."]
    pub fn sas_key(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.sas_key", self.base))
    }

    #[doc = "Get a reference to the value of field `sas_key_name` after provisioning.\nYour shared access policy name defined in your Service Bus entity."]
    pub fn sas_key_name(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.sas_key_name", self.base))
    }
}

#[derive(Serialize)]
pub struct Client_Addons_Box {}

impl Client_Addons_Box {}

impl terrars::ToListMappable for Client_Addons_Box {
    type O = terrars::BlockAssignable<Client_Addons_Box>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_Addons_Box {}

impl BuildClient_Addons_Box {
    pub fn build(self) -> Client_Addons_Box {
        Client_Addons_Box {}
    }
}

#[allow(dead_code)]
pub struct Client_Addons_BoxRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_Addons_BoxRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_Addons_BoxRef {
        Client_Addons_BoxRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_Addons_BoxRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }
}

#[derive(Serialize)]
pub struct Client_Addons_Cloudbees {}

impl Client_Addons_Cloudbees {}

impl terrars::ToListMappable for Client_Addons_Cloudbees {
    type O = terrars::BlockAssignable<Client_Addons_Cloudbees>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_Addons_Cloudbees {}

impl BuildClient_Addons_Cloudbees {
    pub fn build(self) -> Client_Addons_Cloudbees {
        Client_Addons_Cloudbees {}
    }
}

#[allow(dead_code)]
pub struct Client_Addons_CloudbeesRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_Addons_CloudbeesRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_Addons_CloudbeesRef {
        Client_Addons_CloudbeesRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_Addons_CloudbeesRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }
}

#[derive(Serialize)]
pub struct Client_Addons_Concur {}

impl Client_Addons_Concur {}

impl terrars::ToListMappable for Client_Addons_Concur {
    type O = terrars::BlockAssignable<Client_Addons_Concur>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_Addons_Concur {}

impl BuildClient_Addons_Concur {
    pub fn build(self) -> Client_Addons_Concur {
        Client_Addons_Concur {}
    }
}

#[allow(dead_code)]
pub struct Client_Addons_ConcurRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_Addons_ConcurRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_Addons_ConcurRef {
        Client_Addons_ConcurRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_Addons_ConcurRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }
}

#[derive(Serialize)]
pub struct Client_Addons_Dropbox {}

impl Client_Addons_Dropbox {}

impl terrars::ToListMappable for Client_Addons_Dropbox {
    type O = terrars::BlockAssignable<Client_Addons_Dropbox>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_Addons_Dropbox {}

impl BuildClient_Addons_Dropbox {
    pub fn build(self) -> Client_Addons_Dropbox {
        Client_Addons_Dropbox {}
    }
}

#[allow(dead_code)]
pub struct Client_Addons_DropboxRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_Addons_DropboxRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_Addons_DropboxRef {
        Client_Addons_DropboxRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_Addons_DropboxRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }
}

#[derive(Serialize)]
pub struct Client_Addons_Echosign {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    domain: std::option::Option<terrars::PrimField<std::string::String>>,
}

impl Client_Addons_Echosign {
    #[doc = "Set the field `domain`.\nYour custom domain found in your EchoSign URL, for example `https://acme-org.echosign.com` would be `acme-org`."]
    pub fn set_domain(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.domain = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for Client_Addons_Echosign {
    type O = terrars::BlockAssignable<Client_Addons_Echosign>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_Addons_Echosign {}

impl BuildClient_Addons_Echosign {
    pub fn build(self) -> Client_Addons_Echosign {
        Client_Addons_Echosign {
            domain: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_Addons_EchosignRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_Addons_EchosignRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_Addons_EchosignRef {
        Client_Addons_EchosignRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_Addons_EchosignRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `domain` after provisioning.\nYour custom domain found in your EchoSign URL, for example `https://acme-org.echosign.com` would be `acme-org`."]
    pub fn domain(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.domain", self.base))
    }
}

#[derive(Serialize)]
pub struct Client_Addons_Egnyte {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    domain: std::option::Option<terrars::PrimField<std::string::String>>,
}

impl Client_Addons_Egnyte {
    #[doc = "Set the field `domain`.\nYour custom domain found in your Egnyte URL, for example `https://acme-org.echosign.com` would be `acme-org`."]
    pub fn set_domain(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.domain = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for Client_Addons_Egnyte {
    type O = terrars::BlockAssignable<Client_Addons_Egnyte>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_Addons_Egnyte {}

impl BuildClient_Addons_Egnyte {
    pub fn build(self) -> Client_Addons_Egnyte {
        Client_Addons_Egnyte {
            domain: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_Addons_EgnyteRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_Addons_EgnyteRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_Addons_EgnyteRef {
        Client_Addons_EgnyteRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_Addons_EgnyteRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `domain` after provisioning.\nYour custom domain found in your Egnyte URL, for example `https://acme-org.echosign.com` would be `acme-org`."]
    pub fn domain(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.domain", self.base))
    }
}

#[derive(Serialize)]
pub struct Client_Addons_Firebase {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    client_email: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    lifetime_in_seconds: std::option::Option<terrars::PrimField<f64>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    private_key: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    private_key_id: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    secret: std::option::Option<terrars::PrimField<std::string::String>>,
}

impl Client_Addons_Firebase {
    #[doc = "Set the field `client_email`.\nID of the Service Account you have created (shown as `client_email` in the generated JSON file, SDK v3+ tokens only)."]
    pub fn set_client_email(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.client_email = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `lifetime_in_seconds`.\nOptional expiration in seconds for the generated token. Defaults to 3600 seconds (SDK v3+ tokens only)."]
    pub fn set_lifetime_in_seconds(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<f64>>,
    ) -> Self {
        self.lifetime_in_seconds = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `private_key`.\nPrivate Key for signing the token (SDK v3+ tokens only)."]
    pub fn set_private_key(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.private_key = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `private_key_id`.\nOptional ID of the private key to obtain the `kid` header claim from the issued token (SDK v3+ tokens only)."]
    pub fn set_private_key_id(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.private_key_id = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `secret`.\nGoogle Firebase Secret. (SDK v2 only)."]
    pub fn set_secret(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.secret = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for Client_Addons_Firebase {
    type O = terrars::BlockAssignable<Client_Addons_Firebase>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_Addons_Firebase {}

impl BuildClient_Addons_Firebase {
    pub fn build(self) -> Client_Addons_Firebase {
        Client_Addons_Firebase {
            client_email: core::default::Default::default(),
            lifetime_in_seconds: core::default::Default::default(),
            private_key: core::default::Default::default(),
            private_key_id: core::default::Default::default(),
            secret: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_Addons_FirebaseRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_Addons_FirebaseRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_Addons_FirebaseRef {
        Client_Addons_FirebaseRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_Addons_FirebaseRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `client_email` after provisioning.\nID of the Service Account you have created (shown as `client_email` in the generated JSON file, SDK v3+ tokens only)."]
    pub fn client_email(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.client_email", self.base))
    }

    #[doc = "Get a reference to the value of field `lifetime_in_seconds` after provisioning.\nOptional expiration in seconds for the generated token. Defaults to 3600 seconds (SDK v3+ tokens only)."]
    pub fn lifetime_in_seconds(&self) -> terrars::PrimExpr<f64> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.lifetime_in_seconds", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `private_key` after provisioning.\nPrivate Key for signing the token (SDK v3+ tokens only)."]
    pub fn private_key(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.private_key", self.base))
    }

    #[doc = "Get a reference to the value of field `private_key_id` after provisioning.\nOptional ID of the private key to obtain the `kid` header claim from the issued token (SDK v3+ tokens only)."]
    pub fn private_key_id(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.private_key_id", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `secret` after provisioning.\nGoogle Firebase Secret. (SDK v2 only)."]
    pub fn secret(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.secret", self.base))
    }
}

#[derive(Serialize)]
pub struct Client_Addons_Layer {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    expiration: std::option::Option<terrars::PrimField<f64>>,
    key_id: terrars::PrimField<std::string::String>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    principal: std::option::Option<terrars::PrimField<std::string::String>>,
    private_key: terrars::PrimField<std::string::String>,
    provider_id: terrars::PrimField<std::string::String>,
}

impl Client_Addons_Layer {
    #[doc = "Set the field `expiration`.\nOptional expiration in minutes for the generated token. Defaults to 5 minutes."]
    pub fn set_expiration(mut self, v: impl std::convert::Into<terrars::PrimField<f64>>) -> Self {
        self.expiration = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `principal`.\nName of the property used as the unique user ID in Layer. If not specified `user_id` is used."]
    pub fn set_principal(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.principal = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for Client_Addons_Layer {
    type O = terrars::BlockAssignable<Client_Addons_Layer>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_Addons_Layer {
    #[doc = "Authentication Key identifier used to sign the Layer token."]
    pub key_id: terrars::PrimField<std::string::String>,
    #[doc = "Private key for signing the Layer token."]
    pub private_key: terrars::PrimField<std::string::String>,
    #[doc = "Provider ID of your Layer account."]
    pub provider_id: terrars::PrimField<std::string::String>,
}

impl BuildClient_Addons_Layer {
    pub fn build(self) -> Client_Addons_Layer {
        Client_Addons_Layer {
            expiration: core::default::Default::default(),
            key_id: self.key_id,
            principal: core::default::Default::default(),
            private_key: self.private_key,
            provider_id: self.provider_id,
        }
    }
}

#[allow(dead_code)]
pub struct Client_Addons_LayerRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_Addons_LayerRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_Addons_LayerRef {
        Client_Addons_LayerRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_Addons_LayerRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `expiration` after provisioning.\nOptional expiration in minutes for the generated token. Defaults to 5 minutes."]
    pub fn expiration(&self) -> terrars::PrimExpr<f64> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.expiration", self.base))
    }

    #[doc = "Get a reference to the value of field `key_id` after provisioning.\nAuthentication Key identifier used to sign the Layer token."]
    pub fn key_id(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.key_id", self.base))
    }

    #[doc = "Get a reference to the value of field `principal` after provisioning.\nName of the property used as the unique user ID in Layer. If not specified `user_id` is used."]
    pub fn principal(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.principal", self.base))
    }

    #[doc = "Get a reference to the value of field `private_key` after provisioning.\nPrivate key for signing the Layer token."]
    pub fn private_key(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.private_key", self.base))
    }

    #[doc = "Get a reference to the value of field `provider_id` after provisioning.\nProvider ID of your Layer account."]
    pub fn provider_id(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.provider_id", self.base))
    }
}

#[derive(Serialize)]
pub struct Client_Addons_Mscrm {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    url: std::option::Option<terrars::PrimField<std::string::String>>,
}

impl Client_Addons_Mscrm {
    #[doc = "Set the field `url`.\nMicrosoft Dynamics CRM application URL."]
    pub fn set_url(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.url = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for Client_Addons_Mscrm {
    type O = terrars::BlockAssignable<Client_Addons_Mscrm>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_Addons_Mscrm {}

impl BuildClient_Addons_Mscrm {
    pub fn build(self) -> Client_Addons_Mscrm {
        Client_Addons_Mscrm {
            url: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_Addons_MscrmRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_Addons_MscrmRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_Addons_MscrmRef {
        Client_Addons_MscrmRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_Addons_MscrmRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `url` after provisioning.\nMicrosoft Dynamics CRM application URL."]
    pub fn url(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.url", self.base))
    }
}

#[derive(Serialize)]
pub struct Client_Addons_Newrelic {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    account: std::option::Option<terrars::PrimField<std::string::String>>,
}

impl Client_Addons_Newrelic {
    #[doc = "Set the field `account`.\nYour New Relic Account ID found in your New Relic URL after the `/accounts/` path, for example `https://rpm.newrelic.com/accounts/123456/query` would be `123456`."]
    pub fn set_account(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.account = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for Client_Addons_Newrelic {
    type O = terrars::BlockAssignable<Client_Addons_Newrelic>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_Addons_Newrelic {}

impl BuildClient_Addons_Newrelic {
    pub fn build(self) -> Client_Addons_Newrelic {
        Client_Addons_Newrelic {
            account: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_Addons_NewrelicRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_Addons_NewrelicRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_Addons_NewrelicRef {
        Client_Addons_NewrelicRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_Addons_NewrelicRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `account` after provisioning.\nYour New Relic Account ID found in your New Relic URL after the `/accounts/` path, for example `https://rpm.newrelic.com/accounts/123456/query` would be `123456`."]
    pub fn account(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.account", self.base))
    }
}

#[derive(Serialize)]
pub struct Client_Addons_Office365 {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    connection: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    domain: std::option::Option<terrars::PrimField<std::string::String>>,
}

impl Client_Addons_Office365 {
    #[doc = "Set the field `connection`.\nOptional Auth0 database connection for testing an already-configured Office 365 tenant."]
    pub fn set_connection(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.connection = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `domain`.\nYour Office 365 domain name, for example `acme-org.com`."]
    pub fn set_domain(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.domain = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for Client_Addons_Office365 {
    type O = terrars::BlockAssignable<Client_Addons_Office365>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_Addons_Office365 {}

impl BuildClient_Addons_Office365 {
    pub fn build(self) -> Client_Addons_Office365 {
        Client_Addons_Office365 {
            connection: core::default::Default::default(),
            domain: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_Addons_Office365Ref {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_Addons_Office365Ref {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_Addons_Office365Ref {
        Client_Addons_Office365Ref {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_Addons_Office365Ref {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `connection` after provisioning.\nOptional Auth0 database connection for testing an already-configured Office 365 tenant."]
    pub fn connection(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.connection", self.base))
    }

    #[doc = "Get a reference to the value of field `domain` after provisioning.\nYour Office 365 domain name, for example `acme-org.com`."]
    pub fn domain(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.domain", self.base))
    }
}

#[derive(Serialize)]
pub struct Client_Addons_Rms {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    url: std::option::Option<terrars::PrimField<std::string::String>>,
}

impl Client_Addons_Rms {
    #[doc = "Set the field `url`.\nURL of your Rights Management Server. It can be internal or external, but users will have to be able to reach it."]
    pub fn set_url(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.url = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for Client_Addons_Rms {
    type O = terrars::BlockAssignable<Client_Addons_Rms>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_Addons_Rms {}

impl BuildClient_Addons_Rms {
    pub fn build(self) -> Client_Addons_Rms {
        Client_Addons_Rms {
            url: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_Addons_RmsRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_Addons_RmsRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_Addons_RmsRef {
        Client_Addons_RmsRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_Addons_RmsRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `url` after provisioning.\nURL of your Rights Management Server. It can be internal or external, but users will have to be able to reach it."]
    pub fn url(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.url", self.base))
    }
}

#[derive(Serialize)]
pub struct Client_Addons_Salesforce {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    entity_id: std::option::Option<terrars::PrimField<std::string::String>>,
}

impl Client_Addons_Salesforce {
    #[doc = "Set the field `entity_id`.\nArbitrary logical URL that identifies the Saleforce resource, for example `https://acme-org.com`."]
    pub fn set_entity_id(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.entity_id = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for Client_Addons_Salesforce {
    type O = terrars::BlockAssignable<Client_Addons_Salesforce>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_Addons_Salesforce {}

impl BuildClient_Addons_Salesforce {
    pub fn build(self) -> Client_Addons_Salesforce {
        Client_Addons_Salesforce {
            entity_id: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_Addons_SalesforceRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_Addons_SalesforceRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_Addons_SalesforceRef {
        Client_Addons_SalesforceRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_Addons_SalesforceRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `entity_id` after provisioning.\nArbitrary logical URL that identifies the Saleforce resource, for example `https://acme-org.com`."]
    pub fn entity_id(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.entity_id", self.base))
    }
}

#[derive(Serialize)]
pub struct Client_Addons_SalesforceApi {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    client_id: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    community_name: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    community_url_section: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    principal: std::option::Option<terrars::PrimField<std::string::String>>,
}

impl Client_Addons_SalesforceApi {
    #[doc = "Set the field `client_id`.\nConsumer Key assigned by Salesforce to the Connected App."]
    pub fn set_client_id(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.client_id = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `community_name`.\nCommunity name."]
    pub fn set_community_name(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.community_name = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `community_url_section`.\nCommunity URL section."]
    pub fn set_community_url_section(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.community_url_section = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `principal`.\nName of the property in the user object that maps to a Salesforce username, for example `email`."]
    pub fn set_principal(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.principal = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for Client_Addons_SalesforceApi {
    type O = terrars::BlockAssignable<Client_Addons_SalesforceApi>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_Addons_SalesforceApi {}

impl BuildClient_Addons_SalesforceApi {
    pub fn build(self) -> Client_Addons_SalesforceApi {
        Client_Addons_SalesforceApi {
            client_id: core::default::Default::default(),
            community_name: core::default::Default::default(),
            community_url_section: core::default::Default::default(),
            principal: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_Addons_SalesforceApiRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_Addons_SalesforceApiRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> Client_Addons_SalesforceApiRef {
        Client_Addons_SalesforceApiRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_Addons_SalesforceApiRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `client_id` after provisioning.\nConsumer Key assigned by Salesforce to the Connected App."]
    pub fn client_id(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.client_id", self.base))
    }

    #[doc = "Get a reference to the value of field `community_name` after provisioning.\nCommunity name."]
    pub fn community_name(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.community_name", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `community_url_section` after provisioning.\nCommunity URL section."]
    pub fn community_url_section(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.community_url_section", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `principal` after provisioning.\nName of the property in the user object that maps to a Salesforce username, for example `email`."]
    pub fn principal(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.principal", self.base))
    }
}

#[derive(Serialize)]
pub struct Client_Addons_SalesforceSandboxApi {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    client_id: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    community_name: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    community_url_section: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    principal: std::option::Option<terrars::PrimField<std::string::String>>,
}

impl Client_Addons_SalesforceSandboxApi {
    #[doc = "Set the field `client_id`.\nConsumer Key assigned by Salesforce to the Connected App."]
    pub fn set_client_id(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.client_id = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `community_name`.\nCommunity name."]
    pub fn set_community_name(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.community_name = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `community_url_section`.\nCommunity URL section."]
    pub fn set_community_url_section(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.community_url_section = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `principal`.\nName of the property in the user object that maps to a Salesforce username, for example `email`."]
    pub fn set_principal(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.principal = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for Client_Addons_SalesforceSandboxApi {
    type O = terrars::BlockAssignable<Client_Addons_SalesforceSandboxApi>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_Addons_SalesforceSandboxApi {}

impl BuildClient_Addons_SalesforceSandboxApi {
    pub fn build(self) -> Client_Addons_SalesforceSandboxApi {
        Client_Addons_SalesforceSandboxApi {
            client_id: core::default::Default::default(),
            community_name: core::default::Default::default(),
            community_url_section: core::default::Default::default(),
            principal: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_Addons_SalesforceSandboxApiRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_Addons_SalesforceSandboxApiRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> Client_Addons_SalesforceSandboxApiRef {
        Client_Addons_SalesforceSandboxApiRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_Addons_SalesforceSandboxApiRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `client_id` after provisioning.\nConsumer Key assigned by Salesforce to the Connected App."]
    pub fn client_id(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.client_id", self.base))
    }

    #[doc = "Get a reference to the value of field `community_name` after provisioning.\nCommunity name."]
    pub fn community_name(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.community_name", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `community_url_section` after provisioning.\nCommunity URL section."]
    pub fn community_url_section(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.community_url_section", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `principal` after provisioning.\nName of the property in the user object that maps to a Salesforce username, for example `email`."]
    pub fn principal(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.principal", self.base))
    }
}

#[derive(Serialize)]
pub struct Client_Addons_Samlp_Logout {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    callback: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    slo_enabled: std::option::Option<terrars::PrimField<bool>>,
}

impl Client_Addons_Samlp_Logout {
    #[doc = "Set the field `callback`.\nThe service provider (client application)'s Single Logout Service URL, where Auth0 will send logout requests and responses."]
    pub fn set_callback(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.callback = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `slo_enabled`.\nControls whether Auth0 should notify service providers of session termination."]
    pub fn set_slo_enabled(mut self, v: impl std::convert::Into<terrars::PrimField<bool>>) -> Self {
        self.slo_enabled = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for Client_Addons_Samlp_Logout {
    type O = terrars::BlockAssignable<Client_Addons_Samlp_Logout>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_Addons_Samlp_Logout {}

impl BuildClient_Addons_Samlp_Logout {
    pub fn build(self) -> Client_Addons_Samlp_Logout {
        Client_Addons_Samlp_Logout {
            callback: core::default::Default::default(),
            slo_enabled: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_Addons_Samlp_LogoutRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_Addons_Samlp_LogoutRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> Client_Addons_Samlp_LogoutRef {
        Client_Addons_Samlp_LogoutRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_Addons_Samlp_LogoutRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `callback` after provisioning.\nThe service provider (client application)'s Single Logout Service URL, where Auth0 will send logout requests and responses."]
    pub fn callback(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.callback", self.base))
    }

    #[doc = "Get a reference to the value of field `slo_enabled` after provisioning.\nControls whether Auth0 should notify service providers of session termination."]
    pub fn slo_enabled(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.slo_enabled", self.base))
    }
}

#[derive(Serialize, Default)]
struct Client_Addons_SamlpDynamic {
    logout: std::option::Option<terrars::DynamicBlock<Client_Addons_Samlp_Logout>>,
}

#[derive(Serialize)]
pub struct Client_Addons_Samlp {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    audience: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    authn_context_class_ref: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    binding: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    create_upn_claim: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    destination: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    digest_algorithm: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    flexible_mappings: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    include_attribute_name_format: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    issuer: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    lifetime_in_seconds: std::option::Option<terrars::PrimField<f64>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    map_identities: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    map_unknown_claims_as_is: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    mappings: std::option::Option<terrars::RecField<terrars::PrimField<std::string::String>>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    name_identifier_format: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    name_identifier_probes:
        std::option::Option<terrars::ListField<terrars::PrimField<std::string::String>>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    passthrough_claims_with_no_mapping: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    recipient: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    sign_response: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    signature_algorithm: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    signing_cert: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    typed_attributes: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    logout: std::option::Option<std::vec::Vec<Client_Addons_Samlp_Logout>>,
    dynamic: Client_Addons_SamlpDynamic,
}

impl Client_Addons_Samlp {
    #[doc = "Set the field `audience`.\nAudience of the SAML Assertion. Default will be the Issuer on SAMLRequest."]
    pub fn set_audience(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.audience = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `authn_context_class_ref`.\nClass reference of the authentication context."]
    pub fn set_authn_context_class_ref(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.authn_context_class_ref = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `binding`.\nProtocol binding used for SAML logout responses."]
    pub fn set_binding(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.binding = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `create_upn_claim`.\nIndicates whether a UPN claim should be created."]
    pub fn set_create_upn_claim(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.create_upn_claim = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `destination`.\nDestination of the SAML Response. If not specified, it will be `AssertionConsumerUrl` of SAMLRequest or callback URL if there was no SAMLRequest."]
    pub fn set_destination(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.destination = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `digest_algorithm`.\nAlgorithm used to calculate the digest of the SAML Assertion or response. Options include `sha1` and `sha256`."]
    pub fn set_digest_algorithm(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.digest_algorithm = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `flexible_mappings`.\nThis is a supporting attribute to `mappings` field.Please note this is an experimental field. It should only be used when needed to send a map with keys as slices."]
    pub fn set_flexible_mappings(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.flexible_mappings = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `include_attribute_name_format`.\nIndicates whether or not we should infer the NameFormat based on the attribute name. If set to `false`, the attribute NameFormat is not set in the assertion."]
    pub fn set_include_attribute_name_format(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.include_attribute_name_format = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `issuer`.\nIssuer of the SAML Assertion."]
    pub fn set_issuer(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.issuer = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `lifetime_in_seconds`.\nNumber of seconds during which the token is valid."]
    pub fn set_lifetime_in_seconds(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<f64>>,
    ) -> Self {
        self.lifetime_in_seconds = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `map_identities`.\nIndicates whether or not to add additional identity information in the token, such as the provider used and the `access_token`, if available."]
    pub fn set_map_identities(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.map_identities = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `map_unknown_claims_as_is`.\nIndicates whether to add a prefix of `http://schema.auth0.com` to any claims that are not mapped to the common profile when passed through in the output assertion."]
    pub fn set_map_unknown_claims_as_is(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.map_unknown_claims_as_is = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `mappings`.\nMappings between the Auth0 user profile property name (`name`) and the output attributes on the SAML attribute in the assertion (`value`)."]
    pub fn set_mappings(
        mut self,
        v: impl std::convert::Into<terrars::RecField<terrars::PrimField<std::string::String>>>,
    ) -> Self {
        self.mappings = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `name_identifier_format`.\nFormat of the name identifier."]
    pub fn set_name_identifier_format(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.name_identifier_format = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `name_identifier_probes`.\nAttributes that can be used for Subject/NameID. Auth0 will try each of the attributes of this array in order and use the first value it finds."]
    pub fn set_name_identifier_probes(
        mut self,
        v: impl std::convert::Into<terrars::ListField<terrars::PrimField<std::string::String>>>,
    ) -> Self {
        self.name_identifier_probes = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `passthrough_claims_with_no_mapping`.\nIndicates whether or not to passthrough claims that are not mapped to the common profile in the output assertion."]
    pub fn set_passthrough_claims_with_no_mapping(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.passthrough_claims_with_no_mapping = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `recipient`.\nRecipient of the SAML Assertion (SubjectConfirmationData). Default is `AssertionConsumerUrl` on SAMLRequest or callback URL if no SAMLRequest was sent."]
    pub fn set_recipient(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.recipient = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `sign_response`.\nIndicates whether or not the SAML Response should be signed instead of the SAML Assertion."]
    pub fn set_sign_response(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.sign_response = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `signature_algorithm`.\nAlgorithm used to sign the SAML Assertion or response. Options include `rsa-sha1` and `rsa-sha256`."]
    pub fn set_signature_algorithm(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.signature_algorithm = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `signing_cert`.\nOptionally indicates the public key certificate used to validate SAML requests. If set, SAML requests will be required to be signed. A sample value would be `-----BEGIN PUBLIC KEY-----\\nMIGf...bpP/t3\\n+JGNGIRMj1hF1rnb6QIDAQAB\\n-----END PUBLIC KEY-----\\n`."]
    pub fn set_signing_cert(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.signing_cert = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `typed_attributes`.\nIndicates whether or not we should infer the `xs:type` of the element. Types include `xs:string`, `xs:boolean`, `xs:double`, and `xs:anyType`. When set to `false`, all `xs:type` are `xs:anyType`."]
    pub fn set_typed_attributes(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.typed_attributes = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `logout`.\n"]
    pub fn set_logout(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_Addons_Samlp_Logout>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.logout = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.logout = std::option::Option::Some(d);
            }
        }
        self
    }
}

impl terrars::ToListMappable for Client_Addons_Samlp {
    type O = terrars::BlockAssignable<Client_Addons_Samlp>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_Addons_Samlp {}

impl BuildClient_Addons_Samlp {
    pub fn build(self) -> Client_Addons_Samlp {
        Client_Addons_Samlp {
            audience: core::default::Default::default(),
            authn_context_class_ref: core::default::Default::default(),
            binding: core::default::Default::default(),
            create_upn_claim: core::default::Default::default(),
            destination: core::default::Default::default(),
            digest_algorithm: core::default::Default::default(),
            flexible_mappings: core::default::Default::default(),
            include_attribute_name_format: core::default::Default::default(),
            issuer: core::default::Default::default(),
            lifetime_in_seconds: core::default::Default::default(),
            map_identities: core::default::Default::default(),
            map_unknown_claims_as_is: core::default::Default::default(),
            mappings: core::default::Default::default(),
            name_identifier_format: core::default::Default::default(),
            name_identifier_probes: core::default::Default::default(),
            passthrough_claims_with_no_mapping: core::default::Default::default(),
            recipient: core::default::Default::default(),
            sign_response: core::default::Default::default(),
            signature_algorithm: core::default::Default::default(),
            signing_cert: core::default::Default::default(),
            typed_attributes: core::default::Default::default(),
            logout: core::default::Default::default(),
            dynamic: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_Addons_SamlpRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_Addons_SamlpRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_Addons_SamlpRef {
        Client_Addons_SamlpRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_Addons_SamlpRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `audience` after provisioning.\nAudience of the SAML Assertion. Default will be the Issuer on SAMLRequest."]
    pub fn audience(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.audience", self.base))
    }

    #[doc = "Get a reference to the value of field `authn_context_class_ref` after provisioning.\nClass reference of the authentication context."]
    pub fn authn_context_class_ref(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.authn_context_class_ref", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `binding` after provisioning.\nProtocol binding used for SAML logout responses."]
    pub fn binding(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.binding", self.base))
    }

    #[doc = "Get a reference to the value of field `create_upn_claim` after provisioning.\nIndicates whether a UPN claim should be created."]
    pub fn create_upn_claim(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.create_upn_claim", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `destination` after provisioning.\nDestination of the SAML Response. If not specified, it will be `AssertionConsumerUrl` of SAMLRequest or callback URL if there was no SAMLRequest."]
    pub fn destination(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.destination", self.base))
    }

    #[doc = "Get a reference to the value of field `digest_algorithm` after provisioning.\nAlgorithm used to calculate the digest of the SAML Assertion or response. Options include `sha1` and `sha256`."]
    pub fn digest_algorithm(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.digest_algorithm", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `flexible_mappings` after provisioning.\nThis is a supporting attribute to `mappings` field.Please note this is an experimental field. It should only be used when needed to send a map with keys as slices."]
    pub fn flexible_mappings(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.flexible_mappings", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `include_attribute_name_format` after provisioning.\nIndicates whether or not we should infer the NameFormat based on the attribute name. If set to `false`, the attribute NameFormat is not set in the assertion."]
    pub fn include_attribute_name_format(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.include_attribute_name_format", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `issuer` after provisioning.\nIssuer of the SAML Assertion."]
    pub fn issuer(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.issuer", self.base))
    }

    #[doc = "Get a reference to the value of field `lifetime_in_seconds` after provisioning.\nNumber of seconds during which the token is valid."]
    pub fn lifetime_in_seconds(&self) -> terrars::PrimExpr<f64> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.lifetime_in_seconds", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `map_identities` after provisioning.\nIndicates whether or not to add additional identity information in the token, such as the provider used and the `access_token`, if available."]
    pub fn map_identities(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.map_identities", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `map_unknown_claims_as_is` after provisioning.\nIndicates whether to add a prefix of `http://schema.auth0.com` to any claims that are not mapped to the common profile when passed through in the output assertion."]
    pub fn map_unknown_claims_as_is(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.map_unknown_claims_as_is", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `mappings` after provisioning.\nMappings between the Auth0 user profile property name (`name`) and the output attributes on the SAML attribute in the assertion (`value`)."]
    pub fn mappings(&self) -> terrars::RecRef<terrars::PrimExpr<std::string::String>> {
        terrars::RecRef::new(self.shared().clone(), format!("{}.mappings", self.base))
    }

    #[doc = "Get a reference to the value of field `name_identifier_format` after provisioning.\nFormat of the name identifier."]
    pub fn name_identifier_format(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.name_identifier_format", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `name_identifier_probes` after provisioning.\nAttributes that can be used for Subject/NameID. Auth0 will try each of the attributes of this array in order and use the first value it finds."]
    pub fn name_identifier_probes(
        &self,
    ) -> terrars::ListRef<terrars::PrimExpr<std::string::String>> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.name_identifier_probes", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `passthrough_claims_with_no_mapping` after provisioning.\nIndicates whether or not to passthrough claims that are not mapped to the common profile in the output assertion."]
    pub fn passthrough_claims_with_no_mapping(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.passthrough_claims_with_no_mapping", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `recipient` after provisioning.\nRecipient of the SAML Assertion (SubjectConfirmationData). Default is `AssertionConsumerUrl` on SAMLRequest or callback URL if no SAMLRequest was sent."]
    pub fn recipient(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.recipient", self.base))
    }

    #[doc = "Get a reference to the value of field `sign_response` after provisioning.\nIndicates whether or not the SAML Response should be signed instead of the SAML Assertion."]
    pub fn sign_response(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.sign_response", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `signature_algorithm` after provisioning.\nAlgorithm used to sign the SAML Assertion or response. Options include `rsa-sha1` and `rsa-sha256`."]
    pub fn signature_algorithm(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.signature_algorithm", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `signing_cert` after provisioning.\nOptionally indicates the public key certificate used to validate SAML requests. If set, SAML requests will be required to be signed. A sample value would be `-----BEGIN PUBLIC KEY-----\\nMIGf...bpP/t3\\n+JGNGIRMj1hF1rnb6QIDAQAB\\n-----END PUBLIC KEY-----\\n`."]
    pub fn signing_cert(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.signing_cert", self.base))
    }

    #[doc = "Get a reference to the value of field `typed_attributes` after provisioning.\nIndicates whether or not we should infer the `xs:type` of the element. Types include `xs:string`, `xs:boolean`, `xs:double`, and `xs:anyType`. When set to `false`, all `xs:type` are `xs:anyType`."]
    pub fn typed_attributes(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.typed_attributes", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `logout` after provisioning.\n"]
    pub fn logout(&self) -> terrars::ListRef<Client_Addons_Samlp_LogoutRef> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.logout", self.base))
    }
}

#[derive(Serialize)]
pub struct Client_Addons_SapApi {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    client_id: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    name_identifier_format: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    scope: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    service_password: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    token_endpoint_url: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    username_attribute: std::option::Option<terrars::PrimField<std::string::String>>,
}

impl Client_Addons_SapApi {
    #[doc = "Set the field `client_id`.\nIf activated in the OAuth 2.0 client configuration (transaction `SOAUTH2) the SAML attribute `client_id` must be set and equal the `client_id` form parameter of the access token request."]
    pub fn set_client_id(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.client_id = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `name_identifier_format`.\nNameID element of the Subject which can be used to express the user's identity. Defaults to `urn:oasis:names:tc:SAML:1.1:nameid-format:unspecified`."]
    pub fn set_name_identifier_format(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.name_identifier_format = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `scope`.\nRequested scope for SAP APIs."]
    pub fn set_scope(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.scope = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `service_password`.\nService account password to use to authenticate API calls to the token endpoint."]
    pub fn set_service_password(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.service_password = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `token_endpoint_url`.\nThe OAuth2 token endpoint URL of your SAP OData server."]
    pub fn set_token_endpoint_url(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.token_endpoint_url = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `username_attribute`.\nName of the property in the user object that maps to a SAP username, for example `email`."]
    pub fn set_username_attribute(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.username_attribute = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for Client_Addons_SapApi {
    type O = terrars::BlockAssignable<Client_Addons_SapApi>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_Addons_SapApi {}

impl BuildClient_Addons_SapApi {
    pub fn build(self) -> Client_Addons_SapApi {
        Client_Addons_SapApi {
            client_id: core::default::Default::default(),
            name_identifier_format: core::default::Default::default(),
            scope: core::default::Default::default(),
            service_password: core::default::Default::default(),
            token_endpoint_url: core::default::Default::default(),
            username_attribute: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_Addons_SapApiRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_Addons_SapApiRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_Addons_SapApiRef {
        Client_Addons_SapApiRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_Addons_SapApiRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `client_id` after provisioning.\nIf activated in the OAuth 2.0 client configuration (transaction `SOAUTH2) the SAML attribute `client_id` must be set and equal the `client_id` form parameter of the access token request."]
    pub fn client_id(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.client_id", self.base))
    }

    #[doc = "Get a reference to the value of field `name_identifier_format` after provisioning.\nNameID element of the Subject which can be used to express the user's identity. Defaults to `urn:oasis:names:tc:SAML:1.1:nameid-format:unspecified`."]
    pub fn name_identifier_format(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.name_identifier_format", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `scope` after provisioning.\nRequested scope for SAP APIs."]
    pub fn scope(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.scope", self.base))
    }

    #[doc = "Get a reference to the value of field `service_password` after provisioning.\nService account password to use to authenticate API calls to the token endpoint."]
    pub fn service_password(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.service_password", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `token_endpoint_url` after provisioning.\nThe OAuth2 token endpoint URL of your SAP OData server."]
    pub fn token_endpoint_url(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.token_endpoint_url", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `username_attribute` after provisioning.\nName of the property in the user object that maps to a SAP username, for example `email`."]
    pub fn username_attribute(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.username_attribute", self.base),
        )
    }
}

#[derive(Serialize)]
pub struct Client_Addons_Sentry {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    base_url: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    org_slug: std::option::Option<terrars::PrimField<std::string::String>>,
}

impl Client_Addons_Sentry {
    #[doc = "Set the field `base_url`.\nURL prefix only if running Sentry Community Edition, otherwise leave empty."]
    pub fn set_base_url(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.base_url = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `org_slug`.\nGenerated slug for your Sentry organization. Found in your Sentry URL, for example `https://sentry.acme.com/acme-org/` would be `acme-org`."]
    pub fn set_org_slug(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.org_slug = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for Client_Addons_Sentry {
    type O = terrars::BlockAssignable<Client_Addons_Sentry>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_Addons_Sentry {}

impl BuildClient_Addons_Sentry {
    pub fn build(self) -> Client_Addons_Sentry {
        Client_Addons_Sentry {
            base_url: core::default::Default::default(),
            org_slug: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_Addons_SentryRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_Addons_SentryRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_Addons_SentryRef {
        Client_Addons_SentryRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_Addons_SentryRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `base_url` after provisioning.\nURL prefix only if running Sentry Community Edition, otherwise leave empty."]
    pub fn base_url(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.base_url", self.base))
    }

    #[doc = "Get a reference to the value of field `org_slug` after provisioning.\nGenerated slug for your Sentry organization. Found in your Sentry URL, for example `https://sentry.acme.com/acme-org/` would be `acme-org`."]
    pub fn org_slug(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.org_slug", self.base))
    }
}

#[derive(Serialize)]
pub struct Client_Addons_Sharepoint {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    external_url: std::option::Option<terrars::ListField<terrars::PrimField<std::string::String>>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    url: std::option::Option<terrars::PrimField<std::string::String>>,
}

impl Client_Addons_Sharepoint {
    #[doc = "Set the field `external_url`.\nExternal SharePoint application URLs if exposed to the Internet."]
    pub fn set_external_url(
        mut self,
        v: impl std::convert::Into<terrars::ListField<terrars::PrimField<std::string::String>>>,
    ) -> Self {
        self.external_url = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `url`.\nInternal SharePoint application URL."]
    pub fn set_url(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.url = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for Client_Addons_Sharepoint {
    type O = terrars::BlockAssignable<Client_Addons_Sharepoint>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_Addons_Sharepoint {}

impl BuildClient_Addons_Sharepoint {
    pub fn build(self) -> Client_Addons_Sharepoint {
        Client_Addons_Sharepoint {
            external_url: core::default::Default::default(),
            url: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_Addons_SharepointRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_Addons_SharepointRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_Addons_SharepointRef {
        Client_Addons_SharepointRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_Addons_SharepointRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `external_url` after provisioning.\nExternal SharePoint application URLs if exposed to the Internet."]
    pub fn external_url(&self) -> terrars::ListRef<terrars::PrimExpr<std::string::String>> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.external_url", self.base))
    }

    #[doc = "Get a reference to the value of field `url` after provisioning.\nInternal SharePoint application URL."]
    pub fn url(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.url", self.base))
    }
}

#[derive(Serialize)]
pub struct Client_Addons_Slack {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    team: std::option::Option<terrars::PrimField<std::string::String>>,
}

impl Client_Addons_Slack {
    #[doc = "Set the field `team`.\nSlack team name."]
    pub fn set_team(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.team = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for Client_Addons_Slack {
    type O = terrars::BlockAssignable<Client_Addons_Slack>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_Addons_Slack {}

impl BuildClient_Addons_Slack {
    pub fn build(self) -> Client_Addons_Slack {
        Client_Addons_Slack {
            team: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_Addons_SlackRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_Addons_SlackRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_Addons_SlackRef {
        Client_Addons_SlackRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_Addons_SlackRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `team` after provisioning.\nSlack team name."]
    pub fn team(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.team", self.base))
    }
}

#[derive(Serialize)]
pub struct Client_Addons_Springcm {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    acs_url: std::option::Option<terrars::PrimField<std::string::String>>,
}

impl Client_Addons_Springcm {
    #[doc = "Set the field `acs_url`.\nSpringCM ACS URL, for example `https://na11.springcm.com/atlas/sso/SSOEndpoint.ashx`."]
    pub fn set_acs_url(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.acs_url = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for Client_Addons_Springcm {
    type O = terrars::BlockAssignable<Client_Addons_Springcm>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_Addons_Springcm {}

impl BuildClient_Addons_Springcm {
    pub fn build(self) -> Client_Addons_Springcm {
        Client_Addons_Springcm {
            acs_url: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_Addons_SpringcmRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_Addons_SpringcmRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_Addons_SpringcmRef {
        Client_Addons_SpringcmRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_Addons_SpringcmRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `acs_url` after provisioning.\nSpringCM ACS URL, for example `https://na11.springcm.com/atlas/sso/SSOEndpoint.ashx`."]
    pub fn acs_url(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.acs_url", self.base))
    }
}

#[derive(Serialize)]
pub struct Client_Addons_SsoIntegration {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    name: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    version: std::option::Option<terrars::PrimField<std::string::String>>,
}

impl Client_Addons_SsoIntegration {
    #[doc = "Set the field `name`.\nSSO integration name."]
    pub fn set_name(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.name = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `version`.\nSSO integration version installed."]
    pub fn set_version(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.version = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for Client_Addons_SsoIntegration {
    type O = terrars::BlockAssignable<Client_Addons_SsoIntegration>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_Addons_SsoIntegration {}

impl BuildClient_Addons_SsoIntegration {
    pub fn build(self) -> Client_Addons_SsoIntegration {
        Client_Addons_SsoIntegration {
            name: core::default::Default::default(),
            version: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_Addons_SsoIntegrationRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_Addons_SsoIntegrationRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> Client_Addons_SsoIntegrationRef {
        Client_Addons_SsoIntegrationRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_Addons_SsoIntegrationRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `name` after provisioning.\nSSO integration name."]
    pub fn name(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.name", self.base))
    }

    #[doc = "Get a reference to the value of field `version` after provisioning.\nSSO integration version installed."]
    pub fn version(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.version", self.base))
    }
}

#[derive(Serialize)]
pub struct Client_Addons_Wams {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    master_key: std::option::Option<terrars::PrimField<std::string::String>>,
}

impl Client_Addons_Wams {
    #[doc = "Set the field `master_key`.\nYour master key for Windows Azure Mobile Services."]
    pub fn set_master_key(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.master_key = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for Client_Addons_Wams {
    type O = terrars::BlockAssignable<Client_Addons_Wams>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_Addons_Wams {}

impl BuildClient_Addons_Wams {
    pub fn build(self) -> Client_Addons_Wams {
        Client_Addons_Wams {
            master_key: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_Addons_WamsRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_Addons_WamsRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_Addons_WamsRef {
        Client_Addons_WamsRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_Addons_WamsRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `master_key` after provisioning.\nYour master key for Windows Azure Mobile Services."]
    pub fn master_key(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.master_key", self.base))
    }
}

#[derive(Serialize)]
pub struct Client_Addons_Wsfed {}

impl Client_Addons_Wsfed {}

impl terrars::ToListMappable for Client_Addons_Wsfed {
    type O = terrars::BlockAssignable<Client_Addons_Wsfed>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_Addons_Wsfed {}

impl BuildClient_Addons_Wsfed {
    pub fn build(self) -> Client_Addons_Wsfed {
        Client_Addons_Wsfed {}
    }
}

#[allow(dead_code)]
pub struct Client_Addons_WsfedRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_Addons_WsfedRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_Addons_WsfedRef {
        Client_Addons_WsfedRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_Addons_WsfedRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }
}

#[derive(Serialize)]
pub struct Client_Addons_Zendesk {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    account_name: std::option::Option<terrars::PrimField<std::string::String>>,
}

impl Client_Addons_Zendesk {
    #[doc = "Set the field `account_name`.\nZendesk account name. Usually the first segment in your Zendesk URL, for example `https://acme-org.zendesk.com` would be `acme-org`."]
    pub fn set_account_name(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.account_name = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for Client_Addons_Zendesk {
    type O = terrars::BlockAssignable<Client_Addons_Zendesk>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_Addons_Zendesk {}

impl BuildClient_Addons_Zendesk {
    pub fn build(self) -> Client_Addons_Zendesk {
        Client_Addons_Zendesk {
            account_name: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_Addons_ZendeskRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_Addons_ZendeskRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_Addons_ZendeskRef {
        Client_Addons_ZendeskRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_Addons_ZendeskRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `account_name` after provisioning.\nZendesk account name. Usually the first segment in your Zendesk URL, for example `https://acme-org.zendesk.com` would be `acme-org`."]
    pub fn account_name(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.account_name", self.base))
    }
}

#[derive(Serialize)]
pub struct Client_Addons_Zoom {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    account: std::option::Option<terrars::PrimField<std::string::String>>,
}

impl Client_Addons_Zoom {
    #[doc = "Set the field `account`.\nZoom account name. Usually the first segment of your Zoom URL, for example `https://acme-org.zoom.us` would be `acme-org`."]
    pub fn set_account(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.account = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for Client_Addons_Zoom {
    type O = terrars::BlockAssignable<Client_Addons_Zoom>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_Addons_Zoom {}

impl BuildClient_Addons_Zoom {
    pub fn build(self) -> Client_Addons_Zoom {
        Client_Addons_Zoom {
            account: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_Addons_ZoomRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_Addons_ZoomRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_Addons_ZoomRef {
        Client_Addons_ZoomRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_Addons_ZoomRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `account` after provisioning.\nZoom account name. Usually the first segment of your Zoom URL, for example `https://acme-org.zoom.us` would be `acme-org`."]
    pub fn account(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.account", self.base))
    }
}

#[derive(Serialize, Default)]
struct Client_AddonsDynamic {
    aws: std::option::Option<terrars::DynamicBlock<Client_Addons_Aws>>,
    azure_blob: std::option::Option<terrars::DynamicBlock<Client_Addons_AzureBlob>>,
    azure_sb: std::option::Option<terrars::DynamicBlock<Client_Addons_AzureSb>>,
    box_: std::option::Option<terrars::DynamicBlock<Client_Addons_Box>>,
    cloudbees: std::option::Option<terrars::DynamicBlock<Client_Addons_Cloudbees>>,
    concur: std::option::Option<terrars::DynamicBlock<Client_Addons_Concur>>,
    dropbox: std::option::Option<terrars::DynamicBlock<Client_Addons_Dropbox>>,
    echosign: std::option::Option<terrars::DynamicBlock<Client_Addons_Echosign>>,
    egnyte: std::option::Option<terrars::DynamicBlock<Client_Addons_Egnyte>>,
    firebase: std::option::Option<terrars::DynamicBlock<Client_Addons_Firebase>>,
    layer: std::option::Option<terrars::DynamicBlock<Client_Addons_Layer>>,
    mscrm: std::option::Option<terrars::DynamicBlock<Client_Addons_Mscrm>>,
    newrelic: std::option::Option<terrars::DynamicBlock<Client_Addons_Newrelic>>,
    office365: std::option::Option<terrars::DynamicBlock<Client_Addons_Office365>>,
    rms: std::option::Option<terrars::DynamicBlock<Client_Addons_Rms>>,
    salesforce: std::option::Option<terrars::DynamicBlock<Client_Addons_Salesforce>>,
    salesforce_api: std::option::Option<terrars::DynamicBlock<Client_Addons_SalesforceApi>>,
    salesforce_sandbox_api:
        std::option::Option<terrars::DynamicBlock<Client_Addons_SalesforceSandboxApi>>,
    samlp: std::option::Option<terrars::DynamicBlock<Client_Addons_Samlp>>,
    sap_api: std::option::Option<terrars::DynamicBlock<Client_Addons_SapApi>>,
    sentry: std::option::Option<terrars::DynamicBlock<Client_Addons_Sentry>>,
    sharepoint: std::option::Option<terrars::DynamicBlock<Client_Addons_Sharepoint>>,
    slack: std::option::Option<terrars::DynamicBlock<Client_Addons_Slack>>,
    springcm: std::option::Option<terrars::DynamicBlock<Client_Addons_Springcm>>,
    sso_integration: std::option::Option<terrars::DynamicBlock<Client_Addons_SsoIntegration>>,
    wams: std::option::Option<terrars::DynamicBlock<Client_Addons_Wams>>,
    wsfed: std::option::Option<terrars::DynamicBlock<Client_Addons_Wsfed>>,
    zendesk: std::option::Option<terrars::DynamicBlock<Client_Addons_Zendesk>>,
    zoom: std::option::Option<terrars::DynamicBlock<Client_Addons_Zoom>>,
}

#[derive(Serialize)]
pub struct Client_Addons {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    aws: std::option::Option<std::vec::Vec<Client_Addons_Aws>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    azure_blob: std::option::Option<std::vec::Vec<Client_Addons_AzureBlob>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    azure_sb: std::option::Option<std::vec::Vec<Client_Addons_AzureSb>>,
    #[serde(rename = "box", skip_serializing_if = "std::option::Option::is_none")]
    box_: std::option::Option<std::vec::Vec<Client_Addons_Box>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    cloudbees: std::option::Option<std::vec::Vec<Client_Addons_Cloudbees>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    concur: std::option::Option<std::vec::Vec<Client_Addons_Concur>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    dropbox: std::option::Option<std::vec::Vec<Client_Addons_Dropbox>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    echosign: std::option::Option<std::vec::Vec<Client_Addons_Echosign>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    egnyte: std::option::Option<std::vec::Vec<Client_Addons_Egnyte>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    firebase: std::option::Option<std::vec::Vec<Client_Addons_Firebase>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    layer: std::option::Option<std::vec::Vec<Client_Addons_Layer>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    mscrm: std::option::Option<std::vec::Vec<Client_Addons_Mscrm>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    newrelic: std::option::Option<std::vec::Vec<Client_Addons_Newrelic>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    office365: std::option::Option<std::vec::Vec<Client_Addons_Office365>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    rms: std::option::Option<std::vec::Vec<Client_Addons_Rms>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    salesforce: std::option::Option<std::vec::Vec<Client_Addons_Salesforce>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    salesforce_api: std::option::Option<std::vec::Vec<Client_Addons_SalesforceApi>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    salesforce_sandbox_api: std::option::Option<std::vec::Vec<Client_Addons_SalesforceSandboxApi>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    samlp: std::option::Option<std::vec::Vec<Client_Addons_Samlp>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    sap_api: std::option::Option<std::vec::Vec<Client_Addons_SapApi>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    sentry: std::option::Option<std::vec::Vec<Client_Addons_Sentry>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    sharepoint: std::option::Option<std::vec::Vec<Client_Addons_Sharepoint>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    slack: std::option::Option<std::vec::Vec<Client_Addons_Slack>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    springcm: std::option::Option<std::vec::Vec<Client_Addons_Springcm>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    sso_integration: std::option::Option<std::vec::Vec<Client_Addons_SsoIntegration>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    wams: std::option::Option<std::vec::Vec<Client_Addons_Wams>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    wsfed: std::option::Option<std::vec::Vec<Client_Addons_Wsfed>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    zendesk: std::option::Option<std::vec::Vec<Client_Addons_Zendesk>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    zoom: std::option::Option<std::vec::Vec<Client_Addons_Zoom>>,
    dynamic: Client_AddonsDynamic,
}

impl Client_Addons {
    #[doc = "Set the field `aws`.\n"]
    pub fn set_aws(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_Addons_Aws>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.aws = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.aws = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `azure_blob`.\n"]
    pub fn set_azure_blob(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_Addons_AzureBlob>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.azure_blob = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.azure_blob = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `azure_sb`.\n"]
    pub fn set_azure_sb(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_Addons_AzureSb>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.azure_sb = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.azure_sb = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `box_`.\n"]
    pub fn set_box(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_Addons_Box>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.box_ = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.box_ = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `cloudbees`.\n"]
    pub fn set_cloudbees(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_Addons_Cloudbees>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.cloudbees = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.cloudbees = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `concur`.\n"]
    pub fn set_concur(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_Addons_Concur>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.concur = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.concur = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `dropbox`.\n"]
    pub fn set_dropbox(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_Addons_Dropbox>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.dropbox = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.dropbox = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `echosign`.\n"]
    pub fn set_echosign(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_Addons_Echosign>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.echosign = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.echosign = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `egnyte`.\n"]
    pub fn set_egnyte(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_Addons_Egnyte>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.egnyte = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.egnyte = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `firebase`.\n"]
    pub fn set_firebase(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_Addons_Firebase>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.firebase = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.firebase = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `layer`.\n"]
    pub fn set_layer(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_Addons_Layer>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.layer = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.layer = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `mscrm`.\n"]
    pub fn set_mscrm(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_Addons_Mscrm>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.mscrm = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.mscrm = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `newrelic`.\n"]
    pub fn set_newrelic(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_Addons_Newrelic>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.newrelic = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.newrelic = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `office365`.\n"]
    pub fn set_office365(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_Addons_Office365>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.office365 = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.office365 = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `rms`.\n"]
    pub fn set_rms(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_Addons_Rms>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.rms = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.rms = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `salesforce`.\n"]
    pub fn set_salesforce(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_Addons_Salesforce>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.salesforce = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.salesforce = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `salesforce_api`.\n"]
    pub fn set_salesforce_api(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_Addons_SalesforceApi>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.salesforce_api = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.salesforce_api = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `salesforce_sandbox_api`.\n"]
    pub fn set_salesforce_sandbox_api(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_Addons_SalesforceSandboxApi>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.salesforce_sandbox_api = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.salesforce_sandbox_api = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `samlp`.\n"]
    pub fn set_samlp(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_Addons_Samlp>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.samlp = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.samlp = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `sap_api`.\n"]
    pub fn set_sap_api(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_Addons_SapApi>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.sap_api = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.sap_api = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `sentry`.\n"]
    pub fn set_sentry(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_Addons_Sentry>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.sentry = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.sentry = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `sharepoint`.\n"]
    pub fn set_sharepoint(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_Addons_Sharepoint>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.sharepoint = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.sharepoint = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `slack`.\n"]
    pub fn set_slack(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_Addons_Slack>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.slack = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.slack = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `springcm`.\n"]
    pub fn set_springcm(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_Addons_Springcm>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.springcm = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.springcm = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `sso_integration`.\n"]
    pub fn set_sso_integration(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_Addons_SsoIntegration>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.sso_integration = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.sso_integration = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `wams`.\n"]
    pub fn set_wams(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_Addons_Wams>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.wams = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.wams = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `wsfed`.\n"]
    pub fn set_wsfed(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_Addons_Wsfed>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.wsfed = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.wsfed = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `zendesk`.\n"]
    pub fn set_zendesk(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_Addons_Zendesk>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.zendesk = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.zendesk = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `zoom`.\n"]
    pub fn set_zoom(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_Addons_Zoom>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.zoom = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.zoom = std::option::Option::Some(d);
            }
        }
        self
    }
}

impl terrars::ToListMappable for Client_Addons {
    type O = terrars::BlockAssignable<Client_Addons>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_Addons {}

impl BuildClient_Addons {
    pub fn build(self) -> Client_Addons {
        Client_Addons {
            aws: core::default::Default::default(),
            azure_blob: core::default::Default::default(),
            azure_sb: core::default::Default::default(),
            box_: core::default::Default::default(),
            cloudbees: core::default::Default::default(),
            concur: core::default::Default::default(),
            dropbox: core::default::Default::default(),
            echosign: core::default::Default::default(),
            egnyte: core::default::Default::default(),
            firebase: core::default::Default::default(),
            layer: core::default::Default::default(),
            mscrm: core::default::Default::default(),
            newrelic: core::default::Default::default(),
            office365: core::default::Default::default(),
            rms: core::default::Default::default(),
            salesforce: core::default::Default::default(),
            salesforce_api: core::default::Default::default(),
            salesforce_sandbox_api: core::default::Default::default(),
            samlp: core::default::Default::default(),
            sap_api: core::default::Default::default(),
            sentry: core::default::Default::default(),
            sharepoint: core::default::Default::default(),
            slack: core::default::Default::default(),
            springcm: core::default::Default::default(),
            sso_integration: core::default::Default::default(),
            wams: core::default::Default::default(),
            wsfed: core::default::Default::default(),
            zendesk: core::default::Default::default(),
            zoom: core::default::Default::default(),
            dynamic: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_AddonsRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_AddonsRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_AddonsRef {
        Client_AddonsRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_AddonsRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `aws` after provisioning.\n"]
    pub fn aws(&self) -> terrars::ListRef<Client_Addons_AwsRef> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.aws", self.base))
    }

    #[doc = "Get a reference to the value of field `azure_blob` after provisioning.\n"]
    pub fn azure_blob(&self) -> terrars::ListRef<Client_Addons_AzureBlobRef> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.azure_blob", self.base))
    }

    #[doc = "Get a reference to the value of field `azure_sb` after provisioning.\n"]
    pub fn azure_sb(&self) -> terrars::ListRef<Client_Addons_AzureSbRef> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.azure_sb", self.base))
    }

    #[doc = "Get a reference to the value of field `box_` after provisioning.\n"]
    pub fn box_(&self) -> terrars::ListRef<Client_Addons_BoxRef> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.box", self.base))
    }

    #[doc = "Get a reference to the value of field `cloudbees` after provisioning.\n"]
    pub fn cloudbees(&self) -> terrars::ListRef<Client_Addons_CloudbeesRef> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.cloudbees", self.base))
    }

    #[doc = "Get a reference to the value of field `concur` after provisioning.\n"]
    pub fn concur(&self) -> terrars::ListRef<Client_Addons_ConcurRef> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.concur", self.base))
    }

    #[doc = "Get a reference to the value of field `dropbox` after provisioning.\n"]
    pub fn dropbox(&self) -> terrars::ListRef<Client_Addons_DropboxRef> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.dropbox", self.base))
    }

    #[doc = "Get a reference to the value of field `echosign` after provisioning.\n"]
    pub fn echosign(&self) -> terrars::ListRef<Client_Addons_EchosignRef> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.echosign", self.base))
    }

    #[doc = "Get a reference to the value of field `egnyte` after provisioning.\n"]
    pub fn egnyte(&self) -> terrars::ListRef<Client_Addons_EgnyteRef> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.egnyte", self.base))
    }

    #[doc = "Get a reference to the value of field `firebase` after provisioning.\n"]
    pub fn firebase(&self) -> terrars::ListRef<Client_Addons_FirebaseRef> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.firebase", self.base))
    }

    #[doc = "Get a reference to the value of field `layer` after provisioning.\n"]
    pub fn layer(&self) -> terrars::ListRef<Client_Addons_LayerRef> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.layer", self.base))
    }

    #[doc = "Get a reference to the value of field `mscrm` after provisioning.\n"]
    pub fn mscrm(&self) -> terrars::ListRef<Client_Addons_MscrmRef> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.mscrm", self.base))
    }

    #[doc = "Get a reference to the value of field `newrelic` after provisioning.\n"]
    pub fn newrelic(&self) -> terrars::ListRef<Client_Addons_NewrelicRef> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.newrelic", self.base))
    }

    #[doc = "Get a reference to the value of field `office365` after provisioning.\n"]
    pub fn office365(&self) -> terrars::ListRef<Client_Addons_Office365Ref> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.office365", self.base))
    }

    #[doc = "Get a reference to the value of field `rms` after provisioning.\n"]
    pub fn rms(&self) -> terrars::ListRef<Client_Addons_RmsRef> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.rms", self.base))
    }

    #[doc = "Get a reference to the value of field `salesforce` after provisioning.\n"]
    pub fn salesforce(&self) -> terrars::ListRef<Client_Addons_SalesforceRef> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.salesforce", self.base))
    }

    #[doc = "Get a reference to the value of field `salesforce_api` after provisioning.\n"]
    pub fn salesforce_api(&self) -> terrars::ListRef<Client_Addons_SalesforceApiRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.salesforce_api", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `salesforce_sandbox_api` after provisioning.\n"]
    pub fn salesforce_sandbox_api(
        &self,
    ) -> terrars::ListRef<Client_Addons_SalesforceSandboxApiRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.salesforce_sandbox_api", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `samlp` after provisioning.\n"]
    pub fn samlp(&self) -> terrars::ListRef<Client_Addons_SamlpRef> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.samlp", self.base))
    }

    #[doc = "Get a reference to the value of field `sap_api` after provisioning.\n"]
    pub fn sap_api(&self) -> terrars::ListRef<Client_Addons_SapApiRef> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.sap_api", self.base))
    }

    #[doc = "Get a reference to the value of field `sentry` after provisioning.\n"]
    pub fn sentry(&self) -> terrars::ListRef<Client_Addons_SentryRef> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.sentry", self.base))
    }

    #[doc = "Get a reference to the value of field `sharepoint` after provisioning.\n"]
    pub fn sharepoint(&self) -> terrars::ListRef<Client_Addons_SharepointRef> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.sharepoint", self.base))
    }

    #[doc = "Get a reference to the value of field `slack` after provisioning.\n"]
    pub fn slack(&self) -> terrars::ListRef<Client_Addons_SlackRef> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.slack", self.base))
    }

    #[doc = "Get a reference to the value of field `springcm` after provisioning.\n"]
    pub fn springcm(&self) -> terrars::ListRef<Client_Addons_SpringcmRef> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.springcm", self.base))
    }

    #[doc = "Get a reference to the value of field `sso_integration` after provisioning.\n"]
    pub fn sso_integration(&self) -> terrars::ListRef<Client_Addons_SsoIntegrationRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.sso_integration", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `wams` after provisioning.\n"]
    pub fn wams(&self) -> terrars::ListRef<Client_Addons_WamsRef> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.wams", self.base))
    }

    #[doc = "Get a reference to the value of field `wsfed` after provisioning.\n"]
    pub fn wsfed(&self) -> terrars::ListRef<Client_Addons_WsfedRef> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.wsfed", self.base))
    }

    #[doc = "Get a reference to the value of field `zendesk` after provisioning.\n"]
    pub fn zendesk(&self) -> terrars::ListRef<Client_Addons_ZendeskRef> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.zendesk", self.base))
    }

    #[doc = "Get a reference to the value of field `zoom` after provisioning.\n"]
    pub fn zoom(&self) -> terrars::ListRef<Client_Addons_ZoomRef> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.zoom", self.base))
    }
}

#[derive(Serialize)]
pub struct Client_AnonymousSessions {
    active: terrars::PrimField<bool>,
}

impl Client_AnonymousSessions {}

impl terrars::ToListMappable for Client_AnonymousSessions {
    type O = terrars::BlockAssignable<Client_AnonymousSessions>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_AnonymousSessions {
    #[doc = "If set to true, this client is allowed to create anonymous sessions. Requires `oidc_conformant` to be set to `true`. Set to `false` to disable. (EA only)"]
    pub active: terrars::PrimField<bool>,
}

impl BuildClient_AnonymousSessions {
    pub fn build(self) -> Client_AnonymousSessions {
        Client_AnonymousSessions {
            active: self.active,
        }
    }
}

#[allow(dead_code)]
pub struct Client_AnonymousSessionsRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_AnonymousSessionsRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_AnonymousSessionsRef {
        Client_AnonymousSessionsRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_AnonymousSessionsRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `active` after provisioning.\nIf set to true, this client is allowed to create anonymous sessions. Requires `oidc_conformant` to be set to `true`. Set to `false` to disable. (EA only)"]
    pub fn active(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.active", self.base))
    }
}

#[derive(Serialize)]
pub struct Client_B2bIntegrationConfiguration {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    integration_type: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    sso_profiles: std::option::Option<terrars::ListField<terrars::PrimField<std::string::String>>>,
}

impl Client_B2bIntegrationConfiguration {
    #[doc = "Set the field `integration_type`.\nThe type of integration used to connect to this B2B integration client. One of custom_auth_server, third_party, application"]
    pub fn set_integration_type(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.integration_type = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `sso_profiles`.\nID of the self-service SSO profile (an `auth0_self_service_profile` id, in `ssp_...` format) linked to this B2B integration client. Maximum 1."]
    pub fn set_sso_profiles(
        mut self,
        v: impl std::convert::Into<terrars::ListField<terrars::PrimField<std::string::String>>>,
    ) -> Self {
        self.sso_profiles = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for Client_B2bIntegrationConfiguration {
    type O = terrars::BlockAssignable<Client_B2bIntegrationConfiguration>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_B2bIntegrationConfiguration {}

impl BuildClient_B2bIntegrationConfiguration {
    pub fn build(self) -> Client_B2bIntegrationConfiguration {
        Client_B2bIntegrationConfiguration {
            integration_type: core::default::Default::default(),
            sso_profiles: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_B2bIntegrationConfigurationRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_B2bIntegrationConfigurationRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> Client_B2bIntegrationConfigurationRef {
        Client_B2bIntegrationConfigurationRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_B2bIntegrationConfigurationRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `integration_type` after provisioning.\nThe type of integration used to connect to this B2B integration client. One of custom_auth_server, third_party, application"]
    pub fn integration_type(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.integration_type", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `sso_profiles` after provisioning.\nID of the self-service SSO profile (an `auth0_self_service_profile` id, in `ssp_...` format) linked to this B2B integration client. Maximum 1."]
    pub fn sso_profiles(&self) -> terrars::ListRef<terrars::PrimExpr<std::string::String>> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.sso_profiles", self.base))
    }
}

#[derive(Serialize)]
pub struct Client_DefaultOrganization {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    disable: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    flows: std::option::Option<terrars::ListField<terrars::PrimField<std::string::String>>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    organization_id: std::option::Option<terrars::PrimField<std::string::String>>,
}

impl Client_DefaultOrganization {
    #[doc = "Set the field `disable`.\nIf set, the `default_organization` will be removed."]
    pub fn set_disable(mut self, v: impl std::convert::Into<terrars::PrimField<bool>>) -> Self {
        self.disable = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `flows`.\nDefinition of the flow that needs to be configured. Eg. client_credentials"]
    pub fn set_flows(
        mut self,
        v: impl std::convert::Into<terrars::ListField<terrars::PrimField<std::string::String>>>,
    ) -> Self {
        self.flows = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `organization_id`.\nThe unique identifier of the organization"]
    pub fn set_organization_id(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.organization_id = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for Client_DefaultOrganization {
    type O = terrars::BlockAssignable<Client_DefaultOrganization>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_DefaultOrganization {}

impl BuildClient_DefaultOrganization {
    pub fn build(self) -> Client_DefaultOrganization {
        Client_DefaultOrganization {
            disable: core::default::Default::default(),
            flows: core::default::Default::default(),
            organization_id: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_DefaultOrganizationRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_DefaultOrganizationRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> Client_DefaultOrganizationRef {
        Client_DefaultOrganizationRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_DefaultOrganizationRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `disable` after provisioning.\nIf set, the `default_organization` will be removed."]
    pub fn disable(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.disable", self.base))
    }

    #[doc = "Get a reference to the value of field `flows` after provisioning.\nDefinition of the flow that needs to be configured. Eg. client_credentials"]
    pub fn flows(&self) -> terrars::ListRef<terrars::PrimExpr<std::string::String>> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.flows", self.base))
    }

    #[doc = "Get a reference to the value of field `organization_id` after provisioning.\nThe unique identifier of the organization"]
    pub fn organization_id(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.organization_id", self.base),
        )
    }
}

#[derive(Serialize)]
pub struct Client_ExpressConfiguration_LinkedClients {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    client_id: std::option::Option<terrars::PrimField<std::string::String>>,
}

impl Client_ExpressConfiguration_LinkedClients {
    #[doc = "Set the field `client_id`.\nThe ID of the linked client."]
    pub fn set_client_id(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.client_id = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for Client_ExpressConfiguration_LinkedClients {
    type O = terrars::BlockAssignable<Client_ExpressConfiguration_LinkedClients>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_ExpressConfiguration_LinkedClients {}

impl BuildClient_ExpressConfiguration_LinkedClients {
    pub fn build(self) -> Client_ExpressConfiguration_LinkedClients {
        Client_ExpressConfiguration_LinkedClients {
            client_id: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_ExpressConfiguration_LinkedClientsRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_ExpressConfiguration_LinkedClientsRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> Client_ExpressConfiguration_LinkedClientsRef {
        Client_ExpressConfiguration_LinkedClientsRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_ExpressConfiguration_LinkedClientsRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `client_id` after provisioning.\nThe ID of the linked client."]
    pub fn client_id(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.client_id", self.base))
    }
}

#[derive(Serialize, Default)]
struct Client_ExpressConfigurationDynamic {
    linked_clients:
        std::option::Option<terrars::DynamicBlock<Client_ExpressConfiguration_LinkedClients>>,
}

#[derive(Serialize)]
pub struct Client_ExpressConfiguration {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    admin_login_domain: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    connection_profile_id: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    enable_client: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    enable_organization: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    initiate_login_uri_template: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    okta_oin_client_id: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    user_attribute_profile_id: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    linked_clients: std::option::Option<std::vec::Vec<Client_ExpressConfiguration_LinkedClients>>,
    dynamic: Client_ExpressConfigurationDynamic,
}

impl Client_ExpressConfiguration {
    #[doc = "Set the field `admin_login_domain`.\nThe domain that admins are expected to log in via for authenticating for express configuration."]
    pub fn set_admin_login_domain(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.admin_login_domain = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `connection_profile_id`.\nThe ID of the connection profile to use for this application."]
    pub fn set_connection_profile_id(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.connection_profile_id = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `enable_client`.\nWhen true, all connections made via express configuration will be enabled for this application."]
    pub fn set_enable_client(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.enable_client = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `enable_organization`.\nWhen true, all connections made via express configuration will have the associated organization enabled."]
    pub fn set_enable_organization(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.enable_organization = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `initiate_login_uri_template`.\nThe URI users should bookmark to log in to this application. Variable substitution is permitted for: organization_name, organization_id, and connection_name."]
    pub fn set_initiate_login_uri_template(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.initiate_login_uri_template = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `okta_oin_client_id`.\nThe unique identifier for the Okta OIN Express Configuration Client."]
    pub fn set_okta_oin_client_id(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.okta_oin_client_id = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `user_attribute_profile_id`.\nThe ID of the user attribute profile to use for this application."]
    pub fn set_user_attribute_profile_id(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.user_attribute_profile_id = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `linked_clients`.\n"]
    pub fn set_linked_clients(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_ExpressConfiguration_LinkedClients>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.linked_clients = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.linked_clients = std::option::Option::Some(d);
            }
        }
        self
    }
}

impl terrars::ToListMappable for Client_ExpressConfiguration {
    type O = terrars::BlockAssignable<Client_ExpressConfiguration>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_ExpressConfiguration {}

impl BuildClient_ExpressConfiguration {
    pub fn build(self) -> Client_ExpressConfiguration {
        Client_ExpressConfiguration {
            admin_login_domain: core::default::Default::default(),
            connection_profile_id: core::default::Default::default(),
            enable_client: core::default::Default::default(),
            enable_organization: core::default::Default::default(),
            initiate_login_uri_template: core::default::Default::default(),
            okta_oin_client_id: core::default::Default::default(),
            user_attribute_profile_id: core::default::Default::default(),
            linked_clients: core::default::Default::default(),
            dynamic: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_ExpressConfigurationRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_ExpressConfigurationRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> Client_ExpressConfigurationRef {
        Client_ExpressConfigurationRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_ExpressConfigurationRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `admin_login_domain` after provisioning.\nThe domain that admins are expected to log in via for authenticating for express configuration."]
    pub fn admin_login_domain(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.admin_login_domain", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `connection_profile_id` after provisioning.\nThe ID of the connection profile to use for this application."]
    pub fn connection_profile_id(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.connection_profile_id", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `enable_client` after provisioning.\nWhen true, all connections made via express configuration will be enabled for this application."]
    pub fn enable_client(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.enable_client", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `enable_organization` after provisioning.\nWhen true, all connections made via express configuration will have the associated organization enabled."]
    pub fn enable_organization(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.enable_organization", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `initiate_login_uri_template` after provisioning.\nThe URI users should bookmark to log in to this application. Variable substitution is permitted for: organization_name, organization_id, and connection_name."]
    pub fn initiate_login_uri_template(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.initiate_login_uri_template", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `oin_submission_id` after provisioning.\nThe identifier of the published application in the OKTA OIN."]
    pub fn oin_submission_id(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.oin_submission_id", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `okta_oin_client_id` after provisioning.\nThe unique identifier for the Okta OIN Express Configuration Client."]
    pub fn okta_oin_client_id(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.okta_oin_client_id", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `user_attribute_profile_id` after provisioning.\nThe ID of the user attribute profile to use for this application."]
    pub fn user_attribute_profile_id(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.user_attribute_profile_id", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `linked_clients` after provisioning.\n"]
    pub fn linked_clients(&self) -> terrars::ListRef<Client_ExpressConfiguration_LinkedClientsRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.linked_clients", self.base),
        )
    }
}

#[derive(Serialize)]
pub struct Client_FedcmLogin_Google {
    is_enabled: terrars::PrimField<bool>,
}

impl Client_FedcmLogin_Google {}

impl terrars::ToListMappable for Client_FedcmLogin_Google {
    type O = terrars::BlockAssignable<Client_FedcmLogin_Google>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_FedcmLogin_Google {
    #[doc = "Whether to show the Google FedCM prompt on Login. (EA only)"]
    pub is_enabled: terrars::PrimField<bool>,
}

impl BuildClient_FedcmLogin_Google {
    pub fn build(self) -> Client_FedcmLogin_Google {
        Client_FedcmLogin_Google {
            is_enabled: self.is_enabled,
        }
    }
}

#[allow(dead_code)]
pub struct Client_FedcmLogin_GoogleRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_FedcmLogin_GoogleRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_FedcmLogin_GoogleRef {
        Client_FedcmLogin_GoogleRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_FedcmLogin_GoogleRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `is_enabled` after provisioning.\nWhether to show the Google FedCM prompt on Login. (EA only)"]
    pub fn is_enabled(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.is_enabled", self.base))
    }
}

#[derive(Serialize, Default)]
struct Client_FedcmLoginDynamic {
    google: std::option::Option<terrars::DynamicBlock<Client_FedcmLogin_Google>>,
}

#[derive(Serialize)]
pub struct Client_FedcmLogin {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    google: std::option::Option<std::vec::Vec<Client_FedcmLogin_Google>>,
    dynamic: Client_FedcmLoginDynamic,
}

impl Client_FedcmLogin {
    #[doc = "Set the field `google`.\n"]
    pub fn set_google(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_FedcmLogin_Google>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.google = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.google = std::option::Option::Some(d);
            }
        }
        self
    }
}

impl terrars::ToListMappable for Client_FedcmLogin {
    type O = terrars::BlockAssignable<Client_FedcmLogin>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_FedcmLogin {}

impl BuildClient_FedcmLogin {
    pub fn build(self) -> Client_FedcmLogin {
        Client_FedcmLogin {
            google: core::default::Default::default(),
            dynamic: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_FedcmLoginRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_FedcmLoginRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_FedcmLoginRef {
        Client_FedcmLoginRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_FedcmLoginRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `google` after provisioning.\n"]
    pub fn google(&self) -> terrars::ListRef<Client_FedcmLogin_GoogleRef> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.google", self.base))
    }
}

#[derive(Serialize)]
pub struct Client_IdentityAssertionAuthorizationGrant {
    active: terrars::PrimField<bool>,
}

impl Client_IdentityAssertionAuthorizationGrant {}

impl terrars::ToListMappable for Client_IdentityAssertionAuthorizationGrant {
    type O = terrars::BlockAssignable<Client_IdentityAssertionAuthorizationGrant>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_IdentityAssertionAuthorizationGrant {
    #[doc = "Whether the client can exchange ID-JAGs for access tokens. (EA only)"]
    pub active: terrars::PrimField<bool>,
}

impl BuildClient_IdentityAssertionAuthorizationGrant {
    pub fn build(self) -> Client_IdentityAssertionAuthorizationGrant {
        Client_IdentityAssertionAuthorizationGrant {
            active: self.active,
        }
    }
}

#[allow(dead_code)]
pub struct Client_IdentityAssertionAuthorizationGrantRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_IdentityAssertionAuthorizationGrantRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> Client_IdentityAssertionAuthorizationGrantRef {
        Client_IdentityAssertionAuthorizationGrantRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_IdentityAssertionAuthorizationGrantRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `active` after provisioning.\nWhether the client can exchange ID-JAGs for access tokens. (EA only)"]
    pub fn active(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.active", self.base))
    }
}

#[derive(Serialize)]
pub struct Client_JwtConfiguration {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    alg: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    lifetime_in_seconds: std::option::Option<terrars::PrimField<f64>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    scopes: std::option::Option<terrars::RecField<terrars::PrimField<std::string::String>>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    secret_encoded: std::option::Option<terrars::PrimField<bool>>,
}

impl Client_JwtConfiguration {
    #[doc = "Set the field `alg`.\nAlgorithm used to sign JWTs. Can be one of `HS256`, `RS256`, `PS256`."]
    pub fn set_alg(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.alg = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `lifetime_in_seconds`.\nNumber of seconds during which the JWT will be valid."]
    pub fn set_lifetime_in_seconds(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<f64>>,
    ) -> Self {
        self.lifetime_in_seconds = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `scopes`.\nPermissions (scopes) included in JWTs."]
    pub fn set_scopes(
        mut self,
        v: impl std::convert::Into<terrars::RecField<terrars::PrimField<std::string::String>>>,
    ) -> Self {
        self.scopes = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `secret_encoded`.\nIndicates whether the client secret is Base64-encoded."]
    pub fn set_secret_encoded(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.secret_encoded = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for Client_JwtConfiguration {
    type O = terrars::BlockAssignable<Client_JwtConfiguration>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_JwtConfiguration {}

impl BuildClient_JwtConfiguration {
    pub fn build(self) -> Client_JwtConfiguration {
        Client_JwtConfiguration {
            alg: core::default::Default::default(),
            lifetime_in_seconds: core::default::Default::default(),
            scopes: core::default::Default::default(),
            secret_encoded: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_JwtConfigurationRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_JwtConfigurationRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_JwtConfigurationRef {
        Client_JwtConfigurationRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_JwtConfigurationRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `alg` after provisioning.\nAlgorithm used to sign JWTs. Can be one of `HS256`, `RS256`, `PS256`."]
    pub fn alg(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.alg", self.base))
    }

    #[doc = "Get a reference to the value of field `lifetime_in_seconds` after provisioning.\nNumber of seconds during which the JWT will be valid."]
    pub fn lifetime_in_seconds(&self) -> terrars::PrimExpr<f64> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.lifetime_in_seconds", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `scopes` after provisioning.\nPermissions (scopes) included in JWTs."]
    pub fn scopes(&self) -> terrars::RecRef<terrars::PrimExpr<std::string::String>> {
        terrars::RecRef::new(self.shared().clone(), format!("{}.scopes", self.base))
    }

    #[doc = "Get a reference to the value of field `secret_encoded` after provisioning.\nIndicates whether the client secret is Base64-encoded."]
    pub fn secret_encoded(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.secret_encoded", self.base),
        )
    }
}

#[derive(Serialize)]
pub struct Client_Mobile_Android {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    app_package_name: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    sha256_cert_fingerprints:
        std::option::Option<terrars::ListField<terrars::PrimField<std::string::String>>>,
}

impl Client_Mobile_Android {
    #[doc = "Set the field `app_package_name`.\n"]
    pub fn set_app_package_name(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.app_package_name = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `sha256_cert_fingerprints`.\n"]
    pub fn set_sha256_cert_fingerprints(
        mut self,
        v: impl std::convert::Into<terrars::ListField<terrars::PrimField<std::string::String>>>,
    ) -> Self {
        self.sha256_cert_fingerprints = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for Client_Mobile_Android {
    type O = terrars::BlockAssignable<Client_Mobile_Android>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_Mobile_Android {}

impl BuildClient_Mobile_Android {
    pub fn build(self) -> Client_Mobile_Android {
        Client_Mobile_Android {
            app_package_name: core::default::Default::default(),
            sha256_cert_fingerprints: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_Mobile_AndroidRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_Mobile_AndroidRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_Mobile_AndroidRef {
        Client_Mobile_AndroidRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_Mobile_AndroidRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `app_package_name` after provisioning.\n"]
    pub fn app_package_name(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.app_package_name", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `sha256_cert_fingerprints` after provisioning.\n"]
    pub fn sha256_cert_fingerprints(
        &self,
    ) -> terrars::ListRef<terrars::PrimExpr<std::string::String>> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.sha256_cert_fingerprints", self.base),
        )
    }
}

#[derive(Serialize)]
pub struct Client_Mobile_Ios {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    app_bundle_identifier: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    team_id: std::option::Option<terrars::PrimField<std::string::String>>,
}

impl Client_Mobile_Ios {
    #[doc = "Set the field `app_bundle_identifier`.\n"]
    pub fn set_app_bundle_identifier(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.app_bundle_identifier = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `team_id`.\n"]
    pub fn set_team_id(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.team_id = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for Client_Mobile_Ios {
    type O = terrars::BlockAssignable<Client_Mobile_Ios>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_Mobile_Ios {}

impl BuildClient_Mobile_Ios {
    pub fn build(self) -> Client_Mobile_Ios {
        Client_Mobile_Ios {
            app_bundle_identifier: core::default::Default::default(),
            team_id: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_Mobile_IosRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_Mobile_IosRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_Mobile_IosRef {
        Client_Mobile_IosRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_Mobile_IosRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `app_bundle_identifier` after provisioning.\n"]
    pub fn app_bundle_identifier(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.app_bundle_identifier", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `team_id` after provisioning.\n"]
    pub fn team_id(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.team_id", self.base))
    }
}

#[derive(Serialize, Default)]
struct Client_MobileDynamic {
    android: std::option::Option<terrars::DynamicBlock<Client_Mobile_Android>>,
    ios: std::option::Option<terrars::DynamicBlock<Client_Mobile_Ios>>,
}

#[derive(Serialize)]
pub struct Client_Mobile {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    android: std::option::Option<std::vec::Vec<Client_Mobile_Android>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    ios: std::option::Option<std::vec::Vec<Client_Mobile_Ios>>,
    dynamic: Client_MobileDynamic,
}

impl Client_Mobile {
    #[doc = "Set the field `android`.\n"]
    pub fn set_android(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_Mobile_Android>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.android = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.android = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `ios`.\n"]
    pub fn set_ios(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_Mobile_Ios>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.ios = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.ios = std::option::Option::Some(d);
            }
        }
        self
    }
}

impl terrars::ToListMappable for Client_Mobile {
    type O = terrars::BlockAssignable<Client_Mobile>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_Mobile {}

impl BuildClient_Mobile {
    pub fn build(self) -> Client_Mobile {
        Client_Mobile {
            android: core::default::Default::default(),
            ios: core::default::Default::default(),
            dynamic: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_MobileRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_MobileRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_MobileRef {
        Client_MobileRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_MobileRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `android` after provisioning.\n"]
    pub fn android(&self) -> terrars::ListRef<Client_Mobile_AndroidRef> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.android", self.base))
    }

    #[doc = "Get a reference to the value of field `ios` after provisioning.\n"]
    pub fn ios(&self) -> terrars::ListRef<Client_Mobile_IosRef> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.ios", self.base))
    }
}

#[derive(Serialize)]
pub struct Client_MyOrganizationConfiguration_ThirdPartyClientAccess {
    allowed_values: terrars::ListField<terrars::PrimField<std::string::String>>,
}

impl Client_MyOrganizationConfiguration_ThirdPartyClientAccess {}

impl terrars::ToListMappable for Client_MyOrganizationConfiguration_ThirdPartyClientAccess {
    type O = terrars::BlockAssignable<Client_MyOrganizationConfiguration_ThirdPartyClientAccess>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_MyOrganizationConfiguration_ThirdPartyClientAccess {
    #[doc = "The third-party client access values that can be set on organizations created for this client through the My Organization API. Required whenever this block is set — the API rejects the block without it. Possible values: `allow`, `block`. Unlike `auth0_connection_profile`'s `cross_app_access_resource_app`, a single value is accepted here. (EA Only)"]
    pub allowed_values: terrars::ListField<terrars::PrimField<std::string::String>>,
}

impl BuildClient_MyOrganizationConfiguration_ThirdPartyClientAccess {
    pub fn build(self) -> Client_MyOrganizationConfiguration_ThirdPartyClientAccess {
        Client_MyOrganizationConfiguration_ThirdPartyClientAccess {
            allowed_values: self.allowed_values,
        }
    }
}

#[allow(dead_code)]
pub struct Client_MyOrganizationConfiguration_ThirdPartyClientAccessRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_MyOrganizationConfiguration_ThirdPartyClientAccessRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> Client_MyOrganizationConfiguration_ThirdPartyClientAccessRef {
        Client_MyOrganizationConfiguration_ThirdPartyClientAccessRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_MyOrganizationConfiguration_ThirdPartyClientAccessRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `allowed_values` after provisioning.\nThe third-party client access values that can be set on organizations created for this client through the My Organization API. Required whenever this block is set — the API rejects the block without it. Possible values: `allow`, `block`. Unlike `auth0_connection_profile`'s `cross_app_access_resource_app`, a single value is accepted here. (EA Only)"]
    pub fn allowed_values(&self) -> terrars::ListRef<terrars::PrimExpr<std::string::String>> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.allowed_values", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `default_value` after provisioning.\nThe default third-party client access value applied to organizations created for this client. The API currently only accepts \"block\"; \"allow\" is rejected with a 400 error, so this is exposed as computed-only rather than user-settable. (EA Only)"]
    pub fn default_value(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.default_value", self.base),
        )
    }
}

#[derive(Serialize, Default)]
struct Client_MyOrganizationConfigurationDynamic {
    third_party_client_access: std::option::Option<
        terrars::DynamicBlock<Client_MyOrganizationConfiguration_ThirdPartyClientAccess>,
    >,
}

#[derive(Serialize)]
pub struct Client_MyOrganizationConfiguration {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    allowed_strategies:
        std::option::Option<terrars::ListField<terrars::PrimField<std::string::String>>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    connection_deletion_behavior: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    connection_profile_id: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    enforce_permission_ceiling: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    enforce_self_assignment_restriction: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    invitation_landing_client_id: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    user_attribute_profile_id: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    third_party_client_access: std::option::Option<
        std::vec::Vec<Client_MyOrganizationConfiguration_ThirdPartyClientAccess>,
    >,
    dynamic: Client_MyOrganizationConfigurationDynamic,
}

impl Client_MyOrganizationConfiguration {
    #[doc = "Set the field `allowed_strategies`.\nThe list of connection strategies that are allowed when creating organizations for this client (e.g. \"okta\", \"samlp\")."]
    pub fn set_allowed_strategies(
        mut self,
        v: impl std::convert::Into<terrars::ListField<terrars::PrimField<std::string::String>>>,
    ) -> Self {
        self.allowed_strategies = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `connection_deletion_behavior`.\nControls the behavior when deleting connections associated with organizations for this client. Possible values: `allow`, `allow_if_empty`."]
    pub fn set_connection_deletion_behavior(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.connection_deletion_behavior = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `connection_profile_id`.\nThe ID of the connection profile to use when creating organizations for this client."]
    pub fn set_connection_profile_id(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.connection_profile_id = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `enforce_permission_ceiling`.\nWhen true, limits the permissions that organization admins can assign to members to only those held by the admin themselves. Requires the `my_org_member_management_ea` feature flag. Available in Early Access (EA)."]
    pub fn set_enforce_permission_ceiling(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.enforce_permission_ceiling = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `enforce_self_assignment_restriction`.\nWhen true, prevents organization admins from assigning permissions to themselves. Requires the `my_org_member_management_ea` feature flag. Available in Early Access (EA)."]
    pub fn set_enforce_self_assignment_restriction(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.enforce_self_assignment_restriction = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `invitation_landing_client_id`.\nThe client ID used as the invitation landing page when creating invitations through the My Organization API. Requires the tenant to have member management enabled, and the referenced client must allow organizations."]
    pub fn set_invitation_landing_client_id(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.invitation_landing_client_id = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `user_attribute_profile_id`.\nThe ID of the user attribute profile to use when creating organizations for this client."]
    pub fn set_user_attribute_profile_id(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.user_attribute_profile_id = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `third_party_client_access`.\n"]
    pub fn set_third_party_client_access(
        mut self,
        v: impl std::convert::Into<
            terrars::BlockAssignable<Client_MyOrganizationConfiguration_ThirdPartyClientAccess>,
        >,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.third_party_client_access = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.third_party_client_access = std::option::Option::Some(d);
            }
        }
        self
    }
}

impl terrars::ToListMappable for Client_MyOrganizationConfiguration {
    type O = terrars::BlockAssignable<Client_MyOrganizationConfiguration>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_MyOrganizationConfiguration {}

impl BuildClient_MyOrganizationConfiguration {
    pub fn build(self) -> Client_MyOrganizationConfiguration {
        Client_MyOrganizationConfiguration {
            allowed_strategies: core::default::Default::default(),
            connection_deletion_behavior: core::default::Default::default(),
            connection_profile_id: core::default::Default::default(),
            enforce_permission_ceiling: core::default::Default::default(),
            enforce_self_assignment_restriction: core::default::Default::default(),
            invitation_landing_client_id: core::default::Default::default(),
            user_attribute_profile_id: core::default::Default::default(),
            third_party_client_access: core::default::Default::default(),
            dynamic: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_MyOrganizationConfigurationRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_MyOrganizationConfigurationRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> Client_MyOrganizationConfigurationRef {
        Client_MyOrganizationConfigurationRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_MyOrganizationConfigurationRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `allowed_strategies` after provisioning.\nThe list of connection strategies that are allowed when creating organizations for this client (e.g. \"okta\", \"samlp\")."]
    pub fn allowed_strategies(&self) -> terrars::ListRef<terrars::PrimExpr<std::string::String>> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.allowed_strategies", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `connection_deletion_behavior` after provisioning.\nControls the behavior when deleting connections associated with organizations for this client. Possible values: `allow`, `allow_if_empty`."]
    pub fn connection_deletion_behavior(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.connection_deletion_behavior", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `connection_profile_id` after provisioning.\nThe ID of the connection profile to use when creating organizations for this client."]
    pub fn connection_profile_id(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.connection_profile_id", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `enforce_permission_ceiling` after provisioning.\nWhen true, limits the permissions that organization admins can assign to members to only those held by the admin themselves. Requires the `my_org_member_management_ea` feature flag. Available in Early Access (EA)."]
    pub fn enforce_permission_ceiling(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.enforce_permission_ceiling", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `enforce_self_assignment_restriction` after provisioning.\nWhen true, prevents organization admins from assigning permissions to themselves. Requires the `my_org_member_management_ea` feature flag. Available in Early Access (EA)."]
    pub fn enforce_self_assignment_restriction(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.enforce_self_assignment_restriction", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `invitation_landing_client_id` after provisioning.\nThe client ID used as the invitation landing page when creating invitations through the My Organization API. Requires the tenant to have member management enabled, and the referenced client must allow organizations."]
    pub fn invitation_landing_client_id(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.invitation_landing_client_id", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `user_attribute_profile_id` after provisioning.\nThe ID of the user attribute profile to use when creating organizations for this client."]
    pub fn user_attribute_profile_id(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.user_attribute_profile_id", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `third_party_client_access` after provisioning.\n"]
    pub fn third_party_client_access(
        &self,
    ) -> terrars::ListRef<Client_MyOrganizationConfiguration_ThirdPartyClientAccessRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.third_party_client_access", self.base),
        )
    }
}

#[derive(Serialize)]
pub struct Client_NativeSocialLogin_Apple {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    enabled: std::option::Option<terrars::PrimField<bool>>,
}

impl Client_NativeSocialLogin_Apple {
    #[doc = "Set the field `enabled`.\n"]
    pub fn set_enabled(mut self, v: impl std::convert::Into<terrars::PrimField<bool>>) -> Self {
        self.enabled = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for Client_NativeSocialLogin_Apple {
    type O = terrars::BlockAssignable<Client_NativeSocialLogin_Apple>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_NativeSocialLogin_Apple {}

impl BuildClient_NativeSocialLogin_Apple {
    pub fn build(self) -> Client_NativeSocialLogin_Apple {
        Client_NativeSocialLogin_Apple {
            enabled: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_NativeSocialLogin_AppleRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_NativeSocialLogin_AppleRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> Client_NativeSocialLogin_AppleRef {
        Client_NativeSocialLogin_AppleRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_NativeSocialLogin_AppleRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `enabled` after provisioning.\n"]
    pub fn enabled(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.enabled", self.base))
    }
}

#[derive(Serialize)]
pub struct Client_NativeSocialLogin_Facebook {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    enabled: std::option::Option<terrars::PrimField<bool>>,
}

impl Client_NativeSocialLogin_Facebook {
    #[doc = "Set the field `enabled`.\n"]
    pub fn set_enabled(mut self, v: impl std::convert::Into<terrars::PrimField<bool>>) -> Self {
        self.enabled = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for Client_NativeSocialLogin_Facebook {
    type O = terrars::BlockAssignable<Client_NativeSocialLogin_Facebook>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_NativeSocialLogin_Facebook {}

impl BuildClient_NativeSocialLogin_Facebook {
    pub fn build(self) -> Client_NativeSocialLogin_Facebook {
        Client_NativeSocialLogin_Facebook {
            enabled: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_NativeSocialLogin_FacebookRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_NativeSocialLogin_FacebookRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> Client_NativeSocialLogin_FacebookRef {
        Client_NativeSocialLogin_FacebookRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_NativeSocialLogin_FacebookRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `enabled` after provisioning.\n"]
    pub fn enabled(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.enabled", self.base))
    }
}

#[derive(Serialize)]
pub struct Client_NativeSocialLogin_Google {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    enabled: std::option::Option<terrars::PrimField<bool>>,
}

impl Client_NativeSocialLogin_Google {
    #[doc = "Set the field `enabled`.\n"]
    pub fn set_enabled(mut self, v: impl std::convert::Into<terrars::PrimField<bool>>) -> Self {
        self.enabled = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for Client_NativeSocialLogin_Google {
    type O = terrars::BlockAssignable<Client_NativeSocialLogin_Google>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_NativeSocialLogin_Google {}

impl BuildClient_NativeSocialLogin_Google {
    pub fn build(self) -> Client_NativeSocialLogin_Google {
        Client_NativeSocialLogin_Google {
            enabled: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_NativeSocialLogin_GoogleRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_NativeSocialLogin_GoogleRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> Client_NativeSocialLogin_GoogleRef {
        Client_NativeSocialLogin_GoogleRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_NativeSocialLogin_GoogleRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `enabled` after provisioning.\n"]
    pub fn enabled(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.enabled", self.base))
    }
}

#[derive(Serialize, Default)]
struct Client_NativeSocialLoginDynamic {
    apple: std::option::Option<terrars::DynamicBlock<Client_NativeSocialLogin_Apple>>,
    facebook: std::option::Option<terrars::DynamicBlock<Client_NativeSocialLogin_Facebook>>,
    google: std::option::Option<terrars::DynamicBlock<Client_NativeSocialLogin_Google>>,
}

#[derive(Serialize)]
pub struct Client_NativeSocialLogin {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    apple: std::option::Option<std::vec::Vec<Client_NativeSocialLogin_Apple>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    facebook: std::option::Option<std::vec::Vec<Client_NativeSocialLogin_Facebook>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    google: std::option::Option<std::vec::Vec<Client_NativeSocialLogin_Google>>,
    dynamic: Client_NativeSocialLoginDynamic,
}

impl Client_NativeSocialLogin {
    #[doc = "Set the field `apple`.\n"]
    pub fn set_apple(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_NativeSocialLogin_Apple>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.apple = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.apple = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `facebook`.\n"]
    pub fn set_facebook(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_NativeSocialLogin_Facebook>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.facebook = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.facebook = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `google`.\n"]
    pub fn set_google(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_NativeSocialLogin_Google>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.google = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.google = std::option::Option::Some(d);
            }
        }
        self
    }
}

impl terrars::ToListMappable for Client_NativeSocialLogin {
    type O = terrars::BlockAssignable<Client_NativeSocialLogin>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_NativeSocialLogin {}

impl BuildClient_NativeSocialLogin {
    pub fn build(self) -> Client_NativeSocialLogin {
        Client_NativeSocialLogin {
            apple: core::default::Default::default(),
            facebook: core::default::Default::default(),
            google: core::default::Default::default(),
            dynamic: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_NativeSocialLoginRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_NativeSocialLoginRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_NativeSocialLoginRef {
        Client_NativeSocialLoginRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_NativeSocialLoginRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `apple` after provisioning.\n"]
    pub fn apple(&self) -> terrars::ListRef<Client_NativeSocialLogin_AppleRef> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.apple", self.base))
    }

    #[doc = "Get a reference to the value of field `facebook` after provisioning.\n"]
    pub fn facebook(&self) -> terrars::ListRef<Client_NativeSocialLogin_FacebookRef> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.facebook", self.base))
    }

    #[doc = "Get a reference to the value of field `google` after provisioning.\n"]
    pub fn google(&self) -> terrars::ListRef<Client_NativeSocialLogin_GoogleRef> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.google", self.base))
    }
}

#[derive(Serialize)]
pub struct Client_OidcLogout_BackchannelLogoutInitiators {
    mode: terrars::PrimField<std::string::String>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    selected_initiators:
        std::option::Option<terrars::SetField<terrars::PrimField<std::string::String>>>,
}

impl Client_OidcLogout_BackchannelLogoutInitiators {
    #[doc = "Set the field `selected_initiators`.\nContains the list of initiators to be enabled for the given client."]
    pub fn set_selected_initiators(
        mut self,
        v: impl std::convert::Into<terrars::SetField<terrars::PrimField<std::string::String>>>,
    ) -> Self {
        self.selected_initiators = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for Client_OidcLogout_BackchannelLogoutInitiators {
    type O = terrars::BlockAssignable<Client_OidcLogout_BackchannelLogoutInitiators>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_OidcLogout_BackchannelLogoutInitiators {
    #[doc = "Determines the configuration method for enabling initiators. `custom` enables only the initiators listed in the backchannel_logout_selected_initiators set, `all` enables all current and future initiators."]
    pub mode: terrars::PrimField<std::string::String>,
}

impl BuildClient_OidcLogout_BackchannelLogoutInitiators {
    pub fn build(self) -> Client_OidcLogout_BackchannelLogoutInitiators {
        Client_OidcLogout_BackchannelLogoutInitiators {
            mode: self.mode,
            selected_initiators: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_OidcLogout_BackchannelLogoutInitiatorsRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_OidcLogout_BackchannelLogoutInitiatorsRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> Client_OidcLogout_BackchannelLogoutInitiatorsRef {
        Client_OidcLogout_BackchannelLogoutInitiatorsRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_OidcLogout_BackchannelLogoutInitiatorsRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `mode` after provisioning.\nDetermines the configuration method for enabling initiators. `custom` enables only the initiators listed in the backchannel_logout_selected_initiators set, `all` enables all current and future initiators."]
    pub fn mode(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.mode", self.base))
    }

    #[doc = "Get a reference to the value of field `selected_initiators` after provisioning.\nContains the list of initiators to be enabled for the given client."]
    pub fn selected_initiators(&self) -> terrars::SetRef<terrars::PrimExpr<std::string::String>> {
        terrars::SetRef::new(
            self.shared().clone(),
            format!("{}.selected_initiators", self.base),
        )
    }
}

#[derive(Serialize)]
pub struct Client_OidcLogout_BackchannelLogoutSessionMetadata {
    include: terrars::PrimField<bool>,
}

impl Client_OidcLogout_BackchannelLogoutSessionMetadata {}

impl terrars::ToListMappable for Client_OidcLogout_BackchannelLogoutSessionMetadata {
    type O = terrars::BlockAssignable<Client_OidcLogout_BackchannelLogoutSessionMetadata>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_OidcLogout_BackchannelLogoutSessionMetadata {
    #[doc = "The `include` property determines whether session metadata is included in the logout token."]
    pub include: terrars::PrimField<bool>,
}

impl BuildClient_OidcLogout_BackchannelLogoutSessionMetadata {
    pub fn build(self) -> Client_OidcLogout_BackchannelLogoutSessionMetadata {
        Client_OidcLogout_BackchannelLogoutSessionMetadata {
            include: self.include,
        }
    }
}

#[allow(dead_code)]
pub struct Client_OidcLogout_BackchannelLogoutSessionMetadataRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_OidcLogout_BackchannelLogoutSessionMetadataRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> Client_OidcLogout_BackchannelLogoutSessionMetadataRef {
        Client_OidcLogout_BackchannelLogoutSessionMetadataRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_OidcLogout_BackchannelLogoutSessionMetadataRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `include` after provisioning.\nThe `include` property determines whether session metadata is included in the logout token."]
    pub fn include(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.include", self.base))
    }
}

#[derive(Serialize, Default)]
struct Client_OidcLogoutDynamic {
    backchannel_logout_initiators:
        std::option::Option<terrars::DynamicBlock<Client_OidcLogout_BackchannelLogoutInitiators>>,
    backchannel_logout_session_metadata: std::option::Option<
        terrars::DynamicBlock<Client_OidcLogout_BackchannelLogoutSessionMetadata>,
    >,
}

#[derive(Serialize)]
pub struct Client_OidcLogout {
    backchannel_logout_urls: terrars::SetField<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    backchannel_logout_initiators:
        std::option::Option<std::vec::Vec<Client_OidcLogout_BackchannelLogoutInitiators>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    backchannel_logout_session_metadata:
        std::option::Option<std::vec::Vec<Client_OidcLogout_BackchannelLogoutSessionMetadata>>,
    dynamic: Client_OidcLogoutDynamic,
}

impl Client_OidcLogout {
    #[doc = "Set the field `backchannel_logout_initiators`.\n"]
    pub fn set_backchannel_logout_initiators(
        mut self,
        v: impl std::convert::Into<
            terrars::BlockAssignable<Client_OidcLogout_BackchannelLogoutInitiators>,
        >,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.backchannel_logout_initiators = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.backchannel_logout_initiators = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `backchannel_logout_session_metadata`.\n"]
    pub fn set_backchannel_logout_session_metadata(
        mut self,
        v: impl std::convert::Into<
            terrars::BlockAssignable<Client_OidcLogout_BackchannelLogoutSessionMetadata>,
        >,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.backchannel_logout_session_metadata = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.backchannel_logout_session_metadata = std::option::Option::Some(d);
            }
        }
        self
    }
}

impl terrars::ToListMappable for Client_OidcLogout {
    type O = terrars::BlockAssignable<Client_OidcLogout>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_OidcLogout {
    #[doc = "Set of URLs that are valid to call back from Auth0 for OIDC backchannel logout. Currently only one URL is allowed."]
    pub backchannel_logout_urls: terrars::SetField<terrars::PrimField<std::string::String>>,
}

impl BuildClient_OidcLogout {
    pub fn build(self) -> Client_OidcLogout {
        Client_OidcLogout {
            backchannel_logout_urls: self.backchannel_logout_urls,
            backchannel_logout_initiators: core::default::Default::default(),
            backchannel_logout_session_metadata: core::default::Default::default(),
            dynamic: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_OidcLogoutRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_OidcLogoutRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_OidcLogoutRef {
        Client_OidcLogoutRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_OidcLogoutRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `backchannel_logout_urls` after provisioning.\nSet of URLs that are valid to call back from Auth0 for OIDC backchannel logout. Currently only one URL is allowed."]
    pub fn backchannel_logout_urls(
        &self,
    ) -> terrars::SetRef<terrars::PrimExpr<std::string::String>> {
        terrars::SetRef::new(
            self.shared().clone(),
            format!("{}.backchannel_logout_urls", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `backchannel_logout_initiators` after provisioning.\n"]
    pub fn backchannel_logout_initiators(
        &self,
    ) -> terrars::ListRef<Client_OidcLogout_BackchannelLogoutInitiatorsRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.backchannel_logout_initiators", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `backchannel_logout_session_metadata` after provisioning.\n"]
    pub fn backchannel_logout_session_metadata(
        &self,
    ) -> terrars::ListRef<Client_OidcLogout_BackchannelLogoutSessionMetadataRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.backchannel_logout_session_metadata", self.base),
        )
    }
}

#[derive(Serialize)]
pub struct Client_OidcSupport {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    allow_all_scopes: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    allowed_scopes: std::option::Option<terrars::SetField<terrars::PrimField<std::string::String>>>,
    is_allowed: terrars::PrimField<bool>,
}

impl Client_OidcSupport {
    #[doc = "Set the field `allow_all_scopes`.\nIndicates whether the client may request all optional OIDC scopes. Conflicts with `allowed_scopes`."]
    pub fn set_allow_all_scopes(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.allow_all_scopes = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `allowed_scopes`.\nOptional OIDC scopes the client may request. Possible values: [profile email address phone]. The `openid` scope is implied and must not be listed. Conflicts with `allow_all_scopes`."]
    pub fn set_allowed_scopes(
        mut self,
        v: impl std::convert::Into<terrars::SetField<terrars::PrimField<std::string::String>>>,
    ) -> Self {
        self.allowed_scopes = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for Client_OidcSupport {
    type O = terrars::BlockAssignable<Client_OidcSupport>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_OidcSupport {
    #[doc = "Indicates whether the client may request optional OIDC scopes. When `false`, `allowed_scopes` and `allow_all_scopes` cannot be set."]
    pub is_allowed: terrars::PrimField<bool>,
}

impl BuildClient_OidcSupport {
    pub fn build(self) -> Client_OidcSupport {
        Client_OidcSupport {
            allow_all_scopes: core::default::Default::default(),
            allowed_scopes: core::default::Default::default(),
            is_allowed: self.is_allowed,
        }
    }
}

#[allow(dead_code)]
pub struct Client_OidcSupportRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_OidcSupportRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_OidcSupportRef {
        Client_OidcSupportRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_OidcSupportRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `allow_all_scopes` after provisioning.\nIndicates whether the client may request all optional OIDC scopes. Conflicts with `allowed_scopes`."]
    pub fn allow_all_scopes(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.allow_all_scopes", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `allowed_scopes` after provisioning.\nOptional OIDC scopes the client may request. Possible values: [profile email address phone]. The `openid` scope is implied and must not be listed. Conflicts with `allow_all_scopes`."]
    pub fn allowed_scopes(&self) -> terrars::SetRef<terrars::PrimExpr<std::string::String>> {
        terrars::SetRef::new(
            self.shared().clone(),
            format!("{}.allowed_scopes", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `is_allowed` after provisioning.\nIndicates whether the client may request optional OIDC scopes. When `false`, `allowed_scopes` and `allow_all_scopes` cannot be set."]
    pub fn is_allowed(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.is_allowed", self.base))
    }
}

#[derive(Serialize)]
pub struct Client_RefreshToken_Policies {
    audience: terrars::PrimField<std::string::String>,
    scope: terrars::ListField<terrars::PrimField<std::string::String>>,
}

impl Client_RefreshToken_Policies {}

impl terrars::ToListMappable for Client_RefreshToken_Policies {
    type O = terrars::BlockAssignable<Client_RefreshToken_Policies>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_RefreshToken_Policies {
    #[doc = "The identifier of the resource server to which the Multi Resource Refresh Token Policy applies"]
    pub audience: terrars::PrimField<std::string::String>,
    #[doc = "The resource server permissions granted under the Multi Resource Refresh Token Policy, defining the context in which an access token can be used"]
    pub scope: terrars::ListField<terrars::PrimField<std::string::String>>,
}

impl BuildClient_RefreshToken_Policies {
    pub fn build(self) -> Client_RefreshToken_Policies {
        Client_RefreshToken_Policies {
            audience: self.audience,
            scope: self.scope,
        }
    }
}

#[allow(dead_code)]
pub struct Client_RefreshToken_PoliciesRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_RefreshToken_PoliciesRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> Client_RefreshToken_PoliciesRef {
        Client_RefreshToken_PoliciesRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_RefreshToken_PoliciesRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `audience` after provisioning.\nThe identifier of the resource server to which the Multi Resource Refresh Token Policy applies"]
    pub fn audience(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.audience", self.base))
    }

    #[doc = "Get a reference to the value of field `scope` after provisioning.\nThe resource server permissions granted under the Multi Resource Refresh Token Policy, defining the context in which an access token can be used"]
    pub fn scope(&self) -> terrars::ListRef<terrars::PrimExpr<std::string::String>> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.scope", self.base))
    }
}

#[derive(Serialize, Default)]
struct Client_RefreshTokenDynamic {
    policies: std::option::Option<terrars::DynamicBlock<Client_RefreshToken_Policies>>,
}

#[derive(Serialize)]
pub struct Client_RefreshToken {
    expiration_type: terrars::PrimField<std::string::String>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    idle_token_lifetime: std::option::Option<terrars::PrimField<f64>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    infinite_idle_token_lifetime: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    infinite_token_lifetime: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    leeway: std::option::Option<terrars::PrimField<f64>>,
    rotation_type: terrars::PrimField<std::string::String>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    token_lifetime: std::option::Option<terrars::PrimField<f64>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    policies: std::option::Option<std::vec::Vec<Client_RefreshToken_Policies>>,
    dynamic: Client_RefreshTokenDynamic,
}

impl Client_RefreshToken {
    #[doc = "Set the field `idle_token_lifetime`.\nThe time in seconds after which inactive refresh tokens will expire."]
    pub fn set_idle_token_lifetime(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<f64>>,
    ) -> Self {
        self.idle_token_lifetime = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `infinite_idle_token_lifetime`.\nWhether inactive refresh tokens should remain valid indefinitely."]
    pub fn set_infinite_idle_token_lifetime(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.infinite_idle_token_lifetime = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `infinite_token_lifetime`.\nWhether refresh tokens should remain valid indefinitely. If false, `token_lifetime` should also be set."]
    pub fn set_infinite_token_lifetime(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.infinite_token_lifetime = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `leeway`.\nThe amount of time in seconds in which a refresh token may be reused without triggering reuse detection."]
    pub fn set_leeway(mut self, v: impl std::convert::Into<terrars::PrimField<f64>>) -> Self {
        self.leeway = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `token_lifetime`.\nThe absolute lifetime of a refresh token in seconds."]
    pub fn set_token_lifetime(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<f64>>,
    ) -> Self {
        self.token_lifetime = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `policies`.\n"]
    pub fn set_policies(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_RefreshToken_Policies>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.policies = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.policies = std::option::Option::Some(d);
            }
        }
        self
    }
}

impl terrars::ToListMappable for Client_RefreshToken {
    type O = terrars::BlockAssignable<Client_RefreshToken>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_RefreshToken {
    #[doc = "Options include `expiring`, `non-expiring`. Whether a refresh token will expire based on an absolute lifetime, after which the token can no longer be used. If rotation is `rotating`, this must be set to `expiring`."]
    pub expiration_type: terrars::PrimField<std::string::String>,
    #[doc = "Options include `rotating`, `non-rotating`. When `rotating`, exchanging a refresh token will cause a new refresh token to be issued and the existing token will be invalidated. This allows for automatic detection of token reuse if the token is leaked."]
    pub rotation_type: terrars::PrimField<std::string::String>,
}

impl BuildClient_RefreshToken {
    pub fn build(self) -> Client_RefreshToken {
        Client_RefreshToken {
            expiration_type: self.expiration_type,
            idle_token_lifetime: core::default::Default::default(),
            infinite_idle_token_lifetime: core::default::Default::default(),
            infinite_token_lifetime: core::default::Default::default(),
            leeway: core::default::Default::default(),
            rotation_type: self.rotation_type,
            token_lifetime: core::default::Default::default(),
            policies: core::default::Default::default(),
            dynamic: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_RefreshTokenRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_RefreshTokenRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_RefreshTokenRef {
        Client_RefreshTokenRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_RefreshTokenRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `expiration_type` after provisioning.\nOptions include `expiring`, `non-expiring`. Whether a refresh token will expire based on an absolute lifetime, after which the token can no longer be used. If rotation is `rotating`, this must be set to `expiring`."]
    pub fn expiration_type(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.expiration_type", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `idle_token_lifetime` after provisioning.\nThe time in seconds after which inactive refresh tokens will expire."]
    pub fn idle_token_lifetime(&self) -> terrars::PrimExpr<f64> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.idle_token_lifetime", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `infinite_idle_token_lifetime` after provisioning.\nWhether inactive refresh tokens should remain valid indefinitely."]
    pub fn infinite_idle_token_lifetime(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.infinite_idle_token_lifetime", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `infinite_token_lifetime` after provisioning.\nWhether refresh tokens should remain valid indefinitely. If false, `token_lifetime` should also be set."]
    pub fn infinite_token_lifetime(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.infinite_token_lifetime", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `leeway` after provisioning.\nThe amount of time in seconds in which a refresh token may be reused without triggering reuse detection."]
    pub fn leeway(&self) -> terrars::PrimExpr<f64> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.leeway", self.base))
    }

    #[doc = "Get a reference to the value of field `rotation_type` after provisioning.\nOptions include `rotating`, `non-rotating`. When `rotating`, exchanging a refresh token will cause a new refresh token to be issued and the existing token will be invalidated. This allows for automatic detection of token reuse if the token is leaked."]
    pub fn rotation_type(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.rotation_type", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `token_lifetime` after provisioning.\nThe absolute lifetime of a refresh token in seconds."]
    pub fn token_lifetime(&self) -> terrars::PrimExpr<f64> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.token_lifetime", self.base),
        )
    }
}

#[derive(Serialize)]
pub struct Client_SessionTransfer_Delegation {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    allow_delegated_access: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    enforce_device_binding: std::option::Option<terrars::PrimField<std::string::String>>,
}

impl Client_SessionTransfer_Delegation {
    #[doc = "Set the field `allow_delegated_access`.\nIndicates whether delegation (impersonation) access is allowed using Session Transfer Tokens. Defaults to `false`."]
    pub fn set_allow_delegated_access(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.allow_delegated_access = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `enforce_device_binding`.\nIndicates the device binding enforcement for delegation (impersonation) access. If set to 'ip', device binding is enforced by IP. If set to 'asn', device binding is enforced by ASN. Defaults to `ip`."]
    pub fn set_enforce_device_binding(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.enforce_device_binding = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for Client_SessionTransfer_Delegation {
    type O = terrars::BlockAssignable<Client_SessionTransfer_Delegation>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_SessionTransfer_Delegation {}

impl BuildClient_SessionTransfer_Delegation {
    pub fn build(self) -> Client_SessionTransfer_Delegation {
        Client_SessionTransfer_Delegation {
            allow_delegated_access: core::default::Default::default(),
            enforce_device_binding: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_SessionTransfer_DelegationRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_SessionTransfer_DelegationRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> Client_SessionTransfer_DelegationRef {
        Client_SessionTransfer_DelegationRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_SessionTransfer_DelegationRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `allow_delegated_access` after provisioning.\nIndicates whether delegation (impersonation) access is allowed using Session Transfer Tokens. Defaults to `false`."]
    pub fn allow_delegated_access(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.allow_delegated_access", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `enforce_device_binding` after provisioning.\nIndicates the device binding enforcement for delegation (impersonation) access. If set to 'ip', device binding is enforced by IP. If set to 'asn', device binding is enforced by ASN. Defaults to `ip`."]
    pub fn enforce_device_binding(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.enforce_device_binding", self.base),
        )
    }
}

#[derive(Serialize, Default)]
struct Client_SessionTransferDynamic {
    delegation: std::option::Option<terrars::DynamicBlock<Client_SessionTransfer_Delegation>>,
}

#[derive(Serialize)]
pub struct Client_SessionTransfer {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    allow_refresh_token: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    allowed_authentication_methods:
        std::option::Option<terrars::SetField<terrars::PrimField<std::string::String>>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    can_create_session_transfer_token: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    enforce_cascade_revocation: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    enforce_device_binding: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    enforce_online_refresh_tokens: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    delegation: std::option::Option<std::vec::Vec<Client_SessionTransfer_Delegation>>,
    dynamic: Client_SessionTransferDynamic,
}

impl Client_SessionTransfer {
    #[doc = "Set the field `allow_refresh_token`.\nIndicates whether the application is allowed to use a refresh token when using a session_transfer_token session."]
    pub fn set_allow_refresh_token(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.allow_refresh_token = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `allowed_authentication_methods`.\n"]
    pub fn set_allowed_authentication_methods(
        mut self,
        v: impl std::convert::Into<terrars::SetField<terrars::PrimField<std::string::String>>>,
    ) -> Self {
        self.allowed_authentication_methods = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `can_create_session_transfer_token`.\nIndicates whether the application(Native app) can use the Token Exchange endpoint to create a session_transfer_token"]
    pub fn set_can_create_session_transfer_token(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.can_create_session_transfer_token = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `enforce_cascade_revocation`.\nIndicates whether revoking the parent Refresh Token that initiated a Native to Web flow and was used to issue a Session Transfer Token should trigger a cascade revocation affecting its dependent child entities. Usually configured in the native application."]
    pub fn set_enforce_cascade_revocation(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.enforce_cascade_revocation = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `enforce_device_binding`.\nConfigures the level of device binding enforced when a session_transfer_token is consumed. Can be one of `ip`, `asn` or `none`."]
    pub fn set_enforce_device_binding(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.enforce_device_binding = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `enforce_online_refresh_tokens`.\nIndicates whether Refresh Tokens created during a native-to-web session are tied to that session's lifetime. This determines if such refresh tokens should be automatically revoked when their corresponding sessions are. Usually configured in the web application."]
    pub fn set_enforce_online_refresh_tokens(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.enforce_online_refresh_tokens = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `delegation`.\n"]
    pub fn set_delegation(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_SessionTransfer_Delegation>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.delegation = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.delegation = std::option::Option::Some(d);
            }
        }
        self
    }
}

impl terrars::ToListMappable for Client_SessionTransfer {
    type O = terrars::BlockAssignable<Client_SessionTransfer>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_SessionTransfer {}

impl BuildClient_SessionTransfer {
    pub fn build(self) -> Client_SessionTransfer {
        Client_SessionTransfer {
            allow_refresh_token: core::default::Default::default(),
            allowed_authentication_methods: core::default::Default::default(),
            can_create_session_transfer_token: core::default::Default::default(),
            enforce_cascade_revocation: core::default::Default::default(),
            enforce_device_binding: core::default::Default::default(),
            enforce_online_refresh_tokens: core::default::Default::default(),
            delegation: core::default::Default::default(),
            dynamic: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_SessionTransferRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_SessionTransferRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_SessionTransferRef {
        Client_SessionTransferRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_SessionTransferRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `allow_refresh_token` after provisioning.\nIndicates whether the application is allowed to use a refresh token when using a session_transfer_token session."]
    pub fn allow_refresh_token(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.allow_refresh_token", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `allowed_authentication_methods` after provisioning.\n"]
    pub fn allowed_authentication_methods(
        &self,
    ) -> terrars::SetRef<terrars::PrimExpr<std::string::String>> {
        terrars::SetRef::new(
            self.shared().clone(),
            format!("{}.allowed_authentication_methods", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `can_create_session_transfer_token` after provisioning.\nIndicates whether the application(Native app) can use the Token Exchange endpoint to create a session_transfer_token"]
    pub fn can_create_session_transfer_token(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.can_create_session_transfer_token", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `enforce_cascade_revocation` after provisioning.\nIndicates whether revoking the parent Refresh Token that initiated a Native to Web flow and was used to issue a Session Transfer Token should trigger a cascade revocation affecting its dependent child entities. Usually configured in the native application."]
    pub fn enforce_cascade_revocation(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.enforce_cascade_revocation", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `enforce_device_binding` after provisioning.\nConfigures the level of device binding enforced when a session_transfer_token is consumed. Can be one of `ip`, `asn` or `none`."]
    pub fn enforce_device_binding(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.enforce_device_binding", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `enforce_online_refresh_tokens` after provisioning.\nIndicates whether Refresh Tokens created during a native-to-web session are tied to that session's lifetime. This determines if such refresh tokens should be automatically revoked when their corresponding sessions are. Usually configured in the web application."]
    pub fn enforce_online_refresh_tokens(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.enforce_online_refresh_tokens", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `delegation` after provisioning.\n"]
    pub fn delegation(&self) -> terrars::ListRef<Client_SessionTransfer_DelegationRef> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.delegation", self.base))
    }
}

#[derive(Serialize)]
pub struct Client_TokenExchange {
    allow_any_profile_of_type: terrars::ListField<terrars::PrimField<std::string::String>>,
}

impl Client_TokenExchange {}

impl terrars::ToListMappable for Client_TokenExchange {
    type O = terrars::BlockAssignable<Client_TokenExchange>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_TokenExchange {
    #[doc = "List of allowed profile types for token exchange. Supported values include: custom_authentication, on_behalf_of_token_exchange."]
    pub allow_any_profile_of_type: terrars::ListField<terrars::PrimField<std::string::String>>,
}

impl BuildClient_TokenExchange {
    pub fn build(self) -> Client_TokenExchange {
        Client_TokenExchange {
            allow_any_profile_of_type: self.allow_any_profile_of_type,
        }
    }
}

#[allow(dead_code)]
pub struct Client_TokenExchangeRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_TokenExchangeRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_TokenExchangeRef {
        Client_TokenExchangeRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_TokenExchangeRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `allow_any_profile_of_type` after provisioning.\nList of allowed profile types for token exchange. Supported values include: custom_authentication, on_behalf_of_token_exchange."]
    pub fn allow_any_profile_of_type(
        &self,
    ) -> terrars::ListRef<terrars::PrimExpr<std::string::String>> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.allow_any_profile_of_type", self.base),
        )
    }
}

#[derive(Serialize)]
pub struct Client_TokenQuota_ClientCredentials {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    enforce: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    per_day: std::option::Option<terrars::PrimField<f64>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    per_hour: std::option::Option<terrars::PrimField<f64>>,
}

impl Client_TokenQuota_ClientCredentials {
    #[doc = "Set the field `enforce`.\nIf enabled, the quota will be enforced and requests in excess of the quota will fail. If disabled, the quota will not be enforced, but notifications for requests exceeding the quota will be available in logs."]
    pub fn set_enforce(mut self, v: impl std::convert::Into<terrars::PrimField<bool>>) -> Self {
        self.enforce = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `per_day`.\nMaximum number of issued tokens per day"]
    pub fn set_per_day(mut self, v: impl std::convert::Into<terrars::PrimField<f64>>) -> Self {
        self.per_day = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `per_hour`.\nMaximum number of issued tokens per hour"]
    pub fn set_per_hour(mut self, v: impl std::convert::Into<terrars::PrimField<f64>>) -> Self {
        self.per_hour = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for Client_TokenQuota_ClientCredentials {
    type O = terrars::BlockAssignable<Client_TokenQuota_ClientCredentials>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_TokenQuota_ClientCredentials {}

impl BuildClient_TokenQuota_ClientCredentials {
    pub fn build(self) -> Client_TokenQuota_ClientCredentials {
        Client_TokenQuota_ClientCredentials {
            enforce: core::default::Default::default(),
            per_day: core::default::Default::default(),
            per_hour: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_TokenQuota_ClientCredentialsRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_TokenQuota_ClientCredentialsRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> Client_TokenQuota_ClientCredentialsRef {
        Client_TokenQuota_ClientCredentialsRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_TokenQuota_ClientCredentialsRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `enforce` after provisioning.\nIf enabled, the quota will be enforced and requests in excess of the quota will fail. If disabled, the quota will not be enforced, but notifications for requests exceeding the quota will be available in logs."]
    pub fn enforce(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.enforce", self.base))
    }

    #[doc = "Get a reference to the value of field `per_day` after provisioning.\nMaximum number of issued tokens per day"]
    pub fn per_day(&self) -> terrars::PrimExpr<f64> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.per_day", self.base))
    }

    #[doc = "Get a reference to the value of field `per_hour` after provisioning.\nMaximum number of issued tokens per hour"]
    pub fn per_hour(&self) -> terrars::PrimExpr<f64> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.per_hour", self.base))
    }
}

#[derive(Serialize, Default)]
struct Client_TokenQuotaDynamic {
    client_credentials:
        std::option::Option<terrars::DynamicBlock<Client_TokenQuota_ClientCredentials>>,
}

#[derive(Serialize)]
pub struct Client_TokenQuota {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    client_credentials: std::option::Option<std::vec::Vec<Client_TokenQuota_ClientCredentials>>,
    dynamic: Client_TokenQuotaDynamic,
}

impl Client_TokenQuota {
    #[doc = "Set the field `client_credentials`.\n"]
    pub fn set_client_credentials(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Client_TokenQuota_ClientCredentials>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.client_credentials = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.client_credentials = std::option::Option::Some(d);
            }
        }
        self
    }
}

impl terrars::ToListMappable for Client_TokenQuota {
    type O = terrars::BlockAssignable<Client_TokenQuota>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildClient_TokenQuota {}

impl BuildClient_TokenQuota {
    pub fn build(self) -> Client_TokenQuota {
        Client_TokenQuota {
            client_credentials: core::default::Default::default(),
            dynamic: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Client_TokenQuotaRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Client_TokenQuotaRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Client_TokenQuotaRef {
        Client_TokenQuotaRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Client_TokenQuotaRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `client_credentials` after provisioning.\n"]
    pub fn client_credentials(&self) -> terrars::ListRef<Client_TokenQuota_ClientCredentialsRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.client_credentials", self.base),
        )
    }
}
