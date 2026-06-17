# multiauth14

## What is this?

multiauth14 is a local reverse proxy that allows a Space Station 14 server to accept multiple backends without modifying RobustToolbox.

This tool is intended as a temporary stop-gap for the sake of players. **I hope both sides will respect any forks that choose to use it.** The auth split is bad for *everyone*, including both upstreams.

## How do I use it?

Run this application (probably as a systemd service!), then set your `auth.server` cvar to `http://localhost:3750/`

## How does it work?

When someone tries to connect to your Space Station 14 server this application will first try `auth.spacestation14.com`, then `auth.playss14.com`. You may change this behavior in `config.yaml`, which will be generated in the current working directory when you first run the application.

# Configuration

You may change the accepted set of auth servers, and order in which they're matched by editing `config.yaml`, which will be generated in the current working directory when you first run the application.

You may also forbid certain UUIDs from being checked against specific backends via the `forbidden_uuids` key. This may be useful for, e.g. restricting admin logins to a particular backend.

See [SECURITY.md](SECURITY.md) for security implications.

----------

# Soapbox follows

## Why bother?

I believe forks, and therefore, *players*, are being put in a bad situation. Many players are not in Discord to get an explanation for why their favorite servers seemingly do not exist anymore.

I have no stake in the conflict between SS14 maintainers. However, I do not believe that abruptly and disruptively forking the auth infrastructure was necessary if the only concern Wizden had was to develop without PJB's involvement.

While Wizden would eventually need alternate infrastructure to develop RobustToolbox, they clearly do not believe it was urgently required at this time. Modifying `SS14_LAUNCHER_OVERRIDE_AUTH` on the Steam launcher does not afford them control over RT builds.

Wizden could have pushed for multi-auth. They also could have pushed Wizden players, specifically, to use their alternate launcher. Either of these cases would have minimized disruption for other forks, and allowed more time for the community to come together on a common solution.

Instead, many forks have found themselves forced to choose between Wizden and PJB at the expense of friction and difficulty for their players. Negotiation by gunpoint because mommy and daddy are fighting isn't cool.

Much has been said about PJB, and reasonably so. Very little has been said about Wizden staff repeatedly implying, or saying outright, that the original SS14 infrastructure "isn't safe" with no evidence. I don't think legal threats, misleading statements, and pressure tactics are a great basis for 'fresh start' for SS14.

SS14 should not be held hostage by whomever happens to hold the keys for critical infrastructure. **Wizden's problems did not start and end with a single person.**

## Where do we go from here?

The biggest problem with decentralizing is ensuring that RobustToolbox remains safe for users: SS14's threat model assumes absolute trust for its engine. However, SS14 is certainly not the only GPU-heavy application that requires sandboxing.

A full technical discussion of sandboxing is out of scope for this document, however, in brief:

- Native platform sandbox solutions could easily wrap RT; bubblewrap/AppContainer/Seatbelt each support GPU access.
- All unsafe operations in RT could be hoisted out to a trusted shim, and RT itself could be IL verified in the same way content bundles are.
- RT could be compiled as a WASM module and run in the WebAssembly sandbox.

If SS14 supported OAuth from major providers (Google, Apple, Steam, GitHub, Discord, etc) and RT itself was sandboxed, the only real choice a fork would need to make is which hubs, if any, to broadcast on. That would preclude a situation substantially similar to this one from ever happening again.

Ban evasion could be made substantially more difficult under an OAuth scheme. Most people will have an account with history on at least one of those services. That strongly narrows the field for which 'new players' an administrator needs to watch out for.

It also carries the advantage that a SS13 'panic bunker' style defense would not need to punish new players. For instance, old Steam accounts are not an abundant resource available to raiders.