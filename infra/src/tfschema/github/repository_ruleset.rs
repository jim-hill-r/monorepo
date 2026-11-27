#![allow(non_camel_case_types)]

use super::provider::ProviderGithub;
use serde::Serialize;
use terrars::{Ref as _, Referable as _, Resource_ as _};

#[derive(Serialize)]
struct RepositoryRulesetData_ {
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
    enforcement: terrars::PrimField<std::string::String>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    id: std::option::Option<terrars::PrimField<std::string::String>>,
    name: terrars::PrimField<std::string::String>,
    repository: terrars::PrimField<std::string::String>,
    target: terrars::PrimField<std::string::String>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    bypass_actors: std::option::Option<std::vec::Vec<RepositoryRuleset_BypassActors>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    conditions: std::option::Option<std::vec::Vec<RepositoryRuleset_Conditions>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    rules: std::option::Option<std::vec::Vec<RepositoryRuleset_Rules>>,
    dynamic: RepositoryRulesetDynamic,
}

struct RepositoryRuleset_ {
    shared: terrars::StackShared,
    tf_id: std::string::String,
    data: std::cell::RefCell<RepositoryRulesetData_>,
}

#[derive(Clone)]
pub struct RepositoryRuleset(std::rc::Rc<RepositoryRuleset_>);

impl RepositoryRuleset {
    fn shared(&self) -> &terrars::StackShared {
        &self.0.shared
    }

    pub fn depends_on(self, dep: &impl terrars::Referable) -> Self {
        self.0.data.borrow_mut().depends_on.push(dep.extract_ref());
        self
    }

    pub fn set_provider(self, provider: &ProviderGithub) -> Self {
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

    #[doc = "Set the field `id`.\n"]
    pub fn set_id(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().id = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `bypass_actors`.\n"]
    pub fn set_bypass_actors(
        self,
        v: impl std::convert::Into<terrars::BlockAssignable<RepositoryRuleset_BypassActors>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.0.data.borrow_mut().bypass_actors = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.0.data.borrow_mut().dynamic.bypass_actors = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `conditions`.\n"]
    pub fn set_conditions(
        self,
        v: impl std::convert::Into<terrars::BlockAssignable<RepositoryRuleset_Conditions>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.0.data.borrow_mut().conditions = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.0.data.borrow_mut().dynamic.conditions = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `rules`.\n"]
    pub fn set_rules(
        self,
        v: impl std::convert::Into<terrars::BlockAssignable<RepositoryRuleset_Rules>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.0.data.borrow_mut().rules = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.0.data.borrow_mut().dynamic.rules = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Get a reference to the value of field `enforcement` after provisioning.\nPossible values for Enforcement are `disabled`, `active`, `evaluate`. Note: `evaluate` is currently only supported for owners of type `organization`."]
    pub fn enforcement(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.enforcement", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `etag` after provisioning.\n"]
    pub fn etag(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.etag", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `id` after provisioning.\n"]
    pub fn id(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.id", self.extract_ref()))
    }

    #[doc = "Get a reference to the value of field `name` after provisioning.\nThe name of the ruleset."]
    pub fn name(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.name", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `node_id` after provisioning.\nGraphQL global node id for use with v4 API."]
    pub fn node_id(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.node_id", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `repository` after provisioning.\nName of the repository to apply ruleset to."]
    pub fn repository(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.repository", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `ruleset_id` after provisioning.\nGitHub ID for the ruleset."]
    pub fn ruleset_id(&self) -> terrars::PrimExpr<f64> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.ruleset_id", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `target` after provisioning.\nPossible values are branch, push and tag"]
    pub fn target(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.target", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `bypass_actors` after provisioning.\n"]
    pub fn bypass_actors(&self) -> terrars::ListRef<RepositoryRuleset_BypassActorsRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.bypass_actors", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `conditions` after provisioning.\n"]
    pub fn conditions(&self) -> terrars::ListRef<RepositoryRuleset_ConditionsRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.conditions", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `rules` after provisioning.\n"]
    pub fn rules(&self) -> terrars::ListRef<RepositoryRuleset_RulesRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.rules", self.extract_ref()),
        )
    }
}

impl terrars::Referable for RepositoryRuleset {
    fn extract_ref(&self) -> std::string::String {
        format!(
            "{}.{}",
            self.0.extract_resource_type(),
            self.0.extract_tf_id()
        )
    }
}

impl terrars::Resource for RepositoryRuleset {}

impl terrars::ToListMappable for RepositoryRuleset {
    type O = terrars::ListRef<RepositoryRulesetRef>;

    fn do_map(self, base: std::string::String) -> Self::O {
        self.0.data.borrow_mut().for_each = std::option::Option::Some(format!("${{{}}}", base));
        terrars::ListRef::new(self.0.shared.clone(), self.extract_ref())
    }
}

impl terrars::Resource_ for RepositoryRuleset_ {
    fn extract_resource_type(&self) -> std::string::String {
        "github_repository_ruleset".into()
    }

    fn extract_tf_id(&self) -> std::string::String {
        self.tf_id.clone()
    }

    fn extract_value(&self) -> serde_json::Value {
        serde_json::to_value(&self.data).unwrap()
    }
}

pub struct BuildRepositoryRuleset {
    pub tf_id: std::string::String,
    #[doc = "Possible values for Enforcement are `disabled`, `active`, `evaluate`. Note: `evaluate` is currently only supported for owners of type `organization`."]
    pub enforcement: terrars::PrimField<std::string::String>,
    #[doc = "The name of the ruleset."]
    pub name: terrars::PrimField<std::string::String>,
    #[doc = "Name of the repository to apply ruleset to."]
    pub repository: terrars::PrimField<std::string::String>,
    #[doc = "Possible values are branch, push and tag"]
    pub target: terrars::PrimField<std::string::String>,
}

impl BuildRepositoryRuleset {
    pub fn build(self, stack: &mut terrars::Stack) -> RepositoryRuleset {
        let out = RepositoryRuleset(std::rc::Rc::new(RepositoryRuleset_ {
            shared: stack.shared.clone(),
            tf_id: self.tf_id,
            data: std::cell::RefCell::new(RepositoryRulesetData_ {
                depends_on: core::default::Default::default(),
                provider: std::option::Option::None,
                lifecycle: core::default::Default::default(),
                for_each: std::option::Option::None,
                provisioner: core::default::Default::default(),
                enforcement: self.enforcement,
                id: core::default::Default::default(),
                name: self.name,
                repository: self.repository,
                target: self.target,
                bypass_actors: core::default::Default::default(),
                conditions: core::default::Default::default(),
                rules: core::default::Default::default(),
                dynamic: core::default::Default::default(),
            }),
        }));
        stack.add_resource(out.0.clone());
        out
    }
}

pub struct RepositoryRulesetRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for RepositoryRulesetRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Self {
        Self {
            shared: shared,
            base: base,
        }
    }
}

impl RepositoryRulesetRef {
    fn extract_ref(&self) -> std::string::String {
        self.base.clone()
    }

    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `enforcement` after provisioning.\nPossible values for Enforcement are `disabled`, `active`, `evaluate`. Note: `evaluate` is currently only supported for owners of type `organization`."]
    pub fn enforcement(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.enforcement", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `etag` after provisioning.\n"]
    pub fn etag(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.etag", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `id` after provisioning.\n"]
    pub fn id(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.id", self.extract_ref()))
    }

    #[doc = "Get a reference to the value of field `name` after provisioning.\nThe name of the ruleset."]
    pub fn name(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.name", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `node_id` after provisioning.\nGraphQL global node id for use with v4 API."]
    pub fn node_id(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.node_id", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `repository` after provisioning.\nName of the repository to apply ruleset to."]
    pub fn repository(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.repository", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `ruleset_id` after provisioning.\nGitHub ID for the ruleset."]
    pub fn ruleset_id(&self) -> terrars::PrimExpr<f64> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.ruleset_id", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `target` after provisioning.\nPossible values are branch, push and tag"]
    pub fn target(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.target", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `bypass_actors` after provisioning.\n"]
    pub fn bypass_actors(&self) -> terrars::ListRef<RepositoryRuleset_BypassActorsRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.bypass_actors", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `conditions` after provisioning.\n"]
    pub fn conditions(&self) -> terrars::ListRef<RepositoryRuleset_ConditionsRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.conditions", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `rules` after provisioning.\n"]
    pub fn rules(&self) -> terrars::ListRef<RepositoryRuleset_RulesRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.rules", self.extract_ref()),
        )
    }
}

#[derive(Serialize, Default)]
struct RepositoryRulesetDynamic {
    bypass_actors: std::option::Option<terrars::DynamicBlock<RepositoryRuleset_BypassActors>>,
    conditions: std::option::Option<terrars::DynamicBlock<RepositoryRuleset_Conditions>>,
    rules: std::option::Option<terrars::DynamicBlock<RepositoryRuleset_Rules>>,
}

#[derive(Serialize)]
pub struct RepositoryRuleset_BypassActors {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    actor_id: std::option::Option<terrars::PrimField<f64>>,
    actor_type: terrars::PrimField<std::string::String>,
    bypass_mode: terrars::PrimField<std::string::String>,
}

impl RepositoryRuleset_BypassActors {
    #[doc = "Set the field `actor_id`.\nThe ID of the actor that can bypass a ruleset. If `actor_type` is `Integration`, `actor_id` is a GitHub App ID. When `actor_type` is `User`, this should be set to the numeric GitHub user ID. Some actor types such as `OrganizationAdmin`, `EnterpriseOwner`, and `DeployKey` do not have an ID — this argument should not be set in those cases as the GitHub API will ignore it."]
    pub fn set_actor_id(mut self, v: impl std::convert::Into<terrars::PrimField<f64>>) -> Self {
        self.actor_id = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for RepositoryRuleset_BypassActors {
    type O = terrars::BlockAssignable<RepositoryRuleset_BypassActors>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildRepositoryRuleset_BypassActors {
    #[doc = "The type of actor that can bypass a ruleset. Can be one of: `RepositoryRole`, `Team`, `Integration`, `OrganizationAdmin`, `DeployKey`, `EnterpriseOwner`, or `User`. See https://docs.github.com/en/rest/repos/rules for more information."]
    pub actor_type: terrars::PrimField<std::string::String>,
    #[doc = "When the specified actor can bypass the ruleset. pull_request means that an actor can only bypass rules on pull requests. Can be one of: `always`, `pull_request`, `exempt`."]
    pub bypass_mode: terrars::PrimField<std::string::String>,
}

impl BuildRepositoryRuleset_BypassActors {
    pub fn build(self) -> RepositoryRuleset_BypassActors {
        RepositoryRuleset_BypassActors {
            actor_id: core::default::Default::default(),
            actor_type: self.actor_type,
            bypass_mode: self.bypass_mode,
        }
    }
}

#[allow(dead_code)]
pub struct RepositoryRuleset_BypassActorsRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for RepositoryRuleset_BypassActorsRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> RepositoryRuleset_BypassActorsRef {
        RepositoryRuleset_BypassActorsRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl RepositoryRuleset_BypassActorsRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `actor_id` after provisioning.\nThe ID of the actor that can bypass a ruleset. If `actor_type` is `Integration`, `actor_id` is a GitHub App ID. When `actor_type` is `User`, this should be set to the numeric GitHub user ID. Some actor types such as `OrganizationAdmin`, `EnterpriseOwner`, and `DeployKey` do not have an ID — this argument should not be set in those cases as the GitHub API will ignore it."]
    pub fn actor_id(&self) -> terrars::PrimExpr<f64> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.actor_id", self.base))
    }

    #[doc = "Get a reference to the value of field `actor_type` after provisioning.\nThe type of actor that can bypass a ruleset. Can be one of: `RepositoryRole`, `Team`, `Integration`, `OrganizationAdmin`, `DeployKey`, `EnterpriseOwner`, or `User`. See https://docs.github.com/en/rest/repos/rules for more information."]
    pub fn actor_type(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.actor_type", self.base))
    }

    #[doc = "Get a reference to the value of field `bypass_mode` after provisioning.\nWhen the specified actor can bypass the ruleset. pull_request means that an actor can only bypass rules on pull requests. Can be one of: `always`, `pull_request`, `exempt`."]
    pub fn bypass_mode(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.bypass_mode", self.base))
    }
}

#[derive(Serialize)]
pub struct RepositoryRuleset_Conditions_RefName {
    exclude: terrars::ListField<terrars::PrimField<std::string::String>>,
    include: terrars::ListField<terrars::PrimField<std::string::String>>,
}

impl RepositoryRuleset_Conditions_RefName {}

impl terrars::ToListMappable for RepositoryRuleset_Conditions_RefName {
    type O = terrars::BlockAssignable<RepositoryRuleset_Conditions_RefName>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildRepositoryRuleset_Conditions_RefName {
    #[doc = "Array of ref names or patterns to exclude. The condition will not pass if any of these patterns match."]
    pub exclude: terrars::ListField<terrars::PrimField<std::string::String>>,
    #[doc = "Array of ref names or patterns to include. One of these patterns must match for the condition to pass. Also accepts `~DEFAULT_BRANCH` to include the default branch or `~ALL` to include all branches."]
    pub include: terrars::ListField<terrars::PrimField<std::string::String>>,
}

impl BuildRepositoryRuleset_Conditions_RefName {
    pub fn build(self) -> RepositoryRuleset_Conditions_RefName {
        RepositoryRuleset_Conditions_RefName {
            exclude: self.exclude,
            include: self.include,
        }
    }
}

#[allow(dead_code)]
pub struct RepositoryRuleset_Conditions_RefNameRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for RepositoryRuleset_Conditions_RefNameRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> RepositoryRuleset_Conditions_RefNameRef {
        RepositoryRuleset_Conditions_RefNameRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl RepositoryRuleset_Conditions_RefNameRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `exclude` after provisioning.\nArray of ref names or patterns to exclude. The condition will not pass if any of these patterns match."]
    pub fn exclude(&self) -> terrars::ListRef<terrars::PrimExpr<std::string::String>> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.exclude", self.base))
    }

    #[doc = "Get a reference to the value of field `include` after provisioning.\nArray of ref names or patterns to include. One of these patterns must match for the condition to pass. Also accepts `~DEFAULT_BRANCH` to include the default branch or `~ALL` to include all branches."]
    pub fn include(&self) -> terrars::ListRef<terrars::PrimExpr<std::string::String>> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.include", self.base))
    }
}

#[derive(Serialize, Default)]
struct RepositoryRuleset_ConditionsDynamic {
    ref_name: std::option::Option<terrars::DynamicBlock<RepositoryRuleset_Conditions_RefName>>,
}

#[derive(Serialize)]
pub struct RepositoryRuleset_Conditions {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    ref_name: std::option::Option<std::vec::Vec<RepositoryRuleset_Conditions_RefName>>,
    dynamic: RepositoryRuleset_ConditionsDynamic,
}

impl RepositoryRuleset_Conditions {
    #[doc = "Set the field `ref_name`.\n"]
    pub fn set_ref_name(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<RepositoryRuleset_Conditions_RefName>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.ref_name = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.ref_name = std::option::Option::Some(d);
            }
        }
        self
    }
}

impl terrars::ToListMappable for RepositoryRuleset_Conditions {
    type O = terrars::BlockAssignable<RepositoryRuleset_Conditions>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildRepositoryRuleset_Conditions {}

impl BuildRepositoryRuleset_Conditions {
    pub fn build(self) -> RepositoryRuleset_Conditions {
        RepositoryRuleset_Conditions {
            ref_name: core::default::Default::default(),
            dynamic: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct RepositoryRuleset_ConditionsRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for RepositoryRuleset_ConditionsRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> RepositoryRuleset_ConditionsRef {
        RepositoryRuleset_ConditionsRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl RepositoryRuleset_ConditionsRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `ref_name` after provisioning.\n"]
    pub fn ref_name(&self) -> terrars::ListRef<RepositoryRuleset_Conditions_RefNameRef> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.ref_name", self.base))
    }
}

#[derive(Serialize)]
pub struct RepositoryRuleset_Rules_BranchNamePattern {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    name: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    negate: std::option::Option<terrars::PrimField<bool>>,
    operator: terrars::PrimField<std::string::String>,
    pattern: terrars::PrimField<std::string::String>,
}

impl RepositoryRuleset_Rules_BranchNamePattern {
    #[doc = "Set the field `name`.\nHow this rule will appear to users."]
    pub fn set_name(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.name = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `negate`.\nIf true, the rule will fail if the pattern matches."]
    pub fn set_negate(mut self, v: impl std::convert::Into<terrars::PrimField<bool>>) -> Self {
        self.negate = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for RepositoryRuleset_Rules_BranchNamePattern {
    type O = terrars::BlockAssignable<RepositoryRuleset_Rules_BranchNamePattern>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildRepositoryRuleset_Rules_BranchNamePattern {
    #[doc = "The operator to use for matching. Can be one of: `starts_with`, `ends_with`, `contains`, `regex`."]
    pub operator: terrars::PrimField<std::string::String>,
    #[doc = "The pattern to match with."]
    pub pattern: terrars::PrimField<std::string::String>,
}

impl BuildRepositoryRuleset_Rules_BranchNamePattern {
    pub fn build(self) -> RepositoryRuleset_Rules_BranchNamePattern {
        RepositoryRuleset_Rules_BranchNamePattern {
            name: core::default::Default::default(),
            negate: core::default::Default::default(),
            operator: self.operator,
            pattern: self.pattern,
        }
    }
}

#[allow(dead_code)]
pub struct RepositoryRuleset_Rules_BranchNamePatternRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for RepositoryRuleset_Rules_BranchNamePatternRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> RepositoryRuleset_Rules_BranchNamePatternRef {
        RepositoryRuleset_Rules_BranchNamePatternRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl RepositoryRuleset_Rules_BranchNamePatternRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `name` after provisioning.\nHow this rule will appear to users."]
    pub fn name(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.name", self.base))
    }

    #[doc = "Get a reference to the value of field `negate` after provisioning.\nIf true, the rule will fail if the pattern matches."]
    pub fn negate(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.negate", self.base))
    }

    #[doc = "Get a reference to the value of field `operator` after provisioning.\nThe operator to use for matching. Can be one of: `starts_with`, `ends_with`, `contains`, `regex`."]
    pub fn operator(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.operator", self.base))
    }

    #[doc = "Get a reference to the value of field `pattern` after provisioning.\nThe pattern to match with."]
    pub fn pattern(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.pattern", self.base))
    }
}

#[derive(Serialize)]
pub struct RepositoryRuleset_Rules_CommitAuthorEmailPattern {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    name: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    negate: std::option::Option<terrars::PrimField<bool>>,
    operator: terrars::PrimField<std::string::String>,
    pattern: terrars::PrimField<std::string::String>,
}

impl RepositoryRuleset_Rules_CommitAuthorEmailPattern {
    #[doc = "Set the field `name`.\nHow this rule will appear to users."]
    pub fn set_name(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.name = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `negate`.\nIf true, the rule will fail if the pattern matches."]
    pub fn set_negate(mut self, v: impl std::convert::Into<terrars::PrimField<bool>>) -> Self {
        self.negate = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for RepositoryRuleset_Rules_CommitAuthorEmailPattern {
    type O = terrars::BlockAssignable<RepositoryRuleset_Rules_CommitAuthorEmailPattern>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildRepositoryRuleset_Rules_CommitAuthorEmailPattern {
    #[doc = "The operator to use for matching. Can be one of: `starts_with`, `ends_with`, `contains`, `regex`."]
    pub operator: terrars::PrimField<std::string::String>,
    #[doc = "The pattern to match with."]
    pub pattern: terrars::PrimField<std::string::String>,
}

impl BuildRepositoryRuleset_Rules_CommitAuthorEmailPattern {
    pub fn build(self) -> RepositoryRuleset_Rules_CommitAuthorEmailPattern {
        RepositoryRuleset_Rules_CommitAuthorEmailPattern {
            name: core::default::Default::default(),
            negate: core::default::Default::default(),
            operator: self.operator,
            pattern: self.pattern,
        }
    }
}

#[allow(dead_code)]
pub struct RepositoryRuleset_Rules_CommitAuthorEmailPatternRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for RepositoryRuleset_Rules_CommitAuthorEmailPatternRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> RepositoryRuleset_Rules_CommitAuthorEmailPatternRef {
        RepositoryRuleset_Rules_CommitAuthorEmailPatternRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl RepositoryRuleset_Rules_CommitAuthorEmailPatternRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `name` after provisioning.\nHow this rule will appear to users."]
    pub fn name(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.name", self.base))
    }

    #[doc = "Get a reference to the value of field `negate` after provisioning.\nIf true, the rule will fail if the pattern matches."]
    pub fn negate(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.negate", self.base))
    }

    #[doc = "Get a reference to the value of field `operator` after provisioning.\nThe operator to use for matching. Can be one of: `starts_with`, `ends_with`, `contains`, `regex`."]
    pub fn operator(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.operator", self.base))
    }

    #[doc = "Get a reference to the value of field `pattern` after provisioning.\nThe pattern to match with."]
    pub fn pattern(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.pattern", self.base))
    }
}

#[derive(Serialize)]
pub struct RepositoryRuleset_Rules_CommitMessagePattern {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    name: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    negate: std::option::Option<terrars::PrimField<bool>>,
    operator: terrars::PrimField<std::string::String>,
    pattern: terrars::PrimField<std::string::String>,
}

impl RepositoryRuleset_Rules_CommitMessagePattern {
    #[doc = "Set the field `name`.\nHow this rule will appear to users."]
    pub fn set_name(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.name = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `negate`.\nIf true, the rule will fail if the pattern matches."]
    pub fn set_negate(mut self, v: impl std::convert::Into<terrars::PrimField<bool>>) -> Self {
        self.negate = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for RepositoryRuleset_Rules_CommitMessagePattern {
    type O = terrars::BlockAssignable<RepositoryRuleset_Rules_CommitMessagePattern>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildRepositoryRuleset_Rules_CommitMessagePattern {
    #[doc = "The operator to use for matching. Can be one of: `starts_with`, `ends_with`, `contains`, `regex`."]
    pub operator: terrars::PrimField<std::string::String>,
    #[doc = "The pattern to match with."]
    pub pattern: terrars::PrimField<std::string::String>,
}

impl BuildRepositoryRuleset_Rules_CommitMessagePattern {
    pub fn build(self) -> RepositoryRuleset_Rules_CommitMessagePattern {
        RepositoryRuleset_Rules_CommitMessagePattern {
            name: core::default::Default::default(),
            negate: core::default::Default::default(),
            operator: self.operator,
            pattern: self.pattern,
        }
    }
}

#[allow(dead_code)]
pub struct RepositoryRuleset_Rules_CommitMessagePatternRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for RepositoryRuleset_Rules_CommitMessagePatternRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> RepositoryRuleset_Rules_CommitMessagePatternRef {
        RepositoryRuleset_Rules_CommitMessagePatternRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl RepositoryRuleset_Rules_CommitMessagePatternRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `name` after provisioning.\nHow this rule will appear to users."]
    pub fn name(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.name", self.base))
    }

    #[doc = "Get a reference to the value of field `negate` after provisioning.\nIf true, the rule will fail if the pattern matches."]
    pub fn negate(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.negate", self.base))
    }

    #[doc = "Get a reference to the value of field `operator` after provisioning.\nThe operator to use for matching. Can be one of: `starts_with`, `ends_with`, `contains`, `regex`."]
    pub fn operator(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.operator", self.base))
    }

    #[doc = "Get a reference to the value of field `pattern` after provisioning.\nThe pattern to match with."]
    pub fn pattern(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.pattern", self.base))
    }
}

#[derive(Serialize)]
pub struct RepositoryRuleset_Rules_CommitterEmailPattern {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    name: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    negate: std::option::Option<terrars::PrimField<bool>>,
    operator: terrars::PrimField<std::string::String>,
    pattern: terrars::PrimField<std::string::String>,
}

impl RepositoryRuleset_Rules_CommitterEmailPattern {
    #[doc = "Set the field `name`.\nHow this rule will appear to users."]
    pub fn set_name(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.name = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `negate`.\nIf true, the rule will fail if the pattern matches."]
    pub fn set_negate(mut self, v: impl std::convert::Into<terrars::PrimField<bool>>) -> Self {
        self.negate = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for RepositoryRuleset_Rules_CommitterEmailPattern {
    type O = terrars::BlockAssignable<RepositoryRuleset_Rules_CommitterEmailPattern>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildRepositoryRuleset_Rules_CommitterEmailPattern {
    #[doc = "The operator to use for matching. Can be one of: `starts_with`, `ends_with`, `contains`, `regex`."]
    pub operator: terrars::PrimField<std::string::String>,
    #[doc = "The pattern to match with."]
    pub pattern: terrars::PrimField<std::string::String>,
}

impl BuildRepositoryRuleset_Rules_CommitterEmailPattern {
    pub fn build(self) -> RepositoryRuleset_Rules_CommitterEmailPattern {
        RepositoryRuleset_Rules_CommitterEmailPattern {
            name: core::default::Default::default(),
            negate: core::default::Default::default(),
            operator: self.operator,
            pattern: self.pattern,
        }
    }
}

#[allow(dead_code)]
pub struct RepositoryRuleset_Rules_CommitterEmailPatternRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for RepositoryRuleset_Rules_CommitterEmailPatternRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> RepositoryRuleset_Rules_CommitterEmailPatternRef {
        RepositoryRuleset_Rules_CommitterEmailPatternRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl RepositoryRuleset_Rules_CommitterEmailPatternRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `name` after provisioning.\nHow this rule will appear to users."]
    pub fn name(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.name", self.base))
    }

    #[doc = "Get a reference to the value of field `negate` after provisioning.\nIf true, the rule will fail if the pattern matches."]
    pub fn negate(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.negate", self.base))
    }

    #[doc = "Get a reference to the value of field `operator` after provisioning.\nThe operator to use for matching. Can be one of: `starts_with`, `ends_with`, `contains`, `regex`."]
    pub fn operator(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.operator", self.base))
    }

    #[doc = "Get a reference to the value of field `pattern` after provisioning.\nThe pattern to match with."]
    pub fn pattern(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.pattern", self.base))
    }
}

#[derive(Serialize)]
pub struct RepositoryRuleset_Rules_CopilotCodeReview {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    review_draft_pull_requests: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    review_on_push: std::option::Option<terrars::PrimField<bool>>,
}

impl RepositoryRuleset_Rules_CopilotCodeReview {
    #[doc = "Set the field `review_draft_pull_requests`.\nCopilot automatically reviews draft pull requests before they are marked as ready for review. Defaults to `false`."]
    pub fn set_review_draft_pull_requests(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.review_draft_pull_requests = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `review_on_push`.\nCopilot automatically reviews each new push to the pull request. Defaults to `false`."]
    pub fn set_review_on_push(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.review_on_push = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for RepositoryRuleset_Rules_CopilotCodeReview {
    type O = terrars::BlockAssignable<RepositoryRuleset_Rules_CopilotCodeReview>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildRepositoryRuleset_Rules_CopilotCodeReview {}

impl BuildRepositoryRuleset_Rules_CopilotCodeReview {
    pub fn build(self) -> RepositoryRuleset_Rules_CopilotCodeReview {
        RepositoryRuleset_Rules_CopilotCodeReview {
            review_draft_pull_requests: core::default::Default::default(),
            review_on_push: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct RepositoryRuleset_Rules_CopilotCodeReviewRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for RepositoryRuleset_Rules_CopilotCodeReviewRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> RepositoryRuleset_Rules_CopilotCodeReviewRef {
        RepositoryRuleset_Rules_CopilotCodeReviewRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl RepositoryRuleset_Rules_CopilotCodeReviewRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `review_draft_pull_requests` after provisioning.\nCopilot automatically reviews draft pull requests before they are marked as ready for review. Defaults to `false`."]
    pub fn review_draft_pull_requests(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.review_draft_pull_requests", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `review_on_push` after provisioning.\nCopilot automatically reviews each new push to the pull request. Defaults to `false`."]
    pub fn review_on_push(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.review_on_push", self.base),
        )
    }
}

#[derive(Serialize)]
pub struct RepositoryRuleset_Rules_FileExtensionRestriction {
    restricted_file_extensions: terrars::SetField<terrars::PrimField<std::string::String>>,
}

impl RepositoryRuleset_Rules_FileExtensionRestriction {}

impl terrars::ToListMappable for RepositoryRuleset_Rules_FileExtensionRestriction {
    type O = terrars::BlockAssignable<RepositoryRuleset_Rules_FileExtensionRestriction>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildRepositoryRuleset_Rules_FileExtensionRestriction {
    #[doc = "A list of file extensions."]
    pub restricted_file_extensions: terrars::SetField<terrars::PrimField<std::string::String>>,
}

impl BuildRepositoryRuleset_Rules_FileExtensionRestriction {
    pub fn build(self) -> RepositoryRuleset_Rules_FileExtensionRestriction {
        RepositoryRuleset_Rules_FileExtensionRestriction {
            restricted_file_extensions: self.restricted_file_extensions,
        }
    }
}

#[allow(dead_code)]
pub struct RepositoryRuleset_Rules_FileExtensionRestrictionRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for RepositoryRuleset_Rules_FileExtensionRestrictionRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> RepositoryRuleset_Rules_FileExtensionRestrictionRef {
        RepositoryRuleset_Rules_FileExtensionRestrictionRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl RepositoryRuleset_Rules_FileExtensionRestrictionRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `restricted_file_extensions` after provisioning.\nA list of file extensions."]
    pub fn restricted_file_extensions(
        &self,
    ) -> terrars::SetRef<terrars::PrimExpr<std::string::String>> {
        terrars::SetRef::new(
            self.shared().clone(),
            format!("{}.restricted_file_extensions", self.base),
        )
    }
}

#[derive(Serialize)]
pub struct RepositoryRuleset_Rules_FilePathRestriction {
    restricted_file_paths: terrars::ListField<terrars::PrimField<std::string::String>>,
}

impl RepositoryRuleset_Rules_FilePathRestriction {}

impl terrars::ToListMappable for RepositoryRuleset_Rules_FilePathRestriction {
    type O = terrars::BlockAssignable<RepositoryRuleset_Rules_FilePathRestriction>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildRepositoryRuleset_Rules_FilePathRestriction {
    #[doc = "The file paths that are restricted from being pushed to the commit graph."]
    pub restricted_file_paths: terrars::ListField<terrars::PrimField<std::string::String>>,
}

impl BuildRepositoryRuleset_Rules_FilePathRestriction {
    pub fn build(self) -> RepositoryRuleset_Rules_FilePathRestriction {
        RepositoryRuleset_Rules_FilePathRestriction {
            restricted_file_paths: self.restricted_file_paths,
        }
    }
}

#[allow(dead_code)]
pub struct RepositoryRuleset_Rules_FilePathRestrictionRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for RepositoryRuleset_Rules_FilePathRestrictionRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> RepositoryRuleset_Rules_FilePathRestrictionRef {
        RepositoryRuleset_Rules_FilePathRestrictionRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl RepositoryRuleset_Rules_FilePathRestrictionRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `restricted_file_paths` after provisioning.\nThe file paths that are restricted from being pushed to the commit graph."]
    pub fn restricted_file_paths(
        &self,
    ) -> terrars::ListRef<terrars::PrimExpr<std::string::String>> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.restricted_file_paths", self.base),
        )
    }
}

#[derive(Serialize)]
pub struct RepositoryRuleset_Rules_MaxFilePathLength {
    max_file_path_length: terrars::PrimField<f64>,
}

impl RepositoryRuleset_Rules_MaxFilePathLength {}

impl terrars::ToListMappable for RepositoryRuleset_Rules_MaxFilePathLength {
    type O = terrars::BlockAssignable<RepositoryRuleset_Rules_MaxFilePathLength>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildRepositoryRuleset_Rules_MaxFilePathLength {
    #[doc = "The maximum allowed length of a file path."]
    pub max_file_path_length: terrars::PrimField<f64>,
}

impl BuildRepositoryRuleset_Rules_MaxFilePathLength {
    pub fn build(self) -> RepositoryRuleset_Rules_MaxFilePathLength {
        RepositoryRuleset_Rules_MaxFilePathLength {
            max_file_path_length: self.max_file_path_length,
        }
    }
}

#[allow(dead_code)]
pub struct RepositoryRuleset_Rules_MaxFilePathLengthRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for RepositoryRuleset_Rules_MaxFilePathLengthRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> RepositoryRuleset_Rules_MaxFilePathLengthRef {
        RepositoryRuleset_Rules_MaxFilePathLengthRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl RepositoryRuleset_Rules_MaxFilePathLengthRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `max_file_path_length` after provisioning.\nThe maximum allowed length of a file path."]
    pub fn max_file_path_length(&self) -> terrars::PrimExpr<f64> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.max_file_path_length", self.base),
        )
    }
}

#[derive(Serialize)]
pub struct RepositoryRuleset_Rules_MaxFileSize {
    max_file_size: terrars::PrimField<f64>,
}

impl RepositoryRuleset_Rules_MaxFileSize {}

impl terrars::ToListMappable for RepositoryRuleset_Rules_MaxFileSize {
    type O = terrars::BlockAssignable<RepositoryRuleset_Rules_MaxFileSize>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildRepositoryRuleset_Rules_MaxFileSize {
    #[doc = "The maximum allowed size of a file in megabytes (MB). Valid range is 1-100 MB."]
    pub max_file_size: terrars::PrimField<f64>,
}

impl BuildRepositoryRuleset_Rules_MaxFileSize {
    pub fn build(self) -> RepositoryRuleset_Rules_MaxFileSize {
        RepositoryRuleset_Rules_MaxFileSize {
            max_file_size: self.max_file_size,
        }
    }
}

#[allow(dead_code)]
pub struct RepositoryRuleset_Rules_MaxFileSizeRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for RepositoryRuleset_Rules_MaxFileSizeRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> RepositoryRuleset_Rules_MaxFileSizeRef {
        RepositoryRuleset_Rules_MaxFileSizeRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl RepositoryRuleset_Rules_MaxFileSizeRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `max_file_size` after provisioning.\nThe maximum allowed size of a file in megabytes (MB). Valid range is 1-100 MB."]
    pub fn max_file_size(&self) -> terrars::PrimExpr<f64> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.max_file_size", self.base),
        )
    }
}

#[derive(Serialize)]
pub struct RepositoryRuleset_Rules_MergeQueue {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    check_response_timeout_minutes: std::option::Option<terrars::PrimField<f64>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    grouping_strategy: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    max_entries_to_build: std::option::Option<terrars::PrimField<f64>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    max_entries_to_merge: std::option::Option<terrars::PrimField<f64>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    merge_method: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    min_entries_to_merge: std::option::Option<terrars::PrimField<f64>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    min_entries_to_merge_wait_minutes: std::option::Option<terrars::PrimField<f64>>,
}

impl RepositoryRuleset_Rules_MergeQueue {
    #[doc = "Set the field `check_response_timeout_minutes`.\nMaximum time for a required status check to report a conclusion. After this much time has elapsed, checks that have not reported a conclusion will be assumed to have failed. Defaults to `60`."]
    pub fn set_check_response_timeout_minutes(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<f64>>,
    ) -> Self {
        self.check_response_timeout_minutes = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `grouping_strategy`.\nWhen set to ALLGREEN, the merge commit created by merge queue for each PR in the group must pass all required checks to merge. When set to HEADGREEN, only the commit at the head of the merge group, i.e. the commit containing changes from all of the PRs in the group, must pass its required checks to merge. Can be one of: ALLGREEN, HEADGREEN. Defaults to `ALLGREEN`."]
    pub fn set_grouping_strategy(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.grouping_strategy = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `max_entries_to_build`.\nLimit the number of queued pull requests requesting checks and workflow runs at the same time. Defaults to `5`."]
    pub fn set_max_entries_to_build(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<f64>>,
    ) -> Self {
        self.max_entries_to_build = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `max_entries_to_merge`.\nThe maximum number of PRs that will be merged together in a group. Defaults to `5`."]
    pub fn set_max_entries_to_merge(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<f64>>,
    ) -> Self {
        self.max_entries_to_merge = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `merge_method`.\nMethod to use when merging changes from queued pull requests. Can be one of: MERGE, SQUASH, REBASE. Defaults to `MERGE`."]
    pub fn set_merge_method(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.merge_method = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `min_entries_to_merge`.\nThe minimum number of PRs that will be merged together in a group. Defaults to `1`."]
    pub fn set_min_entries_to_merge(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<f64>>,
    ) -> Self {
        self.min_entries_to_merge = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `min_entries_to_merge_wait_minutes`.\nThe time merge queue should wait after the first PR is added to the queue for the minimum group size to be met. After this time has elapsed, the minimum group size will be ignored and a smaller group will be merged. Defaults to `5`."]
    pub fn set_min_entries_to_merge_wait_minutes(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<f64>>,
    ) -> Self {
        self.min_entries_to_merge_wait_minutes = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for RepositoryRuleset_Rules_MergeQueue {
    type O = terrars::BlockAssignable<RepositoryRuleset_Rules_MergeQueue>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildRepositoryRuleset_Rules_MergeQueue {}

impl BuildRepositoryRuleset_Rules_MergeQueue {
    pub fn build(self) -> RepositoryRuleset_Rules_MergeQueue {
        RepositoryRuleset_Rules_MergeQueue {
            check_response_timeout_minutes: core::default::Default::default(),
            grouping_strategy: core::default::Default::default(),
            max_entries_to_build: core::default::Default::default(),
            max_entries_to_merge: core::default::Default::default(),
            merge_method: core::default::Default::default(),
            min_entries_to_merge: core::default::Default::default(),
            min_entries_to_merge_wait_minutes: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct RepositoryRuleset_Rules_MergeQueueRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for RepositoryRuleset_Rules_MergeQueueRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> RepositoryRuleset_Rules_MergeQueueRef {
        RepositoryRuleset_Rules_MergeQueueRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl RepositoryRuleset_Rules_MergeQueueRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `check_response_timeout_minutes` after provisioning.\nMaximum time for a required status check to report a conclusion. After this much time has elapsed, checks that have not reported a conclusion will be assumed to have failed. Defaults to `60`."]
    pub fn check_response_timeout_minutes(&self) -> terrars::PrimExpr<f64> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.check_response_timeout_minutes", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `grouping_strategy` after provisioning.\nWhen set to ALLGREEN, the merge commit created by merge queue for each PR in the group must pass all required checks to merge. When set to HEADGREEN, only the commit at the head of the merge group, i.e. the commit containing changes from all of the PRs in the group, must pass its required checks to merge. Can be one of: ALLGREEN, HEADGREEN. Defaults to `ALLGREEN`."]
    pub fn grouping_strategy(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.grouping_strategy", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `max_entries_to_build` after provisioning.\nLimit the number of queued pull requests requesting checks and workflow runs at the same time. Defaults to `5`."]
    pub fn max_entries_to_build(&self) -> terrars::PrimExpr<f64> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.max_entries_to_build", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `max_entries_to_merge` after provisioning.\nThe maximum number of PRs that will be merged together in a group. Defaults to `5`."]
    pub fn max_entries_to_merge(&self) -> terrars::PrimExpr<f64> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.max_entries_to_merge", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `merge_method` after provisioning.\nMethod to use when merging changes from queued pull requests. Can be one of: MERGE, SQUASH, REBASE. Defaults to `MERGE`."]
    pub fn merge_method(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.merge_method", self.base))
    }

    #[doc = "Get a reference to the value of field `min_entries_to_merge` after provisioning.\nThe minimum number of PRs that will be merged together in a group. Defaults to `1`."]
    pub fn min_entries_to_merge(&self) -> terrars::PrimExpr<f64> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.min_entries_to_merge", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `min_entries_to_merge_wait_minutes` after provisioning.\nThe time merge queue should wait after the first PR is added to the queue for the minimum group size to be met. After this time has elapsed, the minimum group size will be ignored and a smaller group will be merged. Defaults to `5`."]
    pub fn min_entries_to_merge_wait_minutes(&self) -> terrars::PrimExpr<f64> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.min_entries_to_merge_wait_minutes", self.base),
        )
    }
}

#[derive(Serialize)]
pub struct RepositoryRuleset_Rules_PullRequest_RequiredReviewers_Reviewer {
    id: terrars::PrimField<f64>,
    #[serde(rename = "type")]
    type_: terrars::PrimField<std::string::String>,
}

impl RepositoryRuleset_Rules_PullRequest_RequiredReviewers_Reviewer {}

impl terrars::ToListMappable for RepositoryRuleset_Rules_PullRequest_RequiredReviewers_Reviewer {
    type O =
        terrars::BlockAssignable<RepositoryRuleset_Rules_PullRequest_RequiredReviewers_Reviewer>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildRepositoryRuleset_Rules_PullRequest_RequiredReviewers_Reviewer {
    #[doc = "The ID of the reviewer that must review."]
    pub id: terrars::PrimField<f64>,
    #[doc = "The type of reviewer. Currently only `Team` is supported."]
    pub type_: terrars::PrimField<std::string::String>,
}

impl BuildRepositoryRuleset_Rules_PullRequest_RequiredReviewers_Reviewer {
    pub fn build(self) -> RepositoryRuleset_Rules_PullRequest_RequiredReviewers_Reviewer {
        RepositoryRuleset_Rules_PullRequest_RequiredReviewers_Reviewer {
            id: self.id,
            type_: self.type_,
        }
    }
}

#[allow(dead_code)]
pub struct RepositoryRuleset_Rules_PullRequest_RequiredReviewers_ReviewerRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for RepositoryRuleset_Rules_PullRequest_RequiredReviewers_ReviewerRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> RepositoryRuleset_Rules_PullRequest_RequiredReviewers_ReviewerRef {
        RepositoryRuleset_Rules_PullRequest_RequiredReviewers_ReviewerRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl RepositoryRuleset_Rules_PullRequest_RequiredReviewers_ReviewerRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `id` after provisioning.\nThe ID of the reviewer that must review."]
    pub fn id(&self) -> terrars::PrimExpr<f64> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.id", self.base))
    }

    #[doc = "Get a reference to the value of field `type_` after provisioning.\nThe type of reviewer. Currently only `Team` is supported."]
    pub fn type_(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.type", self.base))
    }
}

#[derive(Serialize, Default)]
struct RepositoryRuleset_Rules_PullRequest_RequiredReviewersDynamic {
    reviewer: std::option::Option<
        terrars::DynamicBlock<RepositoryRuleset_Rules_PullRequest_RequiredReviewers_Reviewer>,
    >,
}

#[derive(Serialize)]
pub struct RepositoryRuleset_Rules_PullRequest_RequiredReviewers {
    file_patterns: terrars::ListField<terrars::PrimField<std::string::String>>,
    minimum_approvals: terrars::PrimField<f64>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    reviewer: std::option::Option<
        std::vec::Vec<RepositoryRuleset_Rules_PullRequest_RequiredReviewers_Reviewer>,
    >,
    dynamic: RepositoryRuleset_Rules_PullRequest_RequiredReviewersDynamic,
}

impl RepositoryRuleset_Rules_PullRequest_RequiredReviewers {
    #[doc = "Set the field `reviewer`.\n"]
    pub fn set_reviewer(
        mut self,
        v: impl std::convert::Into<
            terrars::BlockAssignable<
                RepositoryRuleset_Rules_PullRequest_RequiredReviewers_Reviewer,
            >,
        >,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.reviewer = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.reviewer = std::option::Option::Some(d);
            }
        }
        self
    }
}

impl terrars::ToListMappable for RepositoryRuleset_Rules_PullRequest_RequiredReviewers {
    type O = terrars::BlockAssignable<RepositoryRuleset_Rules_PullRequest_RequiredReviewers>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildRepositoryRuleset_Rules_PullRequest_RequiredReviewers {
    #[doc = "File patterns (fnmatch syntax) that this reviewer must approve."]
    pub file_patterns: terrars::ListField<terrars::PrimField<std::string::String>>,
    #[doc = "Minimum number of approvals required from this reviewer. Set to 0 to make approval optional."]
    pub minimum_approvals: terrars::PrimField<f64>,
}

impl BuildRepositoryRuleset_Rules_PullRequest_RequiredReviewers {
    pub fn build(self) -> RepositoryRuleset_Rules_PullRequest_RequiredReviewers {
        RepositoryRuleset_Rules_PullRequest_RequiredReviewers {
            file_patterns: self.file_patterns,
            minimum_approvals: self.minimum_approvals,
            reviewer: core::default::Default::default(),
            dynamic: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct RepositoryRuleset_Rules_PullRequest_RequiredReviewersRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for RepositoryRuleset_Rules_PullRequest_RequiredReviewersRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> RepositoryRuleset_Rules_PullRequest_RequiredReviewersRef {
        RepositoryRuleset_Rules_PullRequest_RequiredReviewersRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl RepositoryRuleset_Rules_PullRequest_RequiredReviewersRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `file_patterns` after provisioning.\nFile patterns (fnmatch syntax) that this reviewer must approve."]
    pub fn file_patterns(&self) -> terrars::ListRef<terrars::PrimExpr<std::string::String>> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.file_patterns", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `minimum_approvals` after provisioning.\nMinimum number of approvals required from this reviewer. Set to 0 to make approval optional."]
    pub fn minimum_approvals(&self) -> terrars::PrimExpr<f64> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.minimum_approvals", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `reviewer` after provisioning.\n"]
    pub fn reviewer(
        &self,
    ) -> terrars::ListRef<RepositoryRuleset_Rules_PullRequest_RequiredReviewers_ReviewerRef> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.reviewer", self.base))
    }
}

#[derive(Serialize, Default)]
struct RepositoryRuleset_Rules_PullRequestDynamic {
    required_reviewers: std::option::Option<
        terrars::DynamicBlock<RepositoryRuleset_Rules_PullRequest_RequiredReviewers>,
    >,
}

#[derive(Serialize)]
pub struct RepositoryRuleset_Rules_PullRequest {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    allowed_merge_methods:
        std::option::Option<terrars::ListField<terrars::PrimField<std::string::String>>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    dismiss_stale_reviews_on_push: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    require_code_owner_review: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    require_last_push_approval: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    required_approving_review_count: std::option::Option<terrars::PrimField<f64>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    required_review_thread_resolution: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    required_reviewers:
        std::option::Option<std::vec::Vec<RepositoryRuleset_Rules_PullRequest_RequiredReviewers>>,
    dynamic: RepositoryRuleset_Rules_PullRequestDynamic,
}

impl RepositoryRuleset_Rules_PullRequest {
    #[doc = "Set the field `allowed_merge_methods`.\nArray of allowed merge methods. Allowed values include `merge`, `squash`, and `rebase`. At least one option must be enabled."]
    pub fn set_allowed_merge_methods(
        mut self,
        v: impl std::convert::Into<terrars::ListField<terrars::PrimField<std::string::String>>>,
    ) -> Self {
        self.allowed_merge_methods = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `dismiss_stale_reviews_on_push`.\nNew, reviewable commits pushed will dismiss previous pull request review approvals. Defaults to `false`."]
    pub fn set_dismiss_stale_reviews_on_push(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.dismiss_stale_reviews_on_push = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `require_code_owner_review`.\nRequire an approving review in pull requests that modify files that have a designated code owner. Defaults to `false`."]
    pub fn set_require_code_owner_review(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.require_code_owner_review = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `require_last_push_approval`.\nWhether the most recent reviewable push must be approved by someone other than the person who pushed it. Defaults to `false`."]
    pub fn set_require_last_push_approval(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.require_last_push_approval = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `required_approving_review_count`.\nThe number of approving reviews that are required before a pull request can be merged. Defaults to `0`."]
    pub fn set_required_approving_review_count(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<f64>>,
    ) -> Self {
        self.required_approving_review_count = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `required_review_thread_resolution`.\nAll conversations on code must be resolved before a pull request can be merged. Defaults to `false`."]
    pub fn set_required_review_thread_resolution(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.required_review_thread_resolution = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `required_reviewers`.\n"]
    pub fn set_required_reviewers(
        mut self,
        v: impl std::convert::Into<
            terrars::BlockAssignable<RepositoryRuleset_Rules_PullRequest_RequiredReviewers>,
        >,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.required_reviewers = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.required_reviewers = std::option::Option::Some(d);
            }
        }
        self
    }
}

impl terrars::ToListMappable for RepositoryRuleset_Rules_PullRequest {
    type O = terrars::BlockAssignable<RepositoryRuleset_Rules_PullRequest>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildRepositoryRuleset_Rules_PullRequest {}

impl BuildRepositoryRuleset_Rules_PullRequest {
    pub fn build(self) -> RepositoryRuleset_Rules_PullRequest {
        RepositoryRuleset_Rules_PullRequest {
            allowed_merge_methods: core::default::Default::default(),
            dismiss_stale_reviews_on_push: core::default::Default::default(),
            require_code_owner_review: core::default::Default::default(),
            require_last_push_approval: core::default::Default::default(),
            required_approving_review_count: core::default::Default::default(),
            required_review_thread_resolution: core::default::Default::default(),
            required_reviewers: core::default::Default::default(),
            dynamic: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct RepositoryRuleset_Rules_PullRequestRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for RepositoryRuleset_Rules_PullRequestRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> RepositoryRuleset_Rules_PullRequestRef {
        RepositoryRuleset_Rules_PullRequestRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl RepositoryRuleset_Rules_PullRequestRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `allowed_merge_methods` after provisioning.\nArray of allowed merge methods. Allowed values include `merge`, `squash`, and `rebase`. At least one option must be enabled."]
    pub fn allowed_merge_methods(
        &self,
    ) -> terrars::ListRef<terrars::PrimExpr<std::string::String>> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.allowed_merge_methods", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `dismiss_stale_reviews_on_push` after provisioning.\nNew, reviewable commits pushed will dismiss previous pull request review approvals. Defaults to `false`."]
    pub fn dismiss_stale_reviews_on_push(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.dismiss_stale_reviews_on_push", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `require_code_owner_review` after provisioning.\nRequire an approving review in pull requests that modify files that have a designated code owner. Defaults to `false`."]
    pub fn require_code_owner_review(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.require_code_owner_review", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `require_last_push_approval` after provisioning.\nWhether the most recent reviewable push must be approved by someone other than the person who pushed it. Defaults to `false`."]
    pub fn require_last_push_approval(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.require_last_push_approval", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `required_approving_review_count` after provisioning.\nThe number of approving reviews that are required before a pull request can be merged. Defaults to `0`."]
    pub fn required_approving_review_count(&self) -> terrars::PrimExpr<f64> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.required_approving_review_count", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `required_review_thread_resolution` after provisioning.\nAll conversations on code must be resolved before a pull request can be merged. Defaults to `false`."]
    pub fn required_review_thread_resolution(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.required_review_thread_resolution", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `required_reviewers` after provisioning.\n"]
    pub fn required_reviewers(
        &self,
    ) -> terrars::ListRef<RepositoryRuleset_Rules_PullRequest_RequiredReviewersRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.required_reviewers", self.base),
        )
    }
}

#[derive(Serialize)]
pub struct RepositoryRuleset_Rules_RequiredCodeScanning_RequiredCodeScanningTool {
    alerts_threshold: terrars::PrimField<std::string::String>,
    security_alerts_threshold: terrars::PrimField<std::string::String>,
    tool: terrars::PrimField<std::string::String>,
}

impl RepositoryRuleset_Rules_RequiredCodeScanning_RequiredCodeScanningTool {}

impl terrars::ToListMappable
    for RepositoryRuleset_Rules_RequiredCodeScanning_RequiredCodeScanningTool
{
    type O = terrars::BlockAssignable<
        RepositoryRuleset_Rules_RequiredCodeScanning_RequiredCodeScanningTool,
    >;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildRepositoryRuleset_Rules_RequiredCodeScanning_RequiredCodeScanningTool {
    #[doc = "The severity level at which code scanning results that raise alerts block a reference update. Can be one of: `none`, `errors`, `errors_and_warnings`, `all`."]
    pub alerts_threshold: terrars::PrimField<std::string::String>,
    #[doc = "The severity level at which code scanning results that raise security alerts block a reference update. Can be one of: `none`, `critical`, `high_or_higher`, `medium_or_higher`, `all`."]
    pub security_alerts_threshold: terrars::PrimField<std::string::String>,
    #[doc = "The name of a code scanning tool"]
    pub tool: terrars::PrimField<std::string::String>,
}

impl BuildRepositoryRuleset_Rules_RequiredCodeScanning_RequiredCodeScanningTool {
    pub fn build(self) -> RepositoryRuleset_Rules_RequiredCodeScanning_RequiredCodeScanningTool {
        RepositoryRuleset_Rules_RequiredCodeScanning_RequiredCodeScanningTool {
            alerts_threshold: self.alerts_threshold,
            security_alerts_threshold: self.security_alerts_threshold,
            tool: self.tool,
        }
    }
}

#[allow(dead_code)]
pub struct RepositoryRuleset_Rules_RequiredCodeScanning_RequiredCodeScanningToolRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for RepositoryRuleset_Rules_RequiredCodeScanning_RequiredCodeScanningToolRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> RepositoryRuleset_Rules_RequiredCodeScanning_RequiredCodeScanningToolRef {
        RepositoryRuleset_Rules_RequiredCodeScanning_RequiredCodeScanningToolRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl RepositoryRuleset_Rules_RequiredCodeScanning_RequiredCodeScanningToolRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `alerts_threshold` after provisioning.\nThe severity level at which code scanning results that raise alerts block a reference update. Can be one of: `none`, `errors`, `errors_and_warnings`, `all`."]
    pub fn alerts_threshold(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.alerts_threshold", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `security_alerts_threshold` after provisioning.\nThe severity level at which code scanning results that raise security alerts block a reference update. Can be one of: `none`, `critical`, `high_or_higher`, `medium_or_higher`, `all`."]
    pub fn security_alerts_threshold(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.security_alerts_threshold", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `tool` after provisioning.\nThe name of a code scanning tool"]
    pub fn tool(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.tool", self.base))
    }
}

#[derive(Serialize, Default)]
struct RepositoryRuleset_Rules_RequiredCodeScanningDynamic {
    required_code_scanning_tool: std::option::Option<
        terrars::DynamicBlock<
            RepositoryRuleset_Rules_RequiredCodeScanning_RequiredCodeScanningTool,
        >,
    >,
}

#[derive(Serialize)]
pub struct RepositoryRuleset_Rules_RequiredCodeScanning {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    required_code_scanning_tool: std::option::Option<
        std::vec::Vec<RepositoryRuleset_Rules_RequiredCodeScanning_RequiredCodeScanningTool>,
    >,
    dynamic: RepositoryRuleset_Rules_RequiredCodeScanningDynamic,
}

impl RepositoryRuleset_Rules_RequiredCodeScanning {
    #[doc = "Set the field `required_code_scanning_tool`.\n"]
    pub fn set_required_code_scanning_tool(
        mut self,
        v: impl std::convert::Into<
            terrars::BlockAssignable<
                RepositoryRuleset_Rules_RequiredCodeScanning_RequiredCodeScanningTool,
            >,
        >,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.required_code_scanning_tool = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.required_code_scanning_tool = std::option::Option::Some(d);
            }
        }
        self
    }
}

impl terrars::ToListMappable for RepositoryRuleset_Rules_RequiredCodeScanning {
    type O = terrars::BlockAssignable<RepositoryRuleset_Rules_RequiredCodeScanning>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildRepositoryRuleset_Rules_RequiredCodeScanning {}

impl BuildRepositoryRuleset_Rules_RequiredCodeScanning {
    pub fn build(self) -> RepositoryRuleset_Rules_RequiredCodeScanning {
        RepositoryRuleset_Rules_RequiredCodeScanning {
            required_code_scanning_tool: core::default::Default::default(),
            dynamic: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct RepositoryRuleset_Rules_RequiredCodeScanningRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for RepositoryRuleset_Rules_RequiredCodeScanningRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> RepositoryRuleset_Rules_RequiredCodeScanningRef {
        RepositoryRuleset_Rules_RequiredCodeScanningRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl RepositoryRuleset_Rules_RequiredCodeScanningRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }
}

#[derive(Serialize)]
pub struct RepositoryRuleset_Rules_RequiredDeployments {
    required_deployment_environments: terrars::ListField<terrars::PrimField<std::string::String>>,
}

impl RepositoryRuleset_Rules_RequiredDeployments {}

impl terrars::ToListMappable for RepositoryRuleset_Rules_RequiredDeployments {
    type O = terrars::BlockAssignable<RepositoryRuleset_Rules_RequiredDeployments>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildRepositoryRuleset_Rules_RequiredDeployments {
    #[doc = "The environments that must be successfully deployed to before branches can be merged."]
    pub required_deployment_environments:
        terrars::ListField<terrars::PrimField<std::string::String>>,
}

impl BuildRepositoryRuleset_Rules_RequiredDeployments {
    pub fn build(self) -> RepositoryRuleset_Rules_RequiredDeployments {
        RepositoryRuleset_Rules_RequiredDeployments {
            required_deployment_environments: self.required_deployment_environments,
        }
    }
}

#[allow(dead_code)]
pub struct RepositoryRuleset_Rules_RequiredDeploymentsRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for RepositoryRuleset_Rules_RequiredDeploymentsRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> RepositoryRuleset_Rules_RequiredDeploymentsRef {
        RepositoryRuleset_Rules_RequiredDeploymentsRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl RepositoryRuleset_Rules_RequiredDeploymentsRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `required_deployment_environments` after provisioning.\nThe environments that must be successfully deployed to before branches can be merged."]
    pub fn required_deployment_environments(
        &self,
    ) -> terrars::ListRef<terrars::PrimExpr<std::string::String>> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.required_deployment_environments", self.base),
        )
    }
}

#[derive(Serialize)]
pub struct RepositoryRuleset_Rules_RequiredStatusChecks_RequiredCheck {
    context: terrars::PrimField<std::string::String>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    integration_id: std::option::Option<terrars::PrimField<f64>>,
}

impl RepositoryRuleset_Rules_RequiredStatusChecks_RequiredCheck {
    #[doc = "Set the field `integration_id`.\nThe optional integration ID that this status check must originate from."]
    pub fn set_integration_id(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<f64>>,
    ) -> Self {
        self.integration_id = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for RepositoryRuleset_Rules_RequiredStatusChecks_RequiredCheck {
    type O = terrars::BlockAssignable<RepositoryRuleset_Rules_RequiredStatusChecks_RequiredCheck>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildRepositoryRuleset_Rules_RequiredStatusChecks_RequiredCheck {
    #[doc = "The status check context name that must be present on the commit."]
    pub context: terrars::PrimField<std::string::String>,
}

impl BuildRepositoryRuleset_Rules_RequiredStatusChecks_RequiredCheck {
    pub fn build(self) -> RepositoryRuleset_Rules_RequiredStatusChecks_RequiredCheck {
        RepositoryRuleset_Rules_RequiredStatusChecks_RequiredCheck {
            context: self.context,
            integration_id: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct RepositoryRuleset_Rules_RequiredStatusChecks_RequiredCheckRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for RepositoryRuleset_Rules_RequiredStatusChecks_RequiredCheckRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> RepositoryRuleset_Rules_RequiredStatusChecks_RequiredCheckRef {
        RepositoryRuleset_Rules_RequiredStatusChecks_RequiredCheckRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl RepositoryRuleset_Rules_RequiredStatusChecks_RequiredCheckRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `context` after provisioning.\nThe status check context name that must be present on the commit."]
    pub fn context(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.context", self.base))
    }

    #[doc = "Get a reference to the value of field `integration_id` after provisioning.\nThe optional integration ID that this status check must originate from."]
    pub fn integration_id(&self) -> terrars::PrimExpr<f64> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.integration_id", self.base),
        )
    }
}

#[derive(Serialize, Default)]
struct RepositoryRuleset_Rules_RequiredStatusChecksDynamic {
    required_check: std::option::Option<
        terrars::DynamicBlock<RepositoryRuleset_Rules_RequiredStatusChecks_RequiredCheck>,
    >,
}

#[derive(Serialize)]
pub struct RepositoryRuleset_Rules_RequiredStatusChecks {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    do_not_enforce_on_create: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    strict_required_status_checks_policy: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    required_check: std::option::Option<
        std::vec::Vec<RepositoryRuleset_Rules_RequiredStatusChecks_RequiredCheck>,
    >,
    dynamic: RepositoryRuleset_Rules_RequiredStatusChecksDynamic,
}

impl RepositoryRuleset_Rules_RequiredStatusChecks {
    #[doc = "Set the field `do_not_enforce_on_create`.\nAllow repositories and branches to be created if a check would otherwise prohibit it."]
    pub fn set_do_not_enforce_on_create(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.do_not_enforce_on_create = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `strict_required_status_checks_policy`.\nWhether pull requests targeting a matching branch must be tested with the latest code. This setting will not take effect unless at least one status check is enabled. Defaults to `false`."]
    pub fn set_strict_required_status_checks_policy(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.strict_required_status_checks_policy = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `required_check`.\n"]
    pub fn set_required_check(
        mut self,
        v: impl std::convert::Into<
            terrars::BlockAssignable<RepositoryRuleset_Rules_RequiredStatusChecks_RequiredCheck>,
        >,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.required_check = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.required_check = std::option::Option::Some(d);
            }
        }
        self
    }
}

impl terrars::ToListMappable for RepositoryRuleset_Rules_RequiredStatusChecks {
    type O = terrars::BlockAssignable<RepositoryRuleset_Rules_RequiredStatusChecks>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildRepositoryRuleset_Rules_RequiredStatusChecks {}

impl BuildRepositoryRuleset_Rules_RequiredStatusChecks {
    pub fn build(self) -> RepositoryRuleset_Rules_RequiredStatusChecks {
        RepositoryRuleset_Rules_RequiredStatusChecks {
            do_not_enforce_on_create: core::default::Default::default(),
            strict_required_status_checks_policy: core::default::Default::default(),
            required_check: core::default::Default::default(),
            dynamic: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct RepositoryRuleset_Rules_RequiredStatusChecksRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for RepositoryRuleset_Rules_RequiredStatusChecksRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> RepositoryRuleset_Rules_RequiredStatusChecksRef {
        RepositoryRuleset_Rules_RequiredStatusChecksRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl RepositoryRuleset_Rules_RequiredStatusChecksRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `do_not_enforce_on_create` after provisioning.\nAllow repositories and branches to be created if a check would otherwise prohibit it."]
    pub fn do_not_enforce_on_create(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.do_not_enforce_on_create", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `strict_required_status_checks_policy` after provisioning.\nWhether pull requests targeting a matching branch must be tested with the latest code. This setting will not take effect unless at least one status check is enabled. Defaults to `false`."]
    pub fn strict_required_status_checks_policy(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.strict_required_status_checks_policy", self.base),
        )
    }
}

#[derive(Serialize)]
pub struct RepositoryRuleset_Rules_TagNamePattern {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    name: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    negate: std::option::Option<terrars::PrimField<bool>>,
    operator: terrars::PrimField<std::string::String>,
    pattern: terrars::PrimField<std::string::String>,
}

impl RepositoryRuleset_Rules_TagNamePattern {
    #[doc = "Set the field `name`.\nHow this rule will appear to users."]
    pub fn set_name(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.name = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `negate`.\nIf true, the rule will fail if the pattern matches."]
    pub fn set_negate(mut self, v: impl std::convert::Into<terrars::PrimField<bool>>) -> Self {
        self.negate = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for RepositoryRuleset_Rules_TagNamePattern {
    type O = terrars::BlockAssignable<RepositoryRuleset_Rules_TagNamePattern>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildRepositoryRuleset_Rules_TagNamePattern {
    #[doc = "The operator to use for matching. Can be one of: `starts_with`, `ends_with`, `contains`, `regex`."]
    pub operator: terrars::PrimField<std::string::String>,
    #[doc = "The pattern to match with."]
    pub pattern: terrars::PrimField<std::string::String>,
}

impl BuildRepositoryRuleset_Rules_TagNamePattern {
    pub fn build(self) -> RepositoryRuleset_Rules_TagNamePattern {
        RepositoryRuleset_Rules_TagNamePattern {
            name: core::default::Default::default(),
            negate: core::default::Default::default(),
            operator: self.operator,
            pattern: self.pattern,
        }
    }
}

#[allow(dead_code)]
pub struct RepositoryRuleset_Rules_TagNamePatternRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for RepositoryRuleset_Rules_TagNamePatternRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> RepositoryRuleset_Rules_TagNamePatternRef {
        RepositoryRuleset_Rules_TagNamePatternRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl RepositoryRuleset_Rules_TagNamePatternRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `name` after provisioning.\nHow this rule will appear to users."]
    pub fn name(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.name", self.base))
    }

    #[doc = "Get a reference to the value of field `negate` after provisioning.\nIf true, the rule will fail if the pattern matches."]
    pub fn negate(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.negate", self.base))
    }

    #[doc = "Get a reference to the value of field `operator` after provisioning.\nThe operator to use for matching. Can be one of: `starts_with`, `ends_with`, `contains`, `regex`."]
    pub fn operator(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.operator", self.base))
    }

    #[doc = "Get a reference to the value of field `pattern` after provisioning.\nThe pattern to match with."]
    pub fn pattern(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.pattern", self.base))
    }
}

#[derive(Serialize, Default)]
struct RepositoryRuleset_RulesDynamic {
    branch_name_pattern:
        std::option::Option<terrars::DynamicBlock<RepositoryRuleset_Rules_BranchNamePattern>>,
    commit_author_email_pattern: std::option::Option<
        terrars::DynamicBlock<RepositoryRuleset_Rules_CommitAuthorEmailPattern>,
    >,
    commit_message_pattern:
        std::option::Option<terrars::DynamicBlock<RepositoryRuleset_Rules_CommitMessagePattern>>,
    committer_email_pattern:
        std::option::Option<terrars::DynamicBlock<RepositoryRuleset_Rules_CommitterEmailPattern>>,
    copilot_code_review:
        std::option::Option<terrars::DynamicBlock<RepositoryRuleset_Rules_CopilotCodeReview>>,
    file_extension_restriction: std::option::Option<
        terrars::DynamicBlock<RepositoryRuleset_Rules_FileExtensionRestriction>,
    >,
    file_path_restriction:
        std::option::Option<terrars::DynamicBlock<RepositoryRuleset_Rules_FilePathRestriction>>,
    max_file_path_length:
        std::option::Option<terrars::DynamicBlock<RepositoryRuleset_Rules_MaxFilePathLength>>,
    max_file_size: std::option::Option<terrars::DynamicBlock<RepositoryRuleset_Rules_MaxFileSize>>,
    merge_queue: std::option::Option<terrars::DynamicBlock<RepositoryRuleset_Rules_MergeQueue>>,
    pull_request: std::option::Option<terrars::DynamicBlock<RepositoryRuleset_Rules_PullRequest>>,
    required_code_scanning:
        std::option::Option<terrars::DynamicBlock<RepositoryRuleset_Rules_RequiredCodeScanning>>,
    required_deployments:
        std::option::Option<terrars::DynamicBlock<RepositoryRuleset_Rules_RequiredDeployments>>,
    required_status_checks:
        std::option::Option<terrars::DynamicBlock<RepositoryRuleset_Rules_RequiredStatusChecks>>,
    tag_name_pattern:
        std::option::Option<terrars::DynamicBlock<RepositoryRuleset_Rules_TagNamePattern>>,
}

#[derive(Serialize)]
pub struct RepositoryRuleset_Rules {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    creation: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    deletion: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    non_fast_forward: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    required_linear_history: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    required_signatures: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    update: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    update_allows_fetch_and_merge: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    branch_name_pattern:
        std::option::Option<std::vec::Vec<RepositoryRuleset_Rules_BranchNamePattern>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    commit_author_email_pattern:
        std::option::Option<std::vec::Vec<RepositoryRuleset_Rules_CommitAuthorEmailPattern>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    commit_message_pattern:
        std::option::Option<std::vec::Vec<RepositoryRuleset_Rules_CommitMessagePattern>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    committer_email_pattern:
        std::option::Option<std::vec::Vec<RepositoryRuleset_Rules_CommitterEmailPattern>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    copilot_code_review:
        std::option::Option<std::vec::Vec<RepositoryRuleset_Rules_CopilotCodeReview>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    file_extension_restriction:
        std::option::Option<std::vec::Vec<RepositoryRuleset_Rules_FileExtensionRestriction>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    file_path_restriction:
        std::option::Option<std::vec::Vec<RepositoryRuleset_Rules_FilePathRestriction>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    max_file_path_length:
        std::option::Option<std::vec::Vec<RepositoryRuleset_Rules_MaxFilePathLength>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    max_file_size: std::option::Option<std::vec::Vec<RepositoryRuleset_Rules_MaxFileSize>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    merge_queue: std::option::Option<std::vec::Vec<RepositoryRuleset_Rules_MergeQueue>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pull_request: std::option::Option<std::vec::Vec<RepositoryRuleset_Rules_PullRequest>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    required_code_scanning:
        std::option::Option<std::vec::Vec<RepositoryRuleset_Rules_RequiredCodeScanning>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    required_deployments:
        std::option::Option<std::vec::Vec<RepositoryRuleset_Rules_RequiredDeployments>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    required_status_checks:
        std::option::Option<std::vec::Vec<RepositoryRuleset_Rules_RequiredStatusChecks>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    tag_name_pattern: std::option::Option<std::vec::Vec<RepositoryRuleset_Rules_TagNamePattern>>,
    dynamic: RepositoryRuleset_RulesDynamic,
}

impl RepositoryRuleset_Rules {
    #[doc = "Set the field `creation`.\nOnly allow users with bypass permission to create matching refs."]
    pub fn set_creation(mut self, v: impl std::convert::Into<terrars::PrimField<bool>>) -> Self {
        self.creation = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `deletion`.\nOnly allow users with bypass permissions to delete matching refs."]
    pub fn set_deletion(mut self, v: impl std::convert::Into<terrars::PrimField<bool>>) -> Self {
        self.deletion = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `non_fast_forward`.\nPrevent users with push access from force pushing to branches."]
    pub fn set_non_fast_forward(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.non_fast_forward = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `required_linear_history`.\nPrevent merge commits from being pushed to matching branches."]
    pub fn set_required_linear_history(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.required_linear_history = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `required_signatures`.\nCommits pushed to matching branches must have verified signatures."]
    pub fn set_required_signatures(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.required_signatures = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `update`.\nOnly allow users with bypass permission to update matching refs."]
    pub fn set_update(mut self, v: impl std::convert::Into<terrars::PrimField<bool>>) -> Self {
        self.update = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `update_allows_fetch_and_merge`.\nBranch can pull changes from its upstream repository. This is only applicable to forked repositories. Requires `update` to be set to `true`."]
    pub fn set_update_allows_fetch_and_merge(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.update_allows_fetch_and_merge = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `branch_name_pattern`.\n"]
    pub fn set_branch_name_pattern(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<RepositoryRuleset_Rules_BranchNamePattern>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.branch_name_pattern = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.branch_name_pattern = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `commit_author_email_pattern`.\n"]
    pub fn set_commit_author_email_pattern(
        mut self,
        v: impl std::convert::Into<
            terrars::BlockAssignable<RepositoryRuleset_Rules_CommitAuthorEmailPattern>,
        >,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.commit_author_email_pattern = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.commit_author_email_pattern = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `commit_message_pattern`.\n"]
    pub fn set_commit_message_pattern(
        mut self,
        v: impl std::convert::Into<
            terrars::BlockAssignable<RepositoryRuleset_Rules_CommitMessagePattern>,
        >,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.commit_message_pattern = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.commit_message_pattern = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `committer_email_pattern`.\n"]
    pub fn set_committer_email_pattern(
        mut self,
        v: impl std::convert::Into<
            terrars::BlockAssignable<RepositoryRuleset_Rules_CommitterEmailPattern>,
        >,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.committer_email_pattern = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.committer_email_pattern = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `copilot_code_review`.\n"]
    pub fn set_copilot_code_review(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<RepositoryRuleset_Rules_CopilotCodeReview>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.copilot_code_review = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.copilot_code_review = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `file_extension_restriction`.\n"]
    pub fn set_file_extension_restriction(
        mut self,
        v: impl std::convert::Into<
            terrars::BlockAssignable<RepositoryRuleset_Rules_FileExtensionRestriction>,
        >,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.file_extension_restriction = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.file_extension_restriction = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `file_path_restriction`.\n"]
    pub fn set_file_path_restriction(
        mut self,
        v: impl std::convert::Into<
            terrars::BlockAssignable<RepositoryRuleset_Rules_FilePathRestriction>,
        >,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.file_path_restriction = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.file_path_restriction = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `max_file_path_length`.\n"]
    pub fn set_max_file_path_length(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<RepositoryRuleset_Rules_MaxFilePathLength>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.max_file_path_length = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.max_file_path_length = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `max_file_size`.\n"]
    pub fn set_max_file_size(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<RepositoryRuleset_Rules_MaxFileSize>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.max_file_size = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.max_file_size = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `merge_queue`.\n"]
    pub fn set_merge_queue(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<RepositoryRuleset_Rules_MergeQueue>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.merge_queue = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.merge_queue = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `pull_request`.\n"]
    pub fn set_pull_request(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<RepositoryRuleset_Rules_PullRequest>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.pull_request = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.pull_request = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `required_code_scanning`.\n"]
    pub fn set_required_code_scanning(
        mut self,
        v: impl std::convert::Into<
            terrars::BlockAssignable<RepositoryRuleset_Rules_RequiredCodeScanning>,
        >,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.required_code_scanning = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.required_code_scanning = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `required_deployments`.\n"]
    pub fn set_required_deployments(
        mut self,
        v: impl std::convert::Into<
            terrars::BlockAssignable<RepositoryRuleset_Rules_RequiredDeployments>,
        >,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.required_deployments = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.required_deployments = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `required_status_checks`.\n"]
    pub fn set_required_status_checks(
        mut self,
        v: impl std::convert::Into<
            terrars::BlockAssignable<RepositoryRuleset_Rules_RequiredStatusChecks>,
        >,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.required_status_checks = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.required_status_checks = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `tag_name_pattern`.\n"]
    pub fn set_tag_name_pattern(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<RepositoryRuleset_Rules_TagNamePattern>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.tag_name_pattern = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.tag_name_pattern = std::option::Option::Some(d);
            }
        }
        self
    }
}

impl terrars::ToListMappable for RepositoryRuleset_Rules {
    type O = terrars::BlockAssignable<RepositoryRuleset_Rules>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildRepositoryRuleset_Rules {}

impl BuildRepositoryRuleset_Rules {
    pub fn build(self) -> RepositoryRuleset_Rules {
        RepositoryRuleset_Rules {
            creation: core::default::Default::default(),
            deletion: core::default::Default::default(),
            non_fast_forward: core::default::Default::default(),
            required_linear_history: core::default::Default::default(),
            required_signatures: core::default::Default::default(),
            update: core::default::Default::default(),
            update_allows_fetch_and_merge: core::default::Default::default(),
            branch_name_pattern: core::default::Default::default(),
            commit_author_email_pattern: core::default::Default::default(),
            commit_message_pattern: core::default::Default::default(),
            committer_email_pattern: core::default::Default::default(),
            copilot_code_review: core::default::Default::default(),
            file_extension_restriction: core::default::Default::default(),
            file_path_restriction: core::default::Default::default(),
            max_file_path_length: core::default::Default::default(),
            max_file_size: core::default::Default::default(),
            merge_queue: core::default::Default::default(),
            pull_request: core::default::Default::default(),
            required_code_scanning: core::default::Default::default(),
            required_deployments: core::default::Default::default(),
            required_status_checks: core::default::Default::default(),
            tag_name_pattern: core::default::Default::default(),
            dynamic: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct RepositoryRuleset_RulesRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for RepositoryRuleset_RulesRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> RepositoryRuleset_RulesRef {
        RepositoryRuleset_RulesRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl RepositoryRuleset_RulesRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `creation` after provisioning.\nOnly allow users with bypass permission to create matching refs."]
    pub fn creation(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.creation", self.base))
    }

    #[doc = "Get a reference to the value of field `deletion` after provisioning.\nOnly allow users with bypass permissions to delete matching refs."]
    pub fn deletion(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.deletion", self.base))
    }

    #[doc = "Get a reference to the value of field `non_fast_forward` after provisioning.\nPrevent users with push access from force pushing to branches."]
    pub fn non_fast_forward(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.non_fast_forward", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `required_linear_history` after provisioning.\nPrevent merge commits from being pushed to matching branches."]
    pub fn required_linear_history(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.required_linear_history", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `required_signatures` after provisioning.\nCommits pushed to matching branches must have verified signatures."]
    pub fn required_signatures(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.required_signatures", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `update` after provisioning.\nOnly allow users with bypass permission to update matching refs."]
    pub fn update(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.update", self.base))
    }

    #[doc = "Get a reference to the value of field `update_allows_fetch_and_merge` after provisioning.\nBranch can pull changes from its upstream repository. This is only applicable to forked repositories. Requires `update` to be set to `true`."]
    pub fn update_allows_fetch_and_merge(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.update_allows_fetch_and_merge", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `branch_name_pattern` after provisioning.\n"]
    pub fn branch_name_pattern(
        &self,
    ) -> terrars::ListRef<RepositoryRuleset_Rules_BranchNamePatternRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.branch_name_pattern", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `commit_author_email_pattern` after provisioning.\n"]
    pub fn commit_author_email_pattern(
        &self,
    ) -> terrars::ListRef<RepositoryRuleset_Rules_CommitAuthorEmailPatternRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.commit_author_email_pattern", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `commit_message_pattern` after provisioning.\n"]
    pub fn commit_message_pattern(
        &self,
    ) -> terrars::ListRef<RepositoryRuleset_Rules_CommitMessagePatternRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.commit_message_pattern", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `committer_email_pattern` after provisioning.\n"]
    pub fn committer_email_pattern(
        &self,
    ) -> terrars::ListRef<RepositoryRuleset_Rules_CommitterEmailPatternRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.committer_email_pattern", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `copilot_code_review` after provisioning.\n"]
    pub fn copilot_code_review(
        &self,
    ) -> terrars::ListRef<RepositoryRuleset_Rules_CopilotCodeReviewRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.copilot_code_review", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `file_extension_restriction` after provisioning.\n"]
    pub fn file_extension_restriction(
        &self,
    ) -> terrars::ListRef<RepositoryRuleset_Rules_FileExtensionRestrictionRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.file_extension_restriction", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `file_path_restriction` after provisioning.\n"]
    pub fn file_path_restriction(
        &self,
    ) -> terrars::ListRef<RepositoryRuleset_Rules_FilePathRestrictionRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.file_path_restriction", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `max_file_path_length` after provisioning.\n"]
    pub fn max_file_path_length(
        &self,
    ) -> terrars::ListRef<RepositoryRuleset_Rules_MaxFilePathLengthRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.max_file_path_length", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `max_file_size` after provisioning.\n"]
    pub fn max_file_size(&self) -> terrars::ListRef<RepositoryRuleset_Rules_MaxFileSizeRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.max_file_size", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `merge_queue` after provisioning.\n"]
    pub fn merge_queue(&self) -> terrars::ListRef<RepositoryRuleset_Rules_MergeQueueRef> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.merge_queue", self.base))
    }

    #[doc = "Get a reference to the value of field `pull_request` after provisioning.\n"]
    pub fn pull_request(&self) -> terrars::ListRef<RepositoryRuleset_Rules_PullRequestRef> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.pull_request", self.base))
    }

    #[doc = "Get a reference to the value of field `required_code_scanning` after provisioning.\n"]
    pub fn required_code_scanning(
        &self,
    ) -> terrars::ListRef<RepositoryRuleset_Rules_RequiredCodeScanningRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.required_code_scanning", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `required_deployments` after provisioning.\n"]
    pub fn required_deployments(
        &self,
    ) -> terrars::ListRef<RepositoryRuleset_Rules_RequiredDeploymentsRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.required_deployments", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `required_status_checks` after provisioning.\n"]
    pub fn required_status_checks(
        &self,
    ) -> terrars::ListRef<RepositoryRuleset_Rules_RequiredStatusChecksRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.required_status_checks", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `tag_name_pattern` after provisioning.\n"]
    pub fn tag_name_pattern(&self) -> terrars::ListRef<RepositoryRuleset_Rules_TagNamePatternRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.tag_name_pattern", self.base),
        )
    }
}
