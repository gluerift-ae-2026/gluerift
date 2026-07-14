# Paper source

This directory contains the anonymous ACM-format manuscript for SCORED '26.

Build from the repository root:

```sh
./paper/build
```

The command compiles `paper/main.tex`, rejects unresolved references and
layout overflow warnings, and writes the submission PDF to:

```text
output/pdf/round-trips-can-lie.pdf
```

Starting from a source-only package, provision pinned tools and then reproduce:

```sh
./artifact/bootstrap-tools
./artifact/reproduce
```

The default reproduction command detects that checked owner files are absent,
generates the complete checked release at its reported external staging root,
and creates `paper/generated/*.tex` there from the canonical result owner. In a
checked-release package the same command verifies and byte-compares those
generated fragments. Build the paper from the resulting checked-release root.

The manuscript is derived only from
`ROUND-TRIPS-CAN-LIE-RESEARCH-ARTIFACT-CONTRACT-v0.3.1a.md`, whose approved
SHA-256 is
`1b0ebee64fcb482f87e1d37bece9a5ae2fc44bac7121607f31a531ea9dcf9fc7`.
