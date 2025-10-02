#!/usr/bin/env bash

set -eo pipefail

BASE_DIR=${XDG_CONFIG_HOME:-${HOME}}
MY_CLI_DIR=${MY_CLI_DIR:-"${BASE_DIR}/.my-cli"}
MY_CLI_BIN_DIR="${MY_CLI_DIR}/bin"
CARGO_MY_CLI="${MY_CLI_BIN_DIR}/cargo-my-cli"

mkdir -p "${MY_CLI_BIN_DIR}"

main() {
  need_cmd curl

  while [[ -n $1 ]]; do
    case $1 in
    --)
      shift
      break
      ;;
    -v | --version)
      shift
      MY_CLI_VERSION=$1
      ;;    
    --arch)
      shift
      MY_CLI_ARCH=$1
      ;;
    --platform)
      shift
      MY_CLI_PLATFORM=$1
      ;;
    -h | --help)
      usage
      exit 0
      ;;
    *)
      err "unknown option: $1"
      echo
      usage
      exit 1
      ;;
    esac
    shift
  done

  uname_s=$(uname -s)
  PLATFORM=$(tolower "${MY_CLI_PLATFORM:-${uname_s}}")

  case "${PLATFORM}" in
    linux) ;;
    darwin | mac*) ;;
    *)
      err "unsupported platform ${PLATFORM}"
      exit 1
      ;;
  esac

  banner

  # Check if the user provided a version
  if [[ -z "${MY_CLI_VERSION}" ]]; then
    MY_CLI_VERSION="latest"
  else
    MY_CLI_VERSION="v${MY_CLI_VERSION}"
  fi

  # Replace with your GitHub username/repo
  MY_CLI_REPO="your-username/your-repo"
  EXT="tar.gz"
  
  step "Installing my-cli ${MY_CLI_VERSION} version..."

  uname_m=$(uname -m)
  ARCHITECTURE=$(tolower "${MY_CLI_ARCH:-${uname_m}}")
  if [ "${ARCHITECTURE}" = "x86_64" ]; then
    # Redirect stderr to /dev/null to avoid printing errors if non Rosetta.
    if [ "$(sysctl -n sysctl.proc_translated 2>/dev/null)" = "1" ]; then
      ARCHITECTURE="arm64" # Rosetta.
    else
      ARCHITECTURE="amd64" # Intel.
    fi
  elif [ "${ARCHITECTURE}" = "arm64" ] || [ "${ARCHITECTURE}" = "aarch64" ]; then
    ARCHITECTURE="arm64" # Arm.
  else
    ARCHITECTURE="amd64" # Amd.
  fi

  # Compute the URL of the release tarball
  if [ "${MY_CLI_VERSION}" = "latest" ]; then
    RELEASE_URL="https://github.com/${MY_CLI_REPO}/releases/latest/download/"
  else 
    RELEASE_URL="https://github.com/${MY_CLI_REPO}/releases/download/${MY_CLI_VERSION}/"
  fi

  BIN_ARCHIVE_URL="${RELEASE_URL}cargo_my-cli_${PLATFORM}_${ARCHITECTURE}.${EXT}"

  # Download and extract the binaries archive
  if [ "${PLATFORM}" = "linux" ] || [ "${PLATFORM}" = "darwin" ]; then
    tmp="$(mktemp -d)/cargo_my-cli.tar.gz"

    step "Downloading my-cli ${MY_CLI_VERSION} version..."
    ensure download "${BIN_ARCHIVE_URL}" "${tmp}"

    step "Installing my-cli ${MY_CLI_VERSION} version..."
    ensure tar -xzf "${tmp}" -C "${MY_CLI_BIN_DIR}"

    # Make binary executable
    chmod +x "${CARGO_MY_CLI}"

    rm -f "${tmp}"
  fi

  # Print installed version message
  MY_CLI_VERSION=$(echo "$(ensure "${CARGO_MY_CLI}" --version)" | awk '{print $2}')
  say "Installed my-cli version ${MY_CLI_VERSION}"

  # Check if the binary is already in PATH
  which_path="$(command -v cargo-my-cli || true)"
  if [ -n "$which_path" ] && [ "${which_path}" != "${CARGO_MY_CLI}" ]; then
    warn ""
    cat 1>&2 <<EOF
There are multiple binaries with the name 'cargo-my-cli' present in your 'PATH'.
This may be the result of installing 'cargo-my-cli' using another method.
You may need to run 'rm ${which_path}' or move '${MY_CLI_BIN_DIR}'
in your 'PATH' to allow the newly installed version to take precedence!

EOF
  fi

  # Determine the profile file based on the current shell
  case "${SHELL}" in
    */zsh)
      PROFILE=${ZDOTDIR:-${HOME}}/.zshenv
      PREF_SHELL="zsh"
      ;;
    */bash)
      PROFILE=${HOME}/.bashrc
      PREF_SHELL="bash"
      ;;
    */fish)
      PROFILE=${HOME}/.config/fish/config.fish
      PREF_SHELL="fish"
      ;;
    */ash)
      PROFILE=${HOME}/.profile
      PREF_SHELL="ash"
      ;;
    *)
      warn "could not detect shell, manually add ${MY_CLI_BIN_DIR} to your PATH"
      exit 1
      ;;
  esac

  # Only add to PATH if it isn't already there
  if [[ ":${PATH}:" != *":${MY_CLI_BIN_DIR}:"* ]]; then
      echo "Adding my-cli to your PATH..."
      echo >>"${PROFILE}" && echo "export PATH=\"\$PATH:${MY_CLI_BIN_DIR}\"" >>"${PROFILE}"
  fi

  step "Done! my-cli version ${MY_CLI_VERSION} has been installed."
  say "Your preferred shell '${PREF_SHELL}' was detected and my-cli has been added to your PATH."
  say "To start using my-cli, run 'source ${PROFILE}' or open a new terminal session."
  say ""
  say "You can now use: cargo my-cli [args]"
  echo
}

usage() {
  cat 1>&2 <<EOF
The installer for my-cli.

Update or revert to a specific my-cli version with ease.

USAGE:
    install-my-cli.sh <OPTIONS>

OPTIONS:
    -h, --help      Print help information
    -v, --version   Install a specific version
    --arch          Install a specific architecture (supports amd64 and arm64)
    --platform      Install a specific platform (supports linux and darwin)
EOF
}

say() {
  printf "%s\n" "$1"
}

step() {
  echo
  printf "\033[0;32m%s\033[0m\n" "$1"
}

warn() {
  printf "\033[0;33mWarning: %s\033[0m\n" "$1"
}

err() {
  printf "\033[0;31mError: %s\033[0m\n" "$1" >&2
}

tolower() {
  echo "$1" | awk '{print tolower($0)}'
}

need_cmd() {
  if ! check_cmd "$1"; then
    err "need '$1' (command not found)"
  fi
}

check_cmd() {
  command -v "$1" &>/dev/null
}

ensure() {
  if ! "$@"; then 
    err "command failed: $*"; 
    exit 1
  fi
}

# Downloads $1 into $2
download() {
  say "Downloading from: $1"
  if check_cmd curl; then
    curl -H "Accept: application/octet-stream" -L -#o "$2" "$1"
  else
    wget --header="Accept: application/octet-stream" --show-progress -qO "$2" "$1"
  fi
}

banner() {
  printf "
########################################################################################

    __  ____  __   ________    ____
   /  |/  / \\/ /  / ____/ /   /  _/
  / /|_/ /\\  /  / /   / /    / /  
 / /  / / / /  / /___/ /____/ /   
/_/  /_/ /_/   \\____/_____/___/   

My CLI Installer

########################################################################################
"
}

main "$@"