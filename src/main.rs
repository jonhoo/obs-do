use anyhow::Context;
use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use obws::{
    requests::{
        inputs::{InputId, Volume},
        scenes::SceneId,
    },
    Client,
};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    cmd: Command,

    /// Retry connecting to OBS for up to this many seconds (with
    /// exponential backoff) instead of failing on the first attempt.
    /// Useful right after launching OBS, when the websocket server
    /// isn't up yet.
    #[arg(long, default_value_t = 0)]
    wait_for_obs: u64,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Start/Stop streaming
    ToggleStream,
    /// Start streaming, if not already streaming.
    StartStream,
    /// Stop streaming, if currently streaming.
    StopStream,
    /// Start/Stop recording
    ToggleRecord,
    /// Pause/Unpause recording
    TogglePause,
    /// Mutes the given input.
    ToggleMute {
        #[clap(default_value = "Mic/Aux")]
        input: String,
    },
    /// Query whether the given input is muted.
    GetMute {
        #[clap(default_value = "Mic/Aux")]
        input: String,
    },
    /// Set the scene
    SetScene { scene: String },
    /// Sets the volume of the given input to specified volume.
    SetVolume {
        #[clap(default_value = "Mic/Aux")]
        input: String,

        /// Volume should be provided in dB for absolute volume or % for relative adjustments.
        ///
        /// If no unit is provided, it is interpreted as %.
        #[arg(allow_hyphen_values = true)]
        volume: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let Some(proj_dirs) = ProjectDirs::from("", "", "obs-do") else {
        anyhow::bail!("could not determine configuration file location");
    };
    let cfg = proj_dirs.config_dir().join("websocket-token");

    let exists = tokio::fs::try_exists(&cfg).await;

    let pw = match exists {
        Ok(true) => Some(
            tokio::fs::read_to_string(&cfg)
                .await
                .unwrap()
                .trim()
                .to_string(),
        ),
        Ok(false) => {
            eprintln!("Attempting to connect to OBS in password-less mode.");
            None
        }
        Err(e) => {
            anyhow::bail!(
                "Failed to read OBS WebSocket password file {}: {e:?}",
                cfg.display()
            );
        }
    };

    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(args.wait_for_obs);
    let mut backoff = std::time::Duration::from_millis(200);
    let client = loop {
        match Client::connect("localhost", 4455, pw.clone()).await {
            Ok(client) => {
                let version = client
                    .general()
                    .version()
                    .await
                    .with_context(|| "get OBS version")?;
                eprintln!(
                    "Connected to OBS: {} / {}",
                    version.obs_studio_version, version.obs_web_socket_version
                );
                break client;
            }
            Err(error) => {
                if args.wait_for_obs == 0 || tokio::time::Instant::now() >= deadline {
                    anyhow::bail!(
                        "\
Could not connect to OBS over WebSocket.

- Make sure OBS is running, and that 'Enable WebSocket server' is checked under Tools -> WebSocket Server Settings.
  If that menu item does not appear for you, your OBS has not been built with WebSocket support.\
  On Arch Linux for example, you'll want one of the AUR obs-studio packages that build WebSocket, such as obs-studio-git.

- If your server requires a password, make sure that you have it written in {}

ERROR message:
    {:?}
                    ",
                        cfg.display(),
                        error
                    )
                }
                tokio::time::sleep(backoff).await;
                backoff = (backoff * 2).min(std::time::Duration::from_secs(2));
            }
        }
    };

    match args.cmd {
        Command::ToggleStream => {
            client
                .streaming()
                .toggle()
                .await
                .context("toggle streaming")?;
        }
        Command::StartStream => {
            let status = client
                .streaming()
                .status()
                .await
                .context("get streaming status")?;
            if !status.active {
                client.streaming().start().await.context("start streaming")?;
            }
        }
        Command::StopStream => {
            let status = client
                .streaming()
                .status()
                .await
                .context("get streaming status")?;
            if status.active {
                client.streaming().stop().await.context("stop streaming")?;
            }
        }
        Command::ToggleRecord => {
            client
                .recording()
                .toggle()
                .await
                .context("toggle recording")?;
        }
        Command::TogglePause => {
            client
                .recording()
                .toggle_pause()
                .await
                .context("toggle pause")?;
        }
        Command::ToggleMute { input } => {
            client
                .inputs()
                .toggle_mute(InputId::Name(&input))
                .await
                .with_context(|| format!("toggle-mute {input}"))?;
        }
        Command::GetMute { input } => {
            let muted = client
                .inputs()
                .muted(InputId::Name(&input))
                .await
                .with_context(|| format!("get-mute {input}"))?;
            println!("{}", muted);
        }
        Command::SetScene { scene } => {
            client
                .scenes()
                .set_current_program_scene(SceneId::Name(&scene))
                .await
                .with_context(|| format!("set-scene {scene}"))?;
        }
        Command::SetVolume { input, volume } => {
            let new_volume = if let Some(db) = volume.strip_suffix("dB") {
                Volume::Db(db.parse().context("invalid dB quantity")?)
            } else {
                let volume = volume.strip_suffix('%').unwrap_or(&volume);
                Volume::Mul(volume.parse::<f32>().context("invalid % volume change")? / 100.)
            };

            client
                .inputs()
                .set_volume(InputId::Name(&input), new_volume)
                .await
                .with_context(|| format!("set-volume {input} {volume}"))?;
        }
    }

    Ok(())
}
