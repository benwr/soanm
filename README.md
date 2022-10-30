# Shell Of A New Machine

`soanm` is a dead-simple tool for easily configuring new UNIX machines, with
almost zero prerequisites on the target machine. All it needs is `curl`.

Right now it works on MacOS and Linux on both aarch64 and x86_64, and OpenBSD
on x86_64. These are the systems I typically use and can easily build on, but
porting it to new platforms is as simple as compiling a new static Rust binary
and adding it to the release on Github. If you want support for a new platform,
add an issue and I'll be happy to get it ported.

## The Basic Idea

When you install a new operating system on some machine, whether it's a server,
a laptop, a desktop, or a VM, it can be a hassle to get to the point of feeling
comfortable on that machine: You use dozens of tools that need to be installed,
and many of these need to be configured. You need to set up Tailscale, and SSH
keys, and API keys.

Wouldn't it be great if you could just do this all at once, in a totally custom
way? Well, now you can!

You'll need two computers: One "sponsor", which is equipped with any resources
needed for provisioning the other machine (the "enrollee"). For example, if you
want your new machines to have access to GitHub, your sponsor computer should have
the ability to add ssh keys to GitHub.

To configure `soanm`, you'll also want a directory that describes the configuration
phases: It should have a subdirectory `enroll`, with programs that will be run
on the enrollee, a subdirectory `sponsor` with programs that will be run on the
sponsor, and a subdirectory `results`, where output from the enrollee will be
stored. There must be the same number of programs in the `sponsor` and `enroll`
directories, and these all must be set as executable.

Once you have created this configuration directory, the provisioning process is
begun by running `soanm sponsor [conf_directory]`. This will print a
corresponding command for you to run on the enrollee (this command will include
a secret used to securely connect the computers to one another).

The program pairs will be run in order, sorted by filename. For each pair, the
`sponsor` program is run first, and its output is sent over a [Magic
Wormhole](https://github.com/magic-wormhole/magic-wormhole.rs). That output is
then used as the input to the corresponding `enroll` program. The `enroll`
output, in turn, is sent back to the sponsor and saved in the `results`
directory, where it can be used by subsequent `sponsor` programs.

Note that on the sponsor machine, the stdin of the configuration programs is
inherited from the top-level programs, allowing for user input. And on both
sponsor and enrollee machines, stderr is printed to the corresponding terminal.

## How to use it

The sponsor machine needs a relatively recent rust version, and a directory
with a subdirectory for the `sponsor` programs, one for the `enroll` programs,
and an empty one called `results`.

```bash
cargo install soanm
soanm sponsor config_dir
```

This will print out a command, which should be run on the enrollee.

## Security considerations

Under some assumptions, it should be relatively safe to use this tool to pass
secrets around.

You obviously have to trust that the release binaries provided here are not
malicious. I hereby stake my reputation on the claim that I built them myself
from the sources contained in this repository, which (of course) does not
contain malicious code. But if you don't trust me, or the tree of dependencies
I used to build these binaries, you should verify that the release binaries
look safe. This also requires trusting GitHub to faithfully serve binaries that
have been uploaded, as well as trusting that no one has unauthorized access to
my Github account. For obvious reasons I try hard to protect my Github account,
and I promise that I will never hand control of this repository to anyone else.

You also need to trust the `magic-wormhole` protocol. We use the default
magic-wormhole rendezvous server (`ws://relay.magic-wormhole.io:4000/v1`) to
establish a secure peer-to-peer connection, over which files are transmitted.
We default to using a 128-bit passphrase (16 words) for the initial rendezvous,
though this is configurable on the sponsor command line. We have chosen to use
very long passphrases by default, since we want an extremely low probability of
being man-in-the-middled, and since the usual scenario is that you're
copy/paste-ing the passphrase from one shell to another.

## Why not just use [a different tool]?

There are lots of tools that could be used for parts of this task. Foremost
among them is [Nix](https://nixos.org). If you only use Linux, MacOS, or
*maybe* FreeBSD, Nix can do everything you need: Just write a quick script that
installs nix, grabs your nix configuration, and instantiates it into your
profile, and then you can `curl -L [myscript] | bash` to magically set
everything up. But if you use any other platforms, like (say) OpenBSD, Nix
isn't currently viable.

Other possibilities:

* You could write a script that uses SCP to send your files and configuration
  scripts to the remote host, and then runs them. This would work fine, but it
  requires that you have SSH access to the new host, which isn't typically the
  case by default on a new laptop, for example.
* You could use Ansible, or Puppet, or Terraform. But then you need to have the
  tool installed on the host, and of course you need to learn how to use these
  systems, which can be complicated. `soanm` doesn't require anything fancier
  than shell scripting, and lets you write code in any language you want as
  long as you ensure it's installed properly by an earlier stage. With most of
  these other tools, you'll likely need an additional tool for managing your
  secrets, while with `soanm` you can pipe them safely to the new host from
  the old one.
* You could use a dotfiles manager, like `chezmoi`. These typically won't
  install packages for you, or run arbitrary code, though.

You can easily combine `soanm` with any of these other methods, if that's what
you'd like to do.

There are probably a thousand ways to do this kind of task. I couldn't find one
I liked, so I wrote one that I do. Maybe you'll like it too.
