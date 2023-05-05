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

1. Clones remote repository into temporary local repository (if necessary)
2. Fetches all branches from remote
3. Compares all branches with each other
4. Outputs result in terminal

