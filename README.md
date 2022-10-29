# Shell Of A New Machine

`soanm` is a tool for easily configuring new UNIX machines, with zero
prerequisites on the target machine other than running a POSIX-compatible
operating system with a working Bourne shell (and [Rust platform
support](https://doc.rust-lang.org/beta/rustc/platform-support.html)).

## The Basic Idea

When you install a new operating system on some machine, whether it's a server,
a laptop, a desktop, or a VM, it can be a hassle to get to the point of feeling
comfortable in that shell: You use dozens of tools that need to be installed,
and many of these need to be configured. You need to set up Tailscale, and SSH
keys, and API keys. 

Wouldn't it be great if you could just do this all at once? Well, now you can!

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
a secret used to securely connect the computers to one another)

The program pairs will be run in order, sorted by filename. For each pair, the
`sponsor` program is run first, and its output is sent over a [Magic
Wormhole](https://github.com/magic-wormhole/magic-wormhole.rs). That output is
then used as the input to the corresponding `enroll` program. The `enroll`
output, in turn, is sent back to the sponsor and saved in the `results`
directory, where it can be used by subsequent `sponsor` programs.

Note that on the sponsor machine, the stdin of the configuration programs is
inherited from the top-level programs, allowing for user input. And on both
sponsor and enrollee machines, stderr is printed to the corresponding terminal.

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
  long as you ensure it's installed properly first. You'll also likely need an
  additional tool for managing your secrets.
