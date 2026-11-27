# Authorization (authz)

Authorization decides what an authenticated user ([authn](authn.md)) may do.

- **RBAC (role-based access control):** users get roles, roles get
  permissions. Simple to reason about and audit; it gets unwieldy when
  access depends on the data (for example "only your own records").
- **ABAC (attribute-based access control):** a policy checks attributes of
  the user, the resource, the action and the context (owner, team, time).
  More expressive than roles, harder to audit.
- **ReBAC (relationship-based):** access follows relationships, such as
  "editor of the folder that holds this file" (Google Zanzibar's model).
- **Rules that hold for all of them:** deny by default, check on the server
  for every request, give the least privilege that works, and log denials.

## Sources

- [OWASP Authorization Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Authorization_Cheat_Sheet.html)
- [NIST role-based access control](https://csrc.nist.gov/projects/role-based-access-control)
- [NIST SP 800-162, Guide to ABAC](https://csrc.nist.gov/publications/detail/sp/800-162/final)
