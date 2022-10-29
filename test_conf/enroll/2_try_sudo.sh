#!/usr/bin/env sh

sudo echo "this is 2_try_sudo.sh, run as root on the enrollee"
sudo echo "this is 2_try_sudo.sh, run as root on the enrollee and piped to stderr" 1>&2

sudo echo "We will now repeat any content we got on stdin, via sudo"

sudo cat
