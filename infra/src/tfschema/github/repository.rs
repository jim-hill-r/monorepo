#![allow(non_camel_case_types)]

use super::provider::ProviderGithub;
use serde::Serialize;
use terrars::{Ref as _, Referable as _, Resource_ as _};

#[derive(Serialize)]
struct RepositoryData_ {
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
    allow_auto_merge: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    allow_forking: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    allow_merge_commit: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    allow_rebase_merge: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    allow_squash_merge: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    allow_update_branch: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    archive_on_destroy: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    archived: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    auto_init: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    default_branch: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    delete_branch_on_merge: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    description: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    etag: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    fork: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    gitignore_template: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    has_discussions: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    has_downloads: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    has_issues: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    has_projects: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    has_wiki: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    homepage_url: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    id: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    ignore_vulnerability_alerts_during_read: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    is_template: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    license_template: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    merge_commit_message: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    merge_commit_title: std::option::Option<terrars::PrimField<std::string::String>>,
    name: terrars::PrimField<std::string::String>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    private: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    source_owner: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    source_repo: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    squash_merge_commit_message: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    squash_merge_commit_title: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    topics: std::option::Option<terrars::SetField<terrars::PrimField<std::string::String>>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    visibility: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    vulnerability_alerts: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    web_commit_signoff_required: std::option::Option<terrars::PrimField<bool>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pages: std::option::Option<std::vec::Vec<Repository_Pages>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    security_and_analysis: std::option::Option<std::vec::Vec<Repository_SecurityAndAnalysis>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    template: std::option::Option<std::vec::Vec<Repository_Template>>,
    dynamic: RepositoryDynamic,
}

struct Repository_ {
    shared: terrars::StackShared,
    tf_id: std::string::String,
    data: std::cell::RefCell<RepositoryData_>,
}

#[derive(Clone)]
pub struct Repository(std::rc::Rc<Repository_>);

impl Repository {
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

    #[doc = "Set the field `allow_auto_merge`.\nSet to 'true' to allow auto-merging pull requests on the repository."]
    pub fn set_allow_auto_merge(
        self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.0.data.borrow_mut().allow_auto_merge = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `allow_forking`.\nSet to 'true' to allow private forking on the repository; this is only relevant if the repository is owned by an organization and is private or internal."]
    pub fn set_allow_forking(self, v: impl std::convert::Into<terrars::PrimField<bool>>) -> Self {
        self.0.data.borrow_mut().allow_forking = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `allow_merge_commit`.\nSet to 'false' to disable merge commits on the repository."]
    pub fn set_allow_merge_commit(
        self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.0.data.borrow_mut().allow_merge_commit = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `allow_rebase_merge`.\nSet to 'false' to disable rebase merges on the repository."]
    pub fn set_allow_rebase_merge(
        self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.0.data.borrow_mut().allow_rebase_merge = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `allow_squash_merge`.\nSet to 'false' to disable squash merges on the repository."]
    pub fn set_allow_squash_merge(
        self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.0.data.borrow_mut().allow_squash_merge = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `allow_update_branch`.\n Set to 'true' to always suggest updating pull request branches."]
    pub fn set_allow_update_branch(
        self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.0.data.borrow_mut().allow_update_branch = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `archive_on_destroy`.\nSet to 'true' to archive the repository instead of deleting on destroy."]
    pub fn set_archive_on_destroy(
        self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.0.data.borrow_mut().archive_on_destroy = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `archived`.\nSpecifies if the repository should be archived. Defaults to 'false'. NOTE Currently, the API does not support unarchiving."]
    pub fn set_archived(self, v: impl std::convert::Into<terrars::PrimField<bool>>) -> Self {
        self.0.data.borrow_mut().archived = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `auto_init`.\nSet to 'true' to produce an initial commit in the repository."]
    pub fn set_auto_init(self, v: impl std::convert::Into<terrars::PrimField<bool>>) -> Self {
        self.0.data.borrow_mut().auto_init = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `default_branch`.\nCan only be set after initial repository creation, and only if the target branch exists"]
    pub fn set_default_branch(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().default_branch = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `delete_branch_on_merge`.\nAutomatically delete head branch after a pull request is merged. Defaults to 'false'."]
    pub fn set_delete_branch_on_merge(
        self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.0.data.borrow_mut().delete_branch_on_merge = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `description`.\nA description of the repository."]
    pub fn set_description(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().description = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `etag`.\n"]
    pub fn set_etag(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().etag = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `fork`.\nSet to 'true' to fork an existing repository."]
    pub fn set_fork(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().fork = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `gitignore_template`.\nUse the name of the template without the extension. For example, 'Haskell'."]
    pub fn set_gitignore_template(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().gitignore_template = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `has_discussions`.\nSet to 'true' to enable GitHub Discussions on the repository. Defaults to 'false'."]
    pub fn set_has_discussions(self, v: impl std::convert::Into<terrars::PrimField<bool>>) -> Self {
        self.0.data.borrow_mut().has_discussions = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `has_downloads`.\nSet to 'true' to enable the (deprecated) downloads features on the repository."]
    pub fn set_has_downloads(self, v: impl std::convert::Into<terrars::PrimField<bool>>) -> Self {
        self.0.data.borrow_mut().has_downloads = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `has_issues`.\nSet to 'true' to enable the GitHub Issues features on the repository"]
    pub fn set_has_issues(self, v: impl std::convert::Into<terrars::PrimField<bool>>) -> Self {
        self.0.data.borrow_mut().has_issues = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `has_projects`.\nSet to 'true' to enable the GitHub Projects features on the repository. Per the GitHub documentation when in an organization that has disabled repository projects it will default to 'false' and will otherwise default to 'true'. If you specify 'true' when it has been disabled it will return an error."]
    pub fn set_has_projects(self, v: impl std::convert::Into<terrars::PrimField<bool>>) -> Self {
        self.0.data.borrow_mut().has_projects = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `has_wiki`.\nSet to 'true' to enable the GitHub Wiki features on the repository."]
    pub fn set_has_wiki(self, v: impl std::convert::Into<terrars::PrimField<bool>>) -> Self {
        self.0.data.borrow_mut().has_wiki = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `homepage_url`.\nURL of a page describing the project."]
    pub fn set_homepage_url(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().homepage_url = std::option::Option::Some(v.into());
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

    #[doc = "Set the field `ignore_vulnerability_alerts_during_read`.\n"]
    pub fn set_ignore_vulnerability_alerts_during_read(
        self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.0
            .data
            .borrow_mut()
            .ignore_vulnerability_alerts_during_read = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `is_template`.\nSet to 'true' to tell GitHub that this is a template repository."]
    pub fn set_is_template(self, v: impl std::convert::Into<terrars::PrimField<bool>>) -> Self {
        self.0.data.borrow_mut().is_template = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `license_template`.\nUse the name of the template without the extension. For example, 'mit' or 'mpl-2.0'."]
    pub fn set_license_template(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().license_template = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `merge_commit_message`.\nCan be 'PR_BODY', 'PR_TITLE', or 'BLANK' for a default merge commit message."]
    pub fn set_merge_commit_message(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().merge_commit_message = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `merge_commit_title`.\nCan be 'PR_TITLE' or 'MERGE_MESSAGE' for a default merge commit title."]
    pub fn set_merge_commit_title(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().merge_commit_title = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `private`.\n"]
    pub fn set_private(self, v: impl std::convert::Into<terrars::PrimField<bool>>) -> Self {
        self.0.data.borrow_mut().private = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `source_owner`.\nThe owner of the source repository to fork from."]
    pub fn set_source_owner(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().source_owner = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `source_repo`.\nThe name of the source repository to fork from."]
    pub fn set_source_repo(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().source_repo = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `squash_merge_commit_message`.\nCan be 'PR_BODY', 'COMMIT_MESSAGES', or 'BLANK' for a default squash merge commit message."]
    pub fn set_squash_merge_commit_message(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().squash_merge_commit_message = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `squash_merge_commit_title`.\nCan be 'PR_TITLE' or 'COMMIT_OR_PR_TITLE' for a default squash merge commit title."]
    pub fn set_squash_merge_commit_title(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().squash_merge_commit_title = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `topics`.\nThe list of topics of the repository."]
    pub fn set_topics(
        self,
        v: impl std::convert::Into<terrars::SetField<terrars::PrimField<std::string::String>>>,
    ) -> Self {
        self.0.data.borrow_mut().topics = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `visibility`.\nCan be 'public' or 'private'. If your organization is associated with an enterprise account using GitHub Enterprise Cloud or GitHub Enterprise Server 2.20+, visibility can also be 'internal'."]
    pub fn set_visibility(
        self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.0.data.borrow_mut().visibility = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `vulnerability_alerts`.\nSet to 'true' to enable security alerts for vulnerable dependencies. Enabling requires alerts to be enabled on the owner level. (Note for importing: GitHub enables the alerts on all repos by default). Note that vulnerability alerts have not been successfully tested on any GitHub Enterprise instance and may be unavailable in those settings."]
    pub fn set_vulnerability_alerts(
        self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.0.data.borrow_mut().vulnerability_alerts = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `web_commit_signoff_required`.\nRequire contributors to sign off on web-based commits."]
    pub fn set_web_commit_signoff_required(
        self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.0.data.borrow_mut().web_commit_signoff_required = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `pages`.\n"]
    pub fn set_pages(
        self,
        v: impl std::convert::Into<terrars::BlockAssignable<Repository_Pages>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.0.data.borrow_mut().pages = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.0.data.borrow_mut().dynamic.pages = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `security_and_analysis`.\n"]
    pub fn set_security_and_analysis(
        self,
        v: impl std::convert::Into<terrars::BlockAssignable<Repository_SecurityAndAnalysis>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.0.data.borrow_mut().security_and_analysis = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.0.data.borrow_mut().dynamic.security_and_analysis =
                    std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `template`.\n"]
    pub fn set_template(
        self,
        v: impl std::convert::Into<terrars::BlockAssignable<Repository_Template>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.0.data.borrow_mut().template = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.0.data.borrow_mut().dynamic.template = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Get a reference to the value of field `allow_auto_merge` after provisioning.\nSet to 'true' to allow auto-merging pull requests on the repository."]
    pub fn allow_auto_merge(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.allow_auto_merge", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `allow_forking` after provisioning.\nSet to 'true' to allow private forking on the repository; this is only relevant if the repository is owned by an organization and is private or internal."]
    pub fn allow_forking(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.allow_forking", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `allow_merge_commit` after provisioning.\nSet to 'false' to disable merge commits on the repository."]
    pub fn allow_merge_commit(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.allow_merge_commit", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `allow_rebase_merge` after provisioning.\nSet to 'false' to disable rebase merges on the repository."]
    pub fn allow_rebase_merge(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.allow_rebase_merge", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `allow_squash_merge` after provisioning.\nSet to 'false' to disable squash merges on the repository."]
    pub fn allow_squash_merge(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.allow_squash_merge", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `allow_update_branch` after provisioning.\n Set to 'true' to always suggest updating pull request branches."]
    pub fn allow_update_branch(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.allow_update_branch", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `archive_on_destroy` after provisioning.\nSet to 'true' to archive the repository instead of deleting on destroy."]
    pub fn archive_on_destroy(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.archive_on_destroy", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `archived` after provisioning.\nSpecifies if the repository should be archived. Defaults to 'false'. NOTE Currently, the API does not support unarchiving."]
    pub fn archived(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.archived", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `auto_init` after provisioning.\nSet to 'true' to produce an initial commit in the repository."]
    pub fn auto_init(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.auto_init", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `default_branch` after provisioning.\nCan only be set after initial repository creation, and only if the target branch exists"]
    pub fn default_branch(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.default_branch", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `delete_branch_on_merge` after provisioning.\nAutomatically delete head branch after a pull request is merged. Defaults to 'false'."]
    pub fn delete_branch_on_merge(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.delete_branch_on_merge", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `description` after provisioning.\nA description of the repository."]
    pub fn description(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.description", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `etag` after provisioning.\n"]
    pub fn etag(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.etag", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `fork` after provisioning.\nSet to 'true' to fork an existing repository."]
    pub fn fork(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.fork", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `full_name` after provisioning.\nA string of the form 'orgname/reponame'."]
    pub fn full_name(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.full_name", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `git_clone_url` after provisioning.\nURL that can be provided to 'git clone' to clone the repository anonymously via the git protocol."]
    pub fn git_clone_url(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.git_clone_url", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `gitignore_template` after provisioning.\nUse the name of the template without the extension. For example, 'Haskell'."]
    pub fn gitignore_template(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.gitignore_template", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `has_discussions` after provisioning.\nSet to 'true' to enable GitHub Discussions on the repository. Defaults to 'false'."]
    pub fn has_discussions(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.has_discussions", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `has_downloads` after provisioning.\nSet to 'true' to enable the (deprecated) downloads features on the repository."]
    pub fn has_downloads(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.has_downloads", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `has_issues` after provisioning.\nSet to 'true' to enable the GitHub Issues features on the repository"]
    pub fn has_issues(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.has_issues", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `has_projects` after provisioning.\nSet to 'true' to enable the GitHub Projects features on the repository. Per the GitHub documentation when in an organization that has disabled repository projects it will default to 'false' and will otherwise default to 'true'. If you specify 'true' when it has been disabled it will return an error."]
    pub fn has_projects(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.has_projects", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `has_wiki` after provisioning.\nSet to 'true' to enable the GitHub Wiki features on the repository."]
    pub fn has_wiki(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.has_wiki", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `homepage_url` after provisioning.\nURL of a page describing the project."]
    pub fn homepage_url(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.homepage_url", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `html_url` after provisioning.\nURL to the repository on the web."]
    pub fn html_url(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.html_url", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `http_clone_url` after provisioning.\nURL that can be provided to 'git clone' to clone the repository via HTTPS."]
    pub fn http_clone_url(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.http_clone_url", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `id` after provisioning.\n"]
    pub fn id(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.id", self.extract_ref()))
    }

    #[doc = "Get a reference to the value of field `ignore_vulnerability_alerts_during_read` after provisioning.\n"]
    pub fn ignore_vulnerability_alerts_during_read(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!(
                "{}.ignore_vulnerability_alerts_during_read",
                self.extract_ref()
            ),
        )
    }

    #[doc = "Get a reference to the value of field `is_template` after provisioning.\nSet to 'true' to tell GitHub that this is a template repository."]
    pub fn is_template(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.is_template", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `license_template` after provisioning.\nUse the name of the template without the extension. For example, 'mit' or 'mpl-2.0'."]
    pub fn license_template(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.license_template", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `merge_commit_message` after provisioning.\nCan be 'PR_BODY', 'PR_TITLE', or 'BLANK' for a default merge commit message."]
    pub fn merge_commit_message(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.merge_commit_message", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `merge_commit_title` after provisioning.\nCan be 'PR_TITLE' or 'MERGE_MESSAGE' for a default merge commit title."]
    pub fn merge_commit_title(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.merge_commit_title", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `name` after provisioning.\nThe name of the repository."]
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

    #[doc = "Get a reference to the value of field `primary_language` after provisioning.\n"]
    pub fn primary_language(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.primary_language", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `private` after provisioning.\n"]
    pub fn private(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.private", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `repo_id` after provisioning.\nGitHub ID for the repository."]
    pub fn repo_id(&self) -> terrars::PrimExpr<f64> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.repo_id", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `source_owner` after provisioning.\nThe owner of the source repository to fork from."]
    pub fn source_owner(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.source_owner", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `source_repo` after provisioning.\nThe name of the source repository to fork from."]
    pub fn source_repo(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.source_repo", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `squash_merge_commit_message` after provisioning.\nCan be 'PR_BODY', 'COMMIT_MESSAGES', or 'BLANK' for a default squash merge commit message."]
    pub fn squash_merge_commit_message(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.squash_merge_commit_message", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `squash_merge_commit_title` after provisioning.\nCan be 'PR_TITLE' or 'COMMIT_OR_PR_TITLE' for a default squash merge commit title."]
    pub fn squash_merge_commit_title(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.squash_merge_commit_title", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `ssh_clone_url` after provisioning.\nURL that can be provided to 'git clone' to clone the repository via SSH."]
    pub fn ssh_clone_url(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.ssh_clone_url", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `svn_url` after provisioning.\nURL that can be provided to 'svn checkout' to check out the repository via GitHub's Subversion protocol emulation."]
    pub fn svn_url(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.svn_url", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `topics` after provisioning.\nThe list of topics of the repository."]
    pub fn topics(&self) -> terrars::SetRef<terrars::PrimExpr<std::string::String>> {
        terrars::SetRef::new(
            self.shared().clone(),
            format!("{}.topics", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `visibility` after provisioning.\nCan be 'public' or 'private'. If your organization is associated with an enterprise account using GitHub Enterprise Cloud or GitHub Enterprise Server 2.20+, visibility can also be 'internal'."]
    pub fn visibility(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.visibility", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `vulnerability_alerts` after provisioning.\nSet to 'true' to enable security alerts for vulnerable dependencies. Enabling requires alerts to be enabled on the owner level. (Note for importing: GitHub enables the alerts on all repos by default). Note that vulnerability alerts have not been successfully tested on any GitHub Enterprise instance and may be unavailable in those settings."]
    pub fn vulnerability_alerts(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.vulnerability_alerts", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `web_commit_signoff_required` after provisioning.\nRequire contributors to sign off on web-based commits."]
    pub fn web_commit_signoff_required(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.web_commit_signoff_required", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `pages` after provisioning.\n"]
    pub fn pages(&self) -> terrars::ListRef<Repository_PagesRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.pages", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `security_and_analysis` after provisioning.\n"]
    pub fn security_and_analysis(&self) -> terrars::ListRef<Repository_SecurityAndAnalysisRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.security_and_analysis", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `template` after provisioning.\n"]
    pub fn template(&self) -> terrars::ListRef<Repository_TemplateRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.template", self.extract_ref()),
        )
    }
}

impl terrars::Referable for Repository {
    fn extract_ref(&self) -> std::string::String {
        format!(
            "{}.{}",
            self.0.extract_resource_type(),
            self.0.extract_tf_id()
        )
    }
}

impl terrars::Resource for Repository {}

impl terrars::ToListMappable for Repository {
    type O = terrars::ListRef<RepositoryRef>;

    fn do_map(self, base: std::string::String) -> Self::O {
        self.0.data.borrow_mut().for_each = std::option::Option::Some(format!("${{{}}}", base));
        terrars::ListRef::new(self.0.shared.clone(), self.extract_ref())
    }
}

impl terrars::Resource_ for Repository_ {
    fn extract_resource_type(&self) -> std::string::String {
        "github_repository".into()
    }

    fn extract_tf_id(&self) -> std::string::String {
        self.tf_id.clone()
    }

    fn extract_value(&self) -> serde_json::Value {
        serde_json::to_value(&self.data).unwrap()
    }
}

pub struct BuildRepository {
    pub tf_id: std::string::String,
    #[doc = "The name of the repository."]
    pub name: terrars::PrimField<std::string::String>,
}

impl BuildRepository {
    pub fn build(self, stack: &mut terrars::Stack) -> Repository {
        let out = Repository(std::rc::Rc::new(Repository_ {
            shared: stack.shared.clone(),
            tf_id: self.tf_id,
            data: std::cell::RefCell::new(RepositoryData_ {
                depends_on: core::default::Default::default(),
                provider: std::option::Option::None,
                lifecycle: core::default::Default::default(),
                for_each: std::option::Option::None,
                provisioner: core::default::Default::default(),
                allow_auto_merge: core::default::Default::default(),
                allow_forking: core::default::Default::default(),
                allow_merge_commit: core::default::Default::default(),
                allow_rebase_merge: core::default::Default::default(),
                allow_squash_merge: core::default::Default::default(),
                allow_update_branch: core::default::Default::default(),
                archive_on_destroy: core::default::Default::default(),
                archived: core::default::Default::default(),
                auto_init: core::default::Default::default(),
                default_branch: core::default::Default::default(),
                delete_branch_on_merge: core::default::Default::default(),
                description: core::default::Default::default(),
                etag: core::default::Default::default(),
                fork: core::default::Default::default(),
                gitignore_template: core::default::Default::default(),
                has_discussions: core::default::Default::default(),
                has_downloads: core::default::Default::default(),
                has_issues: core::default::Default::default(),
                has_projects: core::default::Default::default(),
                has_wiki: core::default::Default::default(),
                homepage_url: core::default::Default::default(),
                id: core::default::Default::default(),
                ignore_vulnerability_alerts_during_read: core::default::Default::default(),
                is_template: core::default::Default::default(),
                license_template: core::default::Default::default(),
                merge_commit_message: core::default::Default::default(),
                merge_commit_title: core::default::Default::default(),
                name: self.name,
                private: core::default::Default::default(),
                source_owner: core::default::Default::default(),
                source_repo: core::default::Default::default(),
                squash_merge_commit_message: core::default::Default::default(),
                squash_merge_commit_title: core::default::Default::default(),
                topics: core::default::Default::default(),
                visibility: core::default::Default::default(),
                vulnerability_alerts: core::default::Default::default(),
                web_commit_signoff_required: core::default::Default::default(),
                pages: core::default::Default::default(),
                security_and_analysis: core::default::Default::default(),
                template: core::default::Default::default(),
                dynamic: core::default::Default::default(),
            }),
        }));
        stack.add_resource(out.0.clone());
        out
    }
}

pub struct RepositoryRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for RepositoryRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Self {
        Self {
            shared: shared,
            base: base,
        }
    }
}

impl RepositoryRef {
    fn extract_ref(&self) -> std::string::String {
        self.base.clone()
    }

    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `allow_auto_merge` after provisioning.\nSet to 'true' to allow auto-merging pull requests on the repository."]
    pub fn allow_auto_merge(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.allow_auto_merge", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `allow_forking` after provisioning.\nSet to 'true' to allow private forking on the repository; this is only relevant if the repository is owned by an organization and is private or internal."]
    pub fn allow_forking(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.allow_forking", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `allow_merge_commit` after provisioning.\nSet to 'false' to disable merge commits on the repository."]
    pub fn allow_merge_commit(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.allow_merge_commit", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `allow_rebase_merge` after provisioning.\nSet to 'false' to disable rebase merges on the repository."]
    pub fn allow_rebase_merge(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.allow_rebase_merge", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `allow_squash_merge` after provisioning.\nSet to 'false' to disable squash merges on the repository."]
    pub fn allow_squash_merge(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.allow_squash_merge", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `allow_update_branch` after provisioning.\n Set to 'true' to always suggest updating pull request branches."]
    pub fn allow_update_branch(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.allow_update_branch", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `archive_on_destroy` after provisioning.\nSet to 'true' to archive the repository instead of deleting on destroy."]
    pub fn archive_on_destroy(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.archive_on_destroy", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `archived` after provisioning.\nSpecifies if the repository should be archived. Defaults to 'false'. NOTE Currently, the API does not support unarchiving."]
    pub fn archived(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.archived", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `auto_init` after provisioning.\nSet to 'true' to produce an initial commit in the repository."]
    pub fn auto_init(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.auto_init", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `default_branch` after provisioning.\nCan only be set after initial repository creation, and only if the target branch exists"]
    pub fn default_branch(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.default_branch", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `delete_branch_on_merge` after provisioning.\nAutomatically delete head branch after a pull request is merged. Defaults to 'false'."]
    pub fn delete_branch_on_merge(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.delete_branch_on_merge", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `description` after provisioning.\nA description of the repository."]
    pub fn description(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.description", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `etag` after provisioning.\n"]
    pub fn etag(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.etag", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `fork` after provisioning.\nSet to 'true' to fork an existing repository."]
    pub fn fork(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.fork", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `full_name` after provisioning.\nA string of the form 'orgname/reponame'."]
    pub fn full_name(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.full_name", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `git_clone_url` after provisioning.\nURL that can be provided to 'git clone' to clone the repository anonymously via the git protocol."]
    pub fn git_clone_url(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.git_clone_url", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `gitignore_template` after provisioning.\nUse the name of the template without the extension. For example, 'Haskell'."]
    pub fn gitignore_template(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.gitignore_template", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `has_discussions` after provisioning.\nSet to 'true' to enable GitHub Discussions on the repository. Defaults to 'false'."]
    pub fn has_discussions(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.has_discussions", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `has_downloads` after provisioning.\nSet to 'true' to enable the (deprecated) downloads features on the repository."]
    pub fn has_downloads(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.has_downloads", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `has_issues` after provisioning.\nSet to 'true' to enable the GitHub Issues features on the repository"]
    pub fn has_issues(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.has_issues", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `has_projects` after provisioning.\nSet to 'true' to enable the GitHub Projects features on the repository. Per the GitHub documentation when in an organization that has disabled repository projects it will default to 'false' and will otherwise default to 'true'. If you specify 'true' when it has been disabled it will return an error."]
    pub fn has_projects(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.has_projects", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `has_wiki` after provisioning.\nSet to 'true' to enable the GitHub Wiki features on the repository."]
    pub fn has_wiki(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.has_wiki", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `homepage_url` after provisioning.\nURL of a page describing the project."]
    pub fn homepage_url(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.homepage_url", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `html_url` after provisioning.\nURL to the repository on the web."]
    pub fn html_url(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.html_url", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `http_clone_url` after provisioning.\nURL that can be provided to 'git clone' to clone the repository via HTTPS."]
    pub fn http_clone_url(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.http_clone_url", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `id` after provisioning.\n"]
    pub fn id(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.id", self.extract_ref()))
    }

    #[doc = "Get a reference to the value of field `ignore_vulnerability_alerts_during_read` after provisioning.\n"]
    pub fn ignore_vulnerability_alerts_during_read(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!(
                "{}.ignore_vulnerability_alerts_during_read",
                self.extract_ref()
            ),
        )
    }

    #[doc = "Get a reference to the value of field `is_template` after provisioning.\nSet to 'true' to tell GitHub that this is a template repository."]
    pub fn is_template(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.is_template", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `license_template` after provisioning.\nUse the name of the template without the extension. For example, 'mit' or 'mpl-2.0'."]
    pub fn license_template(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.license_template", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `merge_commit_message` after provisioning.\nCan be 'PR_BODY', 'PR_TITLE', or 'BLANK' for a default merge commit message."]
    pub fn merge_commit_message(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.merge_commit_message", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `merge_commit_title` after provisioning.\nCan be 'PR_TITLE' or 'MERGE_MESSAGE' for a default merge commit title."]
    pub fn merge_commit_title(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.merge_commit_title", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `name` after provisioning.\nThe name of the repository."]
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

    #[doc = "Get a reference to the value of field `primary_language` after provisioning.\n"]
    pub fn primary_language(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.primary_language", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `private` after provisioning.\n"]
    pub fn private(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.private", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `repo_id` after provisioning.\nGitHub ID for the repository."]
    pub fn repo_id(&self) -> terrars::PrimExpr<f64> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.repo_id", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `source_owner` after provisioning.\nThe owner of the source repository to fork from."]
    pub fn source_owner(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.source_owner", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `source_repo` after provisioning.\nThe name of the source repository to fork from."]
    pub fn source_repo(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.source_repo", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `squash_merge_commit_message` after provisioning.\nCan be 'PR_BODY', 'COMMIT_MESSAGES', or 'BLANK' for a default squash merge commit message."]
    pub fn squash_merge_commit_message(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.squash_merge_commit_message", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `squash_merge_commit_title` after provisioning.\nCan be 'PR_TITLE' or 'COMMIT_OR_PR_TITLE' for a default squash merge commit title."]
    pub fn squash_merge_commit_title(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.squash_merge_commit_title", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `ssh_clone_url` after provisioning.\nURL that can be provided to 'git clone' to clone the repository via SSH."]
    pub fn ssh_clone_url(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.ssh_clone_url", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `svn_url` after provisioning.\nURL that can be provided to 'svn checkout' to check out the repository via GitHub's Subversion protocol emulation."]
    pub fn svn_url(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.svn_url", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `topics` after provisioning.\nThe list of topics of the repository."]
    pub fn topics(&self) -> terrars::SetRef<terrars::PrimExpr<std::string::String>> {
        terrars::SetRef::new(
            self.shared().clone(),
            format!("{}.topics", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `visibility` after provisioning.\nCan be 'public' or 'private'. If your organization is associated with an enterprise account using GitHub Enterprise Cloud or GitHub Enterprise Server 2.20+, visibility can also be 'internal'."]
    pub fn visibility(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.visibility", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `vulnerability_alerts` after provisioning.\nSet to 'true' to enable security alerts for vulnerable dependencies. Enabling requires alerts to be enabled on the owner level. (Note for importing: GitHub enables the alerts on all repos by default). Note that vulnerability alerts have not been successfully tested on any GitHub Enterprise instance and may be unavailable in those settings."]
    pub fn vulnerability_alerts(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.vulnerability_alerts", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `web_commit_signoff_required` after provisioning.\nRequire contributors to sign off on web-based commits."]
    pub fn web_commit_signoff_required(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.web_commit_signoff_required", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `pages` after provisioning.\n"]
    pub fn pages(&self) -> terrars::ListRef<Repository_PagesRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.pages", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `security_and_analysis` after provisioning.\n"]
    pub fn security_and_analysis(&self) -> terrars::ListRef<Repository_SecurityAndAnalysisRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.security_and_analysis", self.extract_ref()),
        )
    }

    #[doc = "Get a reference to the value of field `template` after provisioning.\n"]
    pub fn template(&self) -> terrars::ListRef<Repository_TemplateRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.template", self.extract_ref()),
        )
    }
}

#[derive(Serialize, Default)]
struct RepositoryDynamic {
    pages: std::option::Option<terrars::DynamicBlock<Repository_Pages>>,
    security_and_analysis:
        std::option::Option<terrars::DynamicBlock<Repository_SecurityAndAnalysis>>,
    template: std::option::Option<terrars::DynamicBlock<Repository_Template>>,
}

#[derive(Serialize)]
pub struct Repository_Pages_Source {
    branch: terrars::PrimField<std::string::String>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    path: std::option::Option<terrars::PrimField<std::string::String>>,
}

impl Repository_Pages_Source {
    #[doc = "Set the field `path`.\nThe repository directory from which the site publishes (Default: '/')"]
    pub fn set_path(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.path = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for Repository_Pages_Source {
    type O = terrars::BlockAssignable<Repository_Pages_Source>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildRepository_Pages_Source {
    #[doc = "The repository branch used to publish the site's source files. (i.e. 'main' or 'gh-pages')"]
    pub branch: terrars::PrimField<std::string::String>,
}

impl BuildRepository_Pages_Source {
    pub fn build(self) -> Repository_Pages_Source {
        Repository_Pages_Source {
            branch: self.branch,
            path: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Repository_Pages_SourceRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Repository_Pages_SourceRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Repository_Pages_SourceRef {
        Repository_Pages_SourceRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Repository_Pages_SourceRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `branch` after provisioning.\nThe repository branch used to publish the site's source files. (i.e. 'main' or 'gh-pages')"]
    pub fn branch(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.branch", self.base))
    }

    #[doc = "Get a reference to the value of field `path` after provisioning.\nThe repository directory from which the site publishes (Default: '/')"]
    pub fn path(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.path", self.base))
    }
}

#[derive(Serialize, Default)]
struct Repository_PagesDynamic {
    source: std::option::Option<terrars::DynamicBlock<Repository_Pages_Source>>,
}

#[derive(Serialize)]
pub struct Repository_Pages {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    build_type: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    cname: std::option::Option<terrars::PrimField<std::string::String>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    source: std::option::Option<std::vec::Vec<Repository_Pages_Source>>,
    dynamic: Repository_PagesDynamic,
}

impl Repository_Pages {
    #[doc = "Set the field `build_type`.\nThe type the page should be sourced."]
    pub fn set_build_type(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.build_type = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `cname`.\nThe custom domain for the repository. This can only be set after the repository has been created."]
    pub fn set_cname(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<std::string::String>>,
    ) -> Self {
        self.cname = std::option::Option::Some(v.into());
        self
    }

    #[doc = "Set the field `source`.\n"]
    pub fn set_source(
        mut self,
        v: impl std::convert::Into<terrars::BlockAssignable<Repository_Pages_Source>>,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.source = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.source = std::option::Option::Some(d);
            }
        }
        self
    }
}

impl terrars::ToListMappable for Repository_Pages {
    type O = terrars::BlockAssignable<Repository_Pages>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildRepository_Pages {}

impl BuildRepository_Pages {
    pub fn build(self) -> Repository_Pages {
        Repository_Pages {
            build_type: core::default::Default::default(),
            cname: core::default::Default::default(),
            source: core::default::Default::default(),
            dynamic: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Repository_PagesRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Repository_PagesRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Repository_PagesRef {
        Repository_PagesRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Repository_PagesRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `build_type` after provisioning.\nThe type the page should be sourced."]
    pub fn build_type(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.build_type", self.base))
    }

    #[doc = "Get a reference to the value of field `cname` after provisioning.\nThe custom domain for the repository. This can only be set after the repository has been created."]
    pub fn cname(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.cname", self.base))
    }

    #[doc = "Get a reference to the value of field `custom_404` after provisioning.\nWhether the rendered GitHub Pages site has a custom 404 page"]
    pub fn custom_404(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.custom_404", self.base))
    }

    #[doc = "Get a reference to the value of field `html_url` after provisioning.\nURL to the repository on the web."]
    pub fn html_url(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.html_url", self.base))
    }

    #[doc = "Get a reference to the value of field `status` after provisioning.\nThe GitHub Pages site's build status e.g. building or built."]
    pub fn status(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.status", self.base))
    }

    #[doc = "Get a reference to the value of field `url` after provisioning.\n"]
    pub fn url(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.url", self.base))
    }

    #[doc = "Get a reference to the value of field `source` after provisioning.\n"]
    pub fn source(&self) -> terrars::ListRef<Repository_Pages_SourceRef> {
        terrars::ListRef::new(self.shared().clone(), format!("{}.source", self.base))
    }
}

#[derive(Serialize)]
pub struct Repository_SecurityAndAnalysis_AdvancedSecurity {
    status: terrars::PrimField<std::string::String>,
}

impl Repository_SecurityAndAnalysis_AdvancedSecurity {}

impl terrars::ToListMappable for Repository_SecurityAndAnalysis_AdvancedSecurity {
    type O = terrars::BlockAssignable<Repository_SecurityAndAnalysis_AdvancedSecurity>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildRepository_SecurityAndAnalysis_AdvancedSecurity {
    #[doc = "Set to 'enabled' to enable advanced security features on the repository. Can be 'enabled' or 'disabled', This value being present when split licensing is enabled will error out."]
    pub status: terrars::PrimField<std::string::String>,
}

impl BuildRepository_SecurityAndAnalysis_AdvancedSecurity {
    pub fn build(self) -> Repository_SecurityAndAnalysis_AdvancedSecurity {
        Repository_SecurityAndAnalysis_AdvancedSecurity {
            status: self.status,
        }
    }
}

#[allow(dead_code)]
pub struct Repository_SecurityAndAnalysis_AdvancedSecurityRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Repository_SecurityAndAnalysis_AdvancedSecurityRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> Repository_SecurityAndAnalysis_AdvancedSecurityRef {
        Repository_SecurityAndAnalysis_AdvancedSecurityRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Repository_SecurityAndAnalysis_AdvancedSecurityRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `status` after provisioning.\nSet to 'enabled' to enable advanced security features on the repository. Can be 'enabled' or 'disabled', This value being present when split licensing is enabled will error out."]
    pub fn status(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.status", self.base))
    }
}

#[derive(Serialize)]
pub struct Repository_SecurityAndAnalysis_CodeSecurity {
    status: terrars::PrimField<std::string::String>,
}

impl Repository_SecurityAndAnalysis_CodeSecurity {}

impl terrars::ToListMappable for Repository_SecurityAndAnalysis_CodeSecurity {
    type O = terrars::BlockAssignable<Repository_SecurityAndAnalysis_CodeSecurity>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildRepository_SecurityAndAnalysis_CodeSecurity {
    #[doc = "Set to 'enabled' to enable code security on the repository. Can be 'enabled' or 'disabled'. If set to 'enabled', the repository's visibility must be 'public', 'security_and_analysis[0].advanced_security[0].status' must also be set to 'enabled', or your Organization must have split licensing for Advanced security."]
    pub status: terrars::PrimField<std::string::String>,
}

impl BuildRepository_SecurityAndAnalysis_CodeSecurity {
    pub fn build(self) -> Repository_SecurityAndAnalysis_CodeSecurity {
        Repository_SecurityAndAnalysis_CodeSecurity {
            status: self.status,
        }
    }
}

#[allow(dead_code)]
pub struct Repository_SecurityAndAnalysis_CodeSecurityRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Repository_SecurityAndAnalysis_CodeSecurityRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> Repository_SecurityAndAnalysis_CodeSecurityRef {
        Repository_SecurityAndAnalysis_CodeSecurityRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Repository_SecurityAndAnalysis_CodeSecurityRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `status` after provisioning.\nSet to 'enabled' to enable code security on the repository. Can be 'enabled' or 'disabled'. If set to 'enabled', the repository's visibility must be 'public', 'security_and_analysis[0].advanced_security[0].status' must also be set to 'enabled', or your Organization must have split licensing for Advanced security."]
    pub fn status(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.status", self.base))
    }
}

#[derive(Serialize)]
pub struct Repository_SecurityAndAnalysis_SecretScanning {
    status: terrars::PrimField<std::string::String>,
}

impl Repository_SecurityAndAnalysis_SecretScanning {}

impl terrars::ToListMappable for Repository_SecurityAndAnalysis_SecretScanning {
    type O = terrars::BlockAssignable<Repository_SecurityAndAnalysis_SecretScanning>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildRepository_SecurityAndAnalysis_SecretScanning {
    #[doc = "Set to 'enabled' to enable secret scanning on the repository. Can be 'enabled' or 'disabled'. If set to 'enabled', the repository's visibility must be 'public', 'security_and_analysis[0].advanced_security[0].status' must also be set to 'enabled', or your Organization must have split licensing for Advanced security."]
    pub status: terrars::PrimField<std::string::String>,
}

impl BuildRepository_SecurityAndAnalysis_SecretScanning {
    pub fn build(self) -> Repository_SecurityAndAnalysis_SecretScanning {
        Repository_SecurityAndAnalysis_SecretScanning {
            status: self.status,
        }
    }
}

#[allow(dead_code)]
pub struct Repository_SecurityAndAnalysis_SecretScanningRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Repository_SecurityAndAnalysis_SecretScanningRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> Repository_SecurityAndAnalysis_SecretScanningRef {
        Repository_SecurityAndAnalysis_SecretScanningRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Repository_SecurityAndAnalysis_SecretScanningRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `status` after provisioning.\nSet to 'enabled' to enable secret scanning on the repository. Can be 'enabled' or 'disabled'. If set to 'enabled', the repository's visibility must be 'public', 'security_and_analysis[0].advanced_security[0].status' must also be set to 'enabled', or your Organization must have split licensing for Advanced security."]
    pub fn status(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.status", self.base))
    }
}

#[derive(Serialize)]
pub struct Repository_SecurityAndAnalysis_SecretScanningAiDetection {
    status: terrars::PrimField<std::string::String>,
}

impl Repository_SecurityAndAnalysis_SecretScanningAiDetection {}

impl terrars::ToListMappable for Repository_SecurityAndAnalysis_SecretScanningAiDetection {
    type O = terrars::BlockAssignable<Repository_SecurityAndAnalysis_SecretScanningAiDetection>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildRepository_SecurityAndAnalysis_SecretScanningAiDetection {
    #[doc = "Set to 'enabled' to enable secret scanning AI detection on the repository. Can be 'enabled' or 'disabled'. If set to 'enabled', the repository's visibility must be 'public', 'security_and_analysis[0].advanced_security[0].status' must also be set to 'enabled', or your Organization must have split licensing for Advanced security."]
    pub status: terrars::PrimField<std::string::String>,
}

impl BuildRepository_SecurityAndAnalysis_SecretScanningAiDetection {
    pub fn build(self) -> Repository_SecurityAndAnalysis_SecretScanningAiDetection {
        Repository_SecurityAndAnalysis_SecretScanningAiDetection {
            status: self.status,
        }
    }
}

#[allow(dead_code)]
pub struct Repository_SecurityAndAnalysis_SecretScanningAiDetectionRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Repository_SecurityAndAnalysis_SecretScanningAiDetectionRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> Repository_SecurityAndAnalysis_SecretScanningAiDetectionRef {
        Repository_SecurityAndAnalysis_SecretScanningAiDetectionRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Repository_SecurityAndAnalysis_SecretScanningAiDetectionRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `status` after provisioning.\nSet to 'enabled' to enable secret scanning AI detection on the repository. Can be 'enabled' or 'disabled'. If set to 'enabled', the repository's visibility must be 'public', 'security_and_analysis[0].advanced_security[0].status' must also be set to 'enabled', or your Organization must have split licensing for Advanced security."]
    pub fn status(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.status", self.base))
    }
}

#[derive(Serialize)]
pub struct Repository_SecurityAndAnalysis_SecretScanningNonProviderPatterns {
    status: terrars::PrimField<std::string::String>,
}

impl Repository_SecurityAndAnalysis_SecretScanningNonProviderPatterns {}

impl terrars::ToListMappable for Repository_SecurityAndAnalysis_SecretScanningNonProviderPatterns {
    type O =
        terrars::BlockAssignable<Repository_SecurityAndAnalysis_SecretScanningNonProviderPatterns>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildRepository_SecurityAndAnalysis_SecretScanningNonProviderPatterns {
    #[doc = "Set to 'enabled' to enable secret scanning non-provider patterns on the repository. Can be 'enabled' or 'disabled'. If set to 'enabled', the repository's visibility must be 'public', 'security_and_analysis[0].advanced_security[0].status' must also be set to 'enabled', or your Organization must have split licensing for Advanced security."]
    pub status: terrars::PrimField<std::string::String>,
}

impl BuildRepository_SecurityAndAnalysis_SecretScanningNonProviderPatterns {
    pub fn build(self) -> Repository_SecurityAndAnalysis_SecretScanningNonProviderPatterns {
        Repository_SecurityAndAnalysis_SecretScanningNonProviderPatterns {
            status: self.status,
        }
    }
}

#[allow(dead_code)]
pub struct Repository_SecurityAndAnalysis_SecretScanningNonProviderPatternsRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Repository_SecurityAndAnalysis_SecretScanningNonProviderPatternsRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> Repository_SecurityAndAnalysis_SecretScanningNonProviderPatternsRef {
        Repository_SecurityAndAnalysis_SecretScanningNonProviderPatternsRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Repository_SecurityAndAnalysis_SecretScanningNonProviderPatternsRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `status` after provisioning.\nSet to 'enabled' to enable secret scanning non-provider patterns on the repository. Can be 'enabled' or 'disabled'. If set to 'enabled', the repository's visibility must be 'public', 'security_and_analysis[0].advanced_security[0].status' must also be set to 'enabled', or your Organization must have split licensing for Advanced security."]
    pub fn status(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.status", self.base))
    }
}

#[derive(Serialize)]
pub struct Repository_SecurityAndAnalysis_SecretScanningPushProtection {
    status: terrars::PrimField<std::string::String>,
}

impl Repository_SecurityAndAnalysis_SecretScanningPushProtection {}

impl terrars::ToListMappable for Repository_SecurityAndAnalysis_SecretScanningPushProtection {
    type O = terrars::BlockAssignable<Repository_SecurityAndAnalysis_SecretScanningPushProtection>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildRepository_SecurityAndAnalysis_SecretScanningPushProtection {
    #[doc = "Set to 'enabled' to enable secret scanning push protection on the repository. Can be 'enabled' or 'disabled'. If set to 'enabled', the repository's visibility must be 'public', 'security_and_analysis[0].advanced_security[0].status' must also be set to 'enabled', or your Organization must have split licensing for Advanced security."]
    pub status: terrars::PrimField<std::string::String>,
}

impl BuildRepository_SecurityAndAnalysis_SecretScanningPushProtection {
    pub fn build(self) -> Repository_SecurityAndAnalysis_SecretScanningPushProtection {
        Repository_SecurityAndAnalysis_SecretScanningPushProtection {
            status: self.status,
        }
    }
}

#[allow(dead_code)]
pub struct Repository_SecurityAndAnalysis_SecretScanningPushProtectionRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Repository_SecurityAndAnalysis_SecretScanningPushProtectionRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> Repository_SecurityAndAnalysis_SecretScanningPushProtectionRef {
        Repository_SecurityAndAnalysis_SecretScanningPushProtectionRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Repository_SecurityAndAnalysis_SecretScanningPushProtectionRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `status` after provisioning.\nSet to 'enabled' to enable secret scanning push protection on the repository. Can be 'enabled' or 'disabled'. If set to 'enabled', the repository's visibility must be 'public', 'security_and_analysis[0].advanced_security[0].status' must also be set to 'enabled', or your Organization must have split licensing for Advanced security."]
    pub fn status(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.status", self.base))
    }
}

#[derive(Serialize, Default)]
struct Repository_SecurityAndAnalysisDynamic {
    advanced_security:
        std::option::Option<terrars::DynamicBlock<Repository_SecurityAndAnalysis_AdvancedSecurity>>,
    code_security:
        std::option::Option<terrars::DynamicBlock<Repository_SecurityAndAnalysis_CodeSecurity>>,
    secret_scanning:
        std::option::Option<terrars::DynamicBlock<Repository_SecurityAndAnalysis_SecretScanning>>,
    secret_scanning_ai_detection: std::option::Option<
        terrars::DynamicBlock<Repository_SecurityAndAnalysis_SecretScanningAiDetection>,
    >,
    secret_scanning_non_provider_patterns: std::option::Option<
        terrars::DynamicBlock<Repository_SecurityAndAnalysis_SecretScanningNonProviderPatterns>,
    >,
    secret_scanning_push_protection: std::option::Option<
        terrars::DynamicBlock<Repository_SecurityAndAnalysis_SecretScanningPushProtection>,
    >,
}

#[derive(Serialize)]
pub struct Repository_SecurityAndAnalysis {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    advanced_security:
        std::option::Option<std::vec::Vec<Repository_SecurityAndAnalysis_AdvancedSecurity>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    code_security: std::option::Option<std::vec::Vec<Repository_SecurityAndAnalysis_CodeSecurity>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    secret_scanning:
        std::option::Option<std::vec::Vec<Repository_SecurityAndAnalysis_SecretScanning>>,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    secret_scanning_ai_detection: std::option::Option<
        std::vec::Vec<Repository_SecurityAndAnalysis_SecretScanningAiDetection>,
    >,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    secret_scanning_non_provider_patterns: std::option::Option<
        std::vec::Vec<Repository_SecurityAndAnalysis_SecretScanningNonProviderPatterns>,
    >,
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    secret_scanning_push_protection: std::option::Option<
        std::vec::Vec<Repository_SecurityAndAnalysis_SecretScanningPushProtection>,
    >,
    dynamic: Repository_SecurityAndAnalysisDynamic,
}

impl Repository_SecurityAndAnalysis {
    #[doc = "Set the field `advanced_security`.\n"]
    pub fn set_advanced_security(
        mut self,
        v: impl std::convert::Into<
            terrars::BlockAssignable<Repository_SecurityAndAnalysis_AdvancedSecurity>,
        >,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.advanced_security = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.advanced_security = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `code_security`.\n"]
    pub fn set_code_security(
        mut self,
        v: impl std::convert::Into<
            terrars::BlockAssignable<Repository_SecurityAndAnalysis_CodeSecurity>,
        >,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.code_security = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.code_security = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `secret_scanning`.\n"]
    pub fn set_secret_scanning(
        mut self,
        v: impl std::convert::Into<
            terrars::BlockAssignable<Repository_SecurityAndAnalysis_SecretScanning>,
        >,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.secret_scanning = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.secret_scanning = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `secret_scanning_ai_detection`.\n"]
    pub fn set_secret_scanning_ai_detection(
        mut self,
        v: impl std::convert::Into<
            terrars::BlockAssignable<Repository_SecurityAndAnalysis_SecretScanningAiDetection>,
        >,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.secret_scanning_ai_detection = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.secret_scanning_ai_detection = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `secret_scanning_non_provider_patterns`.\n"]
    pub fn set_secret_scanning_non_provider_patterns(
        mut self,
        v: impl std::convert::Into<
            terrars::BlockAssignable<
                Repository_SecurityAndAnalysis_SecretScanningNonProviderPatterns,
            >,
        >,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.secret_scanning_non_provider_patterns = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.secret_scanning_non_provider_patterns = std::option::Option::Some(d);
            }
        }
        self
    }

    #[doc = "Set the field `secret_scanning_push_protection`.\n"]
    pub fn set_secret_scanning_push_protection(
        mut self,
        v: impl std::convert::Into<
            terrars::BlockAssignable<Repository_SecurityAndAnalysis_SecretScanningPushProtection>,
        >,
    ) -> Self {
        match v.into() {
            terrars::BlockAssignable::Literal(v) => {
                self.secret_scanning_push_protection = std::option::Option::Some(v);
            }
            terrars::BlockAssignable::Dynamic(d) => {
                self.dynamic.secret_scanning_push_protection = std::option::Option::Some(d);
            }
        }
        self
    }
}

impl terrars::ToListMappable for Repository_SecurityAndAnalysis {
    type O = terrars::BlockAssignable<Repository_SecurityAndAnalysis>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildRepository_SecurityAndAnalysis {}

impl BuildRepository_SecurityAndAnalysis {
    pub fn build(self) -> Repository_SecurityAndAnalysis {
        Repository_SecurityAndAnalysis {
            advanced_security: core::default::Default::default(),
            code_security: core::default::Default::default(),
            secret_scanning: core::default::Default::default(),
            secret_scanning_ai_detection: core::default::Default::default(),
            secret_scanning_non_provider_patterns: core::default::Default::default(),
            secret_scanning_push_protection: core::default::Default::default(),
            dynamic: core::default::Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct Repository_SecurityAndAnalysisRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Repository_SecurityAndAnalysisRef {
    fn new(
        shared: terrars::StackShared,
        base: std::string::String,
    ) -> Repository_SecurityAndAnalysisRef {
        Repository_SecurityAndAnalysisRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Repository_SecurityAndAnalysisRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `advanced_security` after provisioning.\n"]
    pub fn advanced_security(
        &self,
    ) -> terrars::ListRef<Repository_SecurityAndAnalysis_AdvancedSecurityRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.advanced_security", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `code_security` after provisioning.\n"]
    pub fn code_security(
        &self,
    ) -> terrars::ListRef<Repository_SecurityAndAnalysis_CodeSecurityRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.code_security", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `secret_scanning` after provisioning.\n"]
    pub fn secret_scanning(
        &self,
    ) -> terrars::ListRef<Repository_SecurityAndAnalysis_SecretScanningRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.secret_scanning", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `secret_scanning_ai_detection` after provisioning.\n"]
    pub fn secret_scanning_ai_detection(
        &self,
    ) -> terrars::ListRef<Repository_SecurityAndAnalysis_SecretScanningAiDetectionRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.secret_scanning_ai_detection", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `secret_scanning_non_provider_patterns` after provisioning.\n"]
    pub fn secret_scanning_non_provider_patterns(
        &self,
    ) -> terrars::ListRef<Repository_SecurityAndAnalysis_SecretScanningNonProviderPatternsRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.secret_scanning_non_provider_patterns", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `secret_scanning_push_protection` after provisioning.\n"]
    pub fn secret_scanning_push_protection(
        &self,
    ) -> terrars::ListRef<Repository_SecurityAndAnalysis_SecretScanningPushProtectionRef> {
        terrars::ListRef::new(
            self.shared().clone(),
            format!("{}.secret_scanning_push_protection", self.base),
        )
    }
}

#[derive(Serialize)]
pub struct Repository_Template {
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    include_all_branches: std::option::Option<terrars::PrimField<bool>>,
    owner: terrars::PrimField<std::string::String>,
    repository: terrars::PrimField<std::string::String>,
}

impl Repository_Template {
    #[doc = "Set the field `include_all_branches`.\nWhether the new repository should include all the branches from the template repository (defaults to 'false', which includes only the default branch from the template)."]
    pub fn set_include_all_branches(
        mut self,
        v: impl std::convert::Into<terrars::PrimField<bool>>,
    ) -> Self {
        self.include_all_branches = std::option::Option::Some(v.into());
        self
    }
}

impl terrars::ToListMappable for Repository_Template {
    type O = terrars::BlockAssignable<Repository_Template>;

    fn do_map(self, base: std::string::String) -> Self::O {
        terrars::BlockAssignable::Dynamic(terrars::DynamicBlock {
            for_each: format!("${{{}}}", base),
            iterator: "each".into(),
            content: self,
        })
    }
}

pub struct BuildRepository_Template {
    #[doc = "The GitHub organization or user the template repository is owned by."]
    pub owner: terrars::PrimField<std::string::String>,
    #[doc = "The name of the template repository."]
    pub repository: terrars::PrimField<std::string::String>,
}

impl BuildRepository_Template {
    pub fn build(self) -> Repository_Template {
        Repository_Template {
            include_all_branches: core::default::Default::default(),
            owner: self.owner,
            repository: self.repository,
        }
    }
}

#[allow(dead_code)]
pub struct Repository_TemplateRef {
    shared: terrars::StackShared,
    base: std::string::String,
}

impl terrars::Ref for Repository_TemplateRef {
    fn new(shared: terrars::StackShared, base: std::string::String) -> Repository_TemplateRef {
        Repository_TemplateRef {
            shared: shared,
            base: base.to_string(),
        }
    }
}

impl Repository_TemplateRef {
    #[allow(dead_code)]
    fn shared(&self) -> &terrars::StackShared {
        &self.shared
    }

    #[doc = "Get a reference to the value of field `include_all_branches` after provisioning.\nWhether the new repository should include all the branches from the template repository (defaults to 'false', which includes only the default branch from the template)."]
    pub fn include_all_branches(&self) -> terrars::PrimExpr<bool> {
        terrars::PrimExpr::new(
            self.shared().clone(),
            format!("{}.include_all_branches", self.base),
        )
    }

    #[doc = "Get a reference to the value of field `owner` after provisioning.\nThe GitHub organization or user the template repository is owned by."]
    pub fn owner(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.owner", self.base))
    }

    #[doc = "Get a reference to the value of field `repository` after provisioning.\nThe name of the template repository."]
    pub fn repository(&self) -> terrars::PrimExpr<std::string::String> {
        terrars::PrimExpr::new(self.shared().clone(), format!("{}.repository", self.base))
    }
}
