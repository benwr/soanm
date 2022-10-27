#!/bin/sh

set -xe

workdir=$(mktemp -d)
cd $workdir

# Install nix
curl -L https://nixos.org/nix/install | sh

# Source nix
if [ -e "$HOME/.nix-profile/etc/profile.d/nix.sh" ]; then
  . "$HOME/.nix-profile/etc/profile.d/nix.sh"
elif [ -e '/nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh' ]; then
  . '/nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh'
fi

# Core packages:
nix-env -iA \
  nixpkgs.xonsh \
  nixpkgs.git \
  nixpkgs.neovim \
  nixpkgs.tmux \
  nixpkgs.git-lfs \
  nixpkgs.python310 \
  nixpkgs.chezmoi \
  nixpkgs.bitwarden-cli \
  nixpkgs.asdf-vm \
  nixpkgs.awscli \
  nixpkgs.gh

# Nice-to-haves:
nix-env -iA \
  nixpkgs.ripgrep \
  nixpkgs.bat \
  nixpkgs.wget \
  nixpkgs.rclone \
  nixpkgs.pigz \
  nixpkgs.mtr \
  nixpkgs.bottom \
  nixpkgs.unixtools.watch \
  nixpkgs.entr \
  nixpkgs.parallel \
  nixpkgs.tailscale \
  nixpkgs.rsync \
  nixpkgs.openssh \
  nixpkgs.cloc \
  nixpkgs.curl \
  nixpkgs.pv \

# TODO how do I get daemons to run in a cross-platform way? especially tailscale; also maybe dropbox later.

command -v xonsh | sudo tee -a /etc/shells
chsh -s $(command -v xonsh)

# Requirements for building python
nix-env -iA nixpkgs.bzip2 \
  nixpkgs.expat \
  nixpkgs.libffi \
  nixpkgs.gdbm \
  nixpkgs.xz \
  nixpkgs.mailcap \
  nixpkgs.ncurses \
  nixpkgs.openssl \
  nixpkgs.readline \
  nixpkgs.sqlite \
  nixpkgs.tcl \
  nixpkgs.tk \
  nixpkgs.tix \
  nixpkgs.xorg.xorgproto \
  nixpkgs.zlib \
  nixpkgs.tzdata \
  nixpkgs.autoconf-archive \
  nixpkgs.bash \
  nixpkgs.pkg-config \
  nixpkgs.gnumake \
  nixpkgs.clang

asdf plugin-add rust
asdf install rust latest
asdf global rust latest

xonsh << EOF
import json
import os

send_link = input().strip()

creds = \$(bw send receive @(send_link))

creds = json.loads(creds)

home = os.path.expanduser("~")

os.makedirs(f"{home}/.ssh", exist_ok=True)
with open(f'{home}/.ssh/id_ed25519', 'w') as f:
  f.write(creds["id_ed25519"])
with open(f'{home}/.ssh/id_ed25519.pub', 'w') as f:
  f.write(creds["id_ed25519.pub"])

os.makedirs(f"{home}/.aws", exist_ok=True)
credentials_file = (
f"""[default]
aws_access_key_id = {creds["access_key_id"]}
aws_secret_access_key = {creds["secret_access_key"]}
""")
with open(f'{home}/.aws/credentials', 'w') as f:
  f.write(credentials_file)

config_file = (
"""[default]
region = us-east-1
""")

with open(f'{home}/.aws/config', 'w') as f:
  f.write(config_file)
EOF

chmod 0600 ~/.ssh/id_ed25519

chezmoi init --apply git@github.com:benwr/dotfiles

sh -c 'curl -fLo "${XDG_DATA_HOME:-$HOME/.local/share}"/nvim/site/autoload/plug.vim --create-dirs https://raw.githubusercontent.com/junegunn/vim-plug/master/plug.vim'

nvim --headless +PlugInstall +q
