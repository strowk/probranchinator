# probranchinator

This is a CLI tool that compares all branches in repository and displays how they can be merged to each other.

![Gif](./vhs/base.gif)

## How it works

0. It takes a remote repository URL from command line option
1. (if necessary) Clones remote repository into temporary local repository
2. Fetches all branches from remote and prunes deleted branches
3. Compares selected branches with each other (either passed via CLI or most recently updated)
4. Outputs result in terminal

Tool compares branches with each other and shows each comparison in a table.
Each entry in that table represents an attempt of merging one branch into another.
You can receive one of the following results:

- ✅✅ No changes: already up-to-date
- 🚀✅ No confilcts: fast-forward merge is possible
- 🤝✅ No conflicts: automatic merge is possible
- 🚧🔧 Found conflicts, have to resolve them manually
- ❌❌ No merge is possible (usually means your branches do not have common ancestor)
- ❌🤔 Unknown merge analysis result (this is not supposed to happen really)

Note that clone, fetch and prune operations currently require `git` CLI to be installed and available in `$PATH` due to compatibility with systems/protocols. Other operations work with cloned repository directly for efficiency. Tool creates temporary local repository in system temporary directory, because analysis of normal merge conflicts leaves working tree in a potentially 'dirty' state and we don't want to mess with user's repository, where unfinished work might be present.

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
probranchinator [OPTIONS] --remote <REMOTE> [BRANCHES]...
```

Example:

```bash
probranchinator --remote=https://github.com/strowk/probranchinator-test.git
```

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

### Output Format

By default, `probranchinator` outputs result in interactive format as a terminal UI.

You can also output result in JSON format by passing `--output=json` like this:

```console
$ probranchinator --remote=https://github.com/strowk/probranchinator-test.git --output=json master feature/1
[
  {
    "from_branch": "master",
    "to_branch": "feature/1",
    "status": "Normal"
  },
  {
    "from_branch": "feature/1",
    "to_branch": "master",
    "status": "Normal"
  }
]

```

By default output would be prettified, but you can pass `--pretty=false` to disable that.

Other available formats are:

- simple - outputs each analysis result in a single line
- table - outputs result in a table format
- markdown - outputs result as a markdown table

Examples:

```console
$ probranchinator --remote=https://github.com/strowk/probranchinator-test.git --output=simple master feature/1 feature/2 main
master -> feature/1 : 🤝✅ No conflicts: automatic merge is possible.
master -> feature/2 : 🚧🔧 Found conflicts, have to resolve them manually.
master -> main : ❌❌ No merge is possible - no merge base found.
feature/1 -> master : 🤝✅ No conflicts: automatic merge is possible.
feature/1 -> feature/2 : 🤝✅ No conflicts: automatic merge is possible.
feature/1 -> main : ❌❌ No merge is possible - no merge base found.
feature/2 -> master : 🚧🔧 Found conflicts, have to resolve them manually.
feature/2 -> feature/1 : 🤝✅ No conflicts: automatic merge is possible.
feature/2 -> main : ❌❌ No merge is possible - no merge base found.
main -> master : ❌❌ No merge is possible - no merge base found.
main -> feature/1 : ❌❌ No merge is possible - no merge base found.
main -> feature/2 : ❌❌ No merge is possible - no merge base found.

```

```console
$ probranchinator --remote=https://github.com/strowk/probranchinator-test.git --output=table master feature/1 main
+-------------+-----------+--------------------------------------------------+
| from_branch | to_branch | status                                           |
+-------------+-----------+--------------------------------------------------+
| master      | feature/1 | 🤝✅ No conflicts: automatic merge is possible.  |
+-------------+-----------+--------------------------------------------------+
| master      | main      | ❌❌ No merge is possible - no merge base found. |
+-------------+-----------+--------------------------------------------------+
| feature/1   | master    | 🤝✅ No conflicts: automatic merge is possible.  |
+-------------+-----------+--------------------------------------------------+
| feature/1   | main      | ❌❌ No merge is possible - no merge base found. |
+-------------+-----------+--------------------------------------------------+
| main        | master    | ❌❌ No merge is possible - no merge base found. |
+-------------+-----------+--------------------------------------------------+
| main        | feature/1 | ❌❌ No merge is possible - no merge base found. |
+-------------+-----------+--------------------------------------------------+

```

```console
$ probranchinator --remote=https://github.com/strowk/probranchinator-test.git --output=markdown master feature/1 main
| from_branch | to_branch | status                                           |
|-------------|-----------|--------------------------------------------------|
| master      | feature/1 | 🤝✅ No conflicts: automatic merge is possible.  |
| master      | main      | ❌❌ No merge is possible - no merge base found. |
| feature/1   | master    | 🤝✅ No conflicts: automatic merge is possible.  |
| feature/1   | main      | ❌❌ No merge is possible - no merge base found. |
| main        | master    | ❌❌ No merge is possible - no merge base found. |
| main        | feature/1 | ❌❌ No merge is possible - no merge base found. |

```