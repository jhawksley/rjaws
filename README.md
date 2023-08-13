# RJaws

> John's AWS Tool (jaws): Nicer ways to interact with the AWS CLI.

Jaws provides some nicer[^1] ways to interact with AWS on the command line. Run the binary with `jaws --help` to get an overview of commands.

Each command provides its own help with the `--help` flag; for example `jaws gci --help`.

Some commands can provide more output with the global `--wide` flag. Bear in mind, using this flag will almost definitely cause the command to run more slowly.

[^1]: for a loose definition of 'nicer.'

# Command Overview

The following commands are implemented:
- `gci` - emit caller-identity information for the current AWS profile using the Security Token Service.  Can be used to check whether the current environment is valid.
- `ec2` - emit a table of EC2 information for all instances in the current region.
   * `--wide`: Also collects and tabulates extended information about each instance.
- `ssm` - log in to a given instance using SSM. The SSM module has a special requirement, see *Prerequisites* below.

For more information, run `jaws --help`.

## Global Flags
- `--region` will cause Jaws to run API calls in the specified region, rather than the default.

# Installing

## Prerequisites

* Your AWS environment must be functional prior to running `jaws`. You can run `aws sts get-caller-identity` to check this. If it doesn't work, neither will Jaws.  If your environment uses two-factor authentication (2FA), you'll need to ensure the environment has been set up for this.  [This post at AWS](https://repost.aws/knowledge-center/authenticate-mfa-cli) can help with that.
* The SSM login module (Jaws command: `ssm`) relies on the AWS SSM Session Manager Plugin, in addition to the AWS CLI.  Both must be installed.  More information can be found here: [Install the Session Manager plugin for the AWS CLI](https://docs.aws.amazon.com/systems-manager/latest/userguide/session-manager-working-with-install-plugin.html).

## Building

### Binaries

There aren't any binaries available yet.  Eventually the project will be installable with `cargo install` as a crate.  But this is still TBD.

### Building From Source

#### Development

First clone the project's Git repo into your local environment.

The project can then be built for release (without debug information) from scratch with the command:

```bash
cargo build --release
```

The target will be written to `target/release`.  You can move this wherever you need.

#### Production

To build and install the software into wherever `cargo` feels is most appropriate, execute:

```bash
cargo install --path .
```

On my workstation, this is `/Users/jhawksley/.cargo/bin`, which I've added to my `$PATH`.

# Short History

Jaws was originally born out of a requirement to simplify some of the actions in AWS that I had to repeat a lot.  The AWS CLI itself is generated out of some kind of universal template, and while it is functional, some of the commands are a bit... unwieldy.

I originally created shell aliases and functions for a few but the quantity of API calls and JSON marshaling to achieve something simple was getting out of hand.  I figured it was time to turn it into something with more structure.

**Jaws 1.0** was written in Ruby, which I still love, but which was already going out of fashion even as I started learning it.  I still think Ruby is a very nicely-executed scripting language which does OO pretty well too. 

A couple of years later I had a requirement to learn Python for another project, so I re-built Jaws in Python (that was **Jaws 2**).  This version - **Jaws 3** - is a new rewrite, purportedly to be faster and more efficient, but really just because I wanted to learn Rust.

The number of implemented commands has shrunk over time as I decided certain things weren't needed; there are still plans to reimplement some of those earlier commands (`res` - show reservation information - for instance). 

I don't intend to change languages again. But I said that the first two times too.

# Warnings

## Costs and Accuracy

Jaws makes calls to the AWS API on your behalf.  Some of these calls may be charged to you by AWS.  The author(s) are not responsible for any costs incurred using this software.

Any financial information emitted by the software is for information only, and the user should double-check its correctness before making any decisions based on it.

