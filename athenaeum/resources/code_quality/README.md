# Code quality references

The documents, articles and patterns we use when we write and review code in
this repo. Agents read the note for a source before leaning on it, so the
same ideas come out the same way in every project.

These notes say what each source is, who owns it, where the original lives,
what we take from it and what we leave. They are summaries in our own words
with links, not copies: the originals stay current at their own address, and
the books are under copyright. A copy may be added beside a note when it
helps, under the rules in [CONTRIBUTING.md](../../CONTRIBUTING.md).

The rules themselves live in the standards, not here:
[standards/docs/code-quality.md](../../../standards/docs/code-quality.md)
(the QUAL rules of the code quality process, decision 0015) and
[standards/docs/rust.md](../../../standards/docs/rust.md). When a standard
and a note disagree, the standard wins.

## Sources

| Note | Owner | Use it for |
| --- | --- | --- |
| [Rust API Guidelines](rust-api-guidelines.md) | The Rust library team | Naming, traits, errors and docs for any public Rust API, by checklist ID |
| [Clippy](clippy.md) | The Rust project | What each lint means and why it fires |
| [Effective Rust](effective-rust.md) | David Drysdale | Types that make bad states unrepresentable, `Option`/`Result` idioms |
| [Rust Design Patterns](rust-design-patterns.md) | The rust-unofficial community | Newtype, RAII guards, builders, and the anti-patterns list |
| [Clean Code](clean-code.md) | Robert C. Martin | Names, small focused functions, comments that say why |
| [A Philosophy of Software Design](philosophy-of-software-design.md) | John Ousterhout | Deep modules, information hiding, the counterweight to tiny functions |
| [Refactoring](refactoring.md) | Martin Fowler | The code smell vocabulary for reviews, the rule of three |

Other reference notes for Rust, such as the crate list and error type
design, are in [../rust/](../rust/README.md).

## How to use them

- **Writing code:** when a choice comes up that a source covers (a name, an
  error type, a function that has grown too long), read its note and follow
  "What we take".
- **Reviewing code:** name the finding with the source's own term (a Rust
  API Guidelines ID such as `C-GETTER`, a Fowler smell such as "feature
  envy") so the fix is easy to look up.
- **Where sources disagree,** the order is: our standards, then the Rust
  sources (API Guidelines, Clippy, Effective Rust, Design Patterns), then
  the general books. Ousterhout's deep modules beat Clean Code's tiny
  functions.

## Adding a source

Add a source when it changed how we write code here, not just because it is
good. Copy the shape of an existing note:

1. A title, then a short list: **Owner**, **Original** (link), **Licence**
   (or "copyright, summary only"), **Added** (date and who found it).
2. **What we take**: the few ideas we apply, in our own words.
3. **What we leave**: the parts that don't fit Rust or this repo, and why.
4. **Where it shows up**: the standard, lint or review step that applies it.

Then add a row to the table above. A blog post or a single pattern gets the
same shape, kept to a few lines.
