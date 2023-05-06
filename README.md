# probranchinator

This is a CLI tool that compares all branches in repository and displays how they can be merged to each other.

![Gif](./vhs/base.gif)

## Installation

Download a binary from [latest release](https://github.com/strowk/probranchinator/releases) and put it somewhere in your `$PATH`.

Tool requires `git` CLI to be installed and available in `$PATH`.

### Install with script

You can also use `install.sh` script with `bash` shell. 
It will download the latest release and put it in `/usr/local/bin` directory.

```bash
curl -s https://raw.githubusercontent.com/strowk/probranchinator/main/install.sh | bash
```

Only limited amount of platforms have prebuilt binaries at the moment: Linux (x86_64 gnu), macOS(x86_64, aarch64) and Windows (x86_64 gnu).

### Install with cargo

In case if your platform/architecture is not supported, you might need to build the tool from source.
You will need to have [Rust](https://www.rust-lang.org/tools/install) toolchain installed.

Then you can install the tool with `cargo`:

```bash
cargo install probranchinator
```

## Usage

```bash
probranchinator.exe [OPTIONS] --remote <REMOTE> [BRANCHES]...
```

Example:

```bash
probranchinator --remote=https://github.com/strowk/probranchinator
```

Change remote to something else, as probranchinator itself does not have many branches ATM.

If you want to examine your local repository, you can use `file://` protocol:

```bash
probranchinator --remote=file://$PWD
```

By default `probranchinator` will analyse 10 most recently updated branches.
You can override that by passing branches to analyse as CLI arguments:

```bash
probranchinator --remote=https://gitlab.com/git-compose/git-compose.git master test-branch-2
```

or by changing the amount of recent branches to analyse:

```bash
probranchinator --remote=https://gitlab.com/git-compose/git-compose.git --recent=2
```

To exit the program, press `q` or `Ctrl+C`.

## How it works

1. (if necessary) Clones remote repository into temporary local repository
2. Fetches all branches from remote and prunes deleted branches
3. Compares selected branches with each other (either passed via CLI or most recently updated)
4. Outputs result in terminal

Tool compares branches with each other and shows each comparison in a table.
Each entry in that table represents an attempt of merging one branch into another.
You can receive one of the following results:

- ✅✅ No changes: the branches are already up-to-date
- 🚀✅ No confilcts: fast-forward merge is possible
- 🚧🍀 Found conflicts, but can resolve them automatically
- 🚧🔧 Found conflicts, have to resolve them manually
- ❌❌ No merge is possible (usually means your branches do not have common ancestor)
- ❌🤔 Unknown merge analysis result (this is not supposed to happen really)

