Forked from jonhoo after seeing that one or two basic additions could be made, then got a little carried away.
This fork adds in:
--optional targeting for the ToggleMute command
--ToggleInputFade, which actively fades between volumes over a set amount of time.
--TBD?

---

[![Crates.io](https://img.shields.io/crates/v/obs-do.svg)](https://crates.io/crates/guardian)
[![codecov](https://codecov.io/gh/jonhoo/obs-do/branch/main/graph/badge.svg?token=QOXMTH9TSA)](https://codecov.io/gh/jonhoo/guardian)
[![Dependency status](https://deps.rs/repo/github/jonhoo/obs-do/status.svg)](https://deps.rs/repo/github/jonhoo/guardian)

`obs-do` is a simple control utility that triggers OBS operations when
invoked on the command line. It's particularly handy when used in
combination with global system hotkeys that can invoke commands.

Assuming you have [Rust installed][Rust], run:

```console
$ cargo install obs-do
```

and make sure you have `~/.cargo/bin` on your `$PATH` (or execute it
directly from there).

The motivation for build it for me is that OBS [does not (yet)
support][nope] global hotkeys under Wayland. Which is unfortunate, given
that they're pretty much essential to doing streaming where you can't
switch over to the OBS window all the time. Some Wayland compositors
support [global hotkey pass-through][hyprland], which are often enough
to get by. In my case though, the combination of [this bug in
Hyprland][bug1] and [this bug in OBS][bug2] meant that I simply could
not get the global shortcuts I wanted that way.

Which brings me to `obs-do`. This super-simple CLI connects to [OBS over
WebSocket][ws] and issues commands that way, and can then simply be
invoked by whatever means you want (including Wayland compositor global
shortcuts) to trigger the desired OBS effect. For example, I'm using the
following hyprland configuration to use my numpad for stream control:

```ini
bind = SHIFT, KP_SUBTRACT, exec, obs-do toggle-stream
bind = SHIFT, KP_ADD, exec, obs-do toggle-record
bind = , KP_DELETE, exec, obs-do toggle-mute
bind = , KP_BEGIN, exec, obs-do set-scene 'Desktop (Q&A)'
bind = , KP_INSERT, exec, obs-do set-scene 'Desktop (code)'
bind = , KP_MULTIPLY, exec, obs-do set-scene 'Break'
bind = , KP_END, exec, obs-do set-scene 'Webcam'
```

Note that for this to work you _must_ have an installation of OBS that
includes WebSocket functionality. If you see Tools -> WebSocket Server
Settings in the OBS menu then you're set — just copy the "Server
Password" from the "Show Connect Info" popup into
`~/.config/obs-do/websocket-token`. Otherwise, grab a different OBS
install that does. For example, on Arch Linux at the moment, the
`obs-studio` package [does not include][arch] WebSocket support.
Instead, you need to install one of the other OBS packages from the AUR
(like `obs-studio-git`).

[nope]: https://ideas.obsproject.com/posts/2066/implement-globalshortcuts-portal
[hyprland]: https://wiki.hyprland.org/Configuring/Binds/#classic
[bug1]: https://github.com/hyprwm/Hyprland/issues/2682
[bug2]: https://github.com/obsproject/obs-studio/issues/9244
[ws]: https://github.com/obsproject/obs-websocket
[arch]: https://bugs.archlinux.org/task/76069

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
