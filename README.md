# RJaws

> John's AWS Tool (jaws): Nicer ways to interact with the AWS CLI.

Jaws provides some nicer[^1] ways to interact with AWS on the command line. Run the binary
with `jaws --help` to get an overview of commands.

Each command provides its own help with the `--help` flag; for example `jaws gci --help`.

Some commands can provide more output with the global `--wide` flag. Bear in mind, using this flag
will almost definitely cause the command to run more slowly.

[^1]: for a loose definition of 'nicer.'

# Command Overview

The following commands are available (in no particular order):

1. `gci` - emit caller-identity information for the current AWS profile using the Security Token Service.  Can be used to check whether the current environment is valid. 
2. `ec2` - emit a table of EC2 information for all instances in the current region.
   * `--wide`: Also collects and tabulates extended information about each instance.
3. `ssm` - log in to a given instance using SSM. The SSM module has a special requirement, see *Prerequisites* below.

For more information, run `jaws --help`.

# Installing

## Prerequisites

* Your AWS environment must be functional prior to running `jaws`. You can
  run `aws sts get-caller-identity` to check this. If it doesn't work, neither will Jaws.
* The SSM login module (command: `ssm`) relies on the AWS SSM Session Manager Plugin, in addition to the AWS CLI.  Both must be installed.  More information can be found here: [Install the Session Manager plugin for the AWS CLI](https://docs.aws.amazon.com/systems-manager/latest/userguide/session-manager-working-with-install-plugin.html).

## Building and Installing

### Binaries

There aren't any binaries available yet.  Eventually the project will be installable with `cargo install` as a crate.  But this is still TBD.

### Building from source

#### Development

The project can be built for release (without debug information) from scratch with the command:

```bash
cargo build --release
```

The target will be written to `target/release`.  You can move this wherever you need.

#### Production

To build and install the software into wherever `cargo` feels is most appropriate, execute:

```bash
cargo install --path .
```

On my workstation, this is `/Users/jhawksley/.cargo/bin`, which I add to my `$PATH`.

# Potted History

Jaws was originally born out of a requirement to simplify some of the actions in AWS that I had to repeat a lot.  The AWS CLI itself is generated out of some kind of universal template, and while it is funciton, some of the commands are a bit... unwieldy.

I originally created shell aliases and functions for a few but the amount of API calls and JSON marshaling to achieve something simple was getting out of hand.  I wanted functions common to all scripts; this is possible but it's something of a hack (imho).

**Jaws 1.0** was written in Ruby, which I still love, but which was going out of fashion even as I started learning it. 

A couple of years later I had a requirement to learn Python for another project, so I re-built Jaws in Python (that was **Jaws 2**).  This version - **Jaws 3** - is a new rewrite, purportedly to be faster and more efficient, but really just because I wanted to learn Rust.  

I don't intend to change languages again. But I said that the first two times too.

# Warnings

## Costs and Accuracy

Jaws makes calls to the AWS API on your behalf.  Some of these calls may be charged to you by AWS.  The author(s) are not responsible for any costs incurred using this software.

Any financial information emitted by the software is for information only, and the user should double-check its correctness before making any decisions based on it.

