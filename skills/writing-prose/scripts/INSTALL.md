# Installing the writing-prose CLI

Installation is optional. The CLI is a single stdlib-only module, so it runs uninstalled from anywhere:

```sh
python <skill>/scripts/writing_prose/cli.py rules <document>
python <skill>/scripts/writing_prose/cli.py edges edges.json
```

Install it when you want `writing-prose` on your PATH. All commands below run from this directory (`<skill>/scripts/`). Options in order of preference:

## 1. uv (preferred)

Local environment, via the `uv run` pattern; no explicit venv creation or install step, uv syncs the project environment on first use and keeps it current:

```sh
uv run writing-prose --help
```

Or as a user-wide tool, no environment and no project directory needed afterward:

```sh
uv tool install --editable .
writing-prose --help
```

## 2. mamba

```sh
mamba create -n writing-prose "python>=3.9" pip
mamba activate writing-prose
pip install -e .
```

## 3. conda

```sh
conda create -n writing-prose "python>=3.9" pip
conda activate writing-prose
pip install -e .
```

## 4. vanilla venv

```sh
python3 -m venv .venv
. .venv/bin/activate
pip install -e .
```

Editable installs (`-e`) are used throughout so skill updates take effect without reinstalling. There are no dependencies to resolve in any of the four paths; the environment only supplies Python 3.9 or newer.
