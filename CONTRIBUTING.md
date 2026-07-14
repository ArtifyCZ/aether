# Contributing to Aether

## How to Contribute

1. **Fork** the repository and create a new branch for your change.
2. Make your changes. Keep commits focused and atomic.
3. Sign off every commit with `git commit -s` (required by the
   Developer Certificate of Origin — see *Contribution Terms* below).
4. Cryptographically sign every commit with your PGP or SSH key so that
   GitHub shows the **Verified** badge (see *Commit Messages → Signing* below).
5. Open a **Pull Request** against the `main` branch and describe what your
   change does and why.

## Branch Naming

In the **main repository**, every contributor branch shall be named:

    <owner-name>/[<issue-id>-]<short-description>

The *owner name* is the contributor's surname followed by the first letter of
their first name, all lowercase.  For example, a contributor named Richard
Tichý uses `tichyr`.

The optional *issue id* is the GitHub issue number without a hash, e.g. `42`.

Examples: `tichyr/serial-driver`, `tichyr/42-serial-driver`.

There are no branch naming requirements for personal forks.

## Code Style

- **Rust:** follow standard `rustfmt` formatting. Configure your editor to
  format on save.
- **C/C++:** follow the `clang-format` style defined in `.clangd` at the repo
  root. Format with `clang-format -i <file>` or configure your editor.
- **Bazel:** format `BUILD.bazel` / `MODULE.bazel` files with
  [`buildifier`](https://github.com/bazelbuild/buildtools):
  ```sh
  buildifier --mode=fix --lint=fix -r .
  ```

## Commit Messages

Every commit message consists of three parts: a **summary**, an optional
**body**, and a **footer**. The summary, body, and footer are each separated by
exactly one empty line.

### Signing

Every contribution commit must be cryptographically signed with a PGP or SSH
key registered in your GitHub account so that GitHub shows the **Verified**
badge on each commit.  Configure Git to sign automatically:

```sh
# PGP
git config --global commit.gpgsign true
git config --global user.signingkey <your-key-id>

# SSH (Git ≥ 2.34)
git config --global gpg.format ssh
git config --global user.signingkey ~/.ssh/id_ed25519.pub
git config --global commit.gpgsign true
```

### Summary

```
<type>: <short description>
```

The short description must use the **present simple** (imperative) form — write
`add`, not `adds`, `added`, or `adding`.

> Scopes (e.g. `feat(kernel): …`) are not used at this time because the
> allowed values have not been defined yet.

#### Change types

- **`feat`** — Any change in features: addition, update, or removal of behavior
  or a public API.
- **`fix`** — Fix a bug; include reasoning in the body when the cause is
  non-obvious.
- **`refactor`** — Code change with no behavior or API change (restructuring,
  renaming, nit fixes, …).
- **`chore`** — Anything that does not fall under the above types and does not
  affect the behavior or interface of generated code (build system, toolchain,
  dependency bumps, CI, documentation, …).

### Body (optional)

One or more paragraphs that explain *why* the change was made or provide
additional context. Paragraphs are separated by exactly one empty line.

### Footer

The footer contains one or more **attributes**, each on its own line, with no
empty lines between them. An attribute has the form:

```
Identifier: value
```

The footer must include at least a `Signed-off-by` line (required by the
Developer Certificate of Origin — see *Contribution Terms* below). Add it
automatically with `git commit -s`.

If the commit is related to one or more GitHub issues, include a `Refs`
attribute listing their IDs separated by `, `:

```
Refs: #123, #456
```

### Full example

```
feat: add PS/2 keyboard driver

The init process needs to read keyboard input to drive the interactive
shell. This commit adds a minimal PS/2 keyboard driver that translates
scan codes to ASCII characters.

The driver currently supports only scan-code set 1 (the default on x86_64).
aarch64 support will require a USB HID driver instead.

Signed-off-by: Your Name <you@example.com>
Refs: #42
```

## Contribution Terms

By contributing to this project, which includes, but is not limited to,
submitting Pull Requests, Issues, RFCs, or comments, you agree that:

1. Your contribution including, but not limited to, code, documentation, and other materials,
   and ideas, concepts, techniques, or any other intellectual property,
   shall be licensed under the project's current license(s).
   By contributing, you agree that any intellectual property in your contribution is also
   made available under the Apache 2.0 license or a similarly permissive OSI-approved license,
   ensuring the ideas can be used freely by the project and the public.

2. To ensure the project can evolve and remain compatible with various ecosystems,
   you grant the project maintainer(s) the irrevocable rights to:

    1. Re-license your contribution under any other OSI-approved
       Open Source license (such as, but not limited to, LGPL, MIT, or Apache 2.0).
       That includes but is not limited to, replacing the license,
       adding another license, or dual-licensing.

    2. Add *linking exceptions* or *additional permissions* to any project license
       to facilitate compatibility with other software (e.g., a "Static Linking Exception").

    3. The previous two points shall also permit differential licensing for specific parts of the contribution.

3. You have read the [Developer Certificate of Origin v1.1](https://developercertificate.org/) and certify you have
   the right to submit this work under its conditions. You agree to signify this by adding a `Signed-off-by:` line
   to each Git commit message (`git commit -s`).

4. You are responsible for complying with all applicable laws,
   including export control and intellectual property laws.

5. The project maintainer(s) reserves the right to reject contributions that:

    1. Do not comply with these terms; or

    2. Do not comply with the project's code of conduct; or

    3. Do not comply with the project's licensing policy; or

    4. The project maintainer(s) determine is not in the best interest of the project
       to be included in the project.

6. The project maintainer(s) reserves the right to modify these terms without prior notice.
   Such modifications shall be made in a manner that is consistent with the project's
   licensing policy. Continued participation in the project after such modifications
   shall constitute acceptance of the modified terms.
