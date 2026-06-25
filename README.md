# multiauth14

## What is this?

multiauth14 is a local reverse proxy that allows a Space Station 14 server to accept multiple backends without modifying RobustToolbox.

## How do I use it?

Run this application (probably as a systemd service!), then set your `auth.server` cvar to `http://localhost:3750/`

## How does it work?

When someone tries to connect to your Space Station 14 server this application will first try `auth.spacestation14.com`, then `auth.playss14.com`. You may change this behavior in `config.yaml`, which will be generated in the current working directory when you first run the application.

# Configuration

You may change the accepted set of auth servers, and order in which they're matched by editing `config.yaml`, which will be generated in the current working directory when you first run the application.

You may also forbid certain UUIDs from being checked against specific backends via the `forbidden_uuids` key. This may be useful for, e.g. restricting admin logins to a particular backend.

See [SECURITY.md](SECURITY.md) for security implications.
