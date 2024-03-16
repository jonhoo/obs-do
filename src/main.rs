use anyhow::Context;
use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use obws::{requests::inputs::Volume, Client};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    ToggleStream,
    ToggleRecord,
    /// Mutes the given input.
    ToggleMute {
        #[clap(default_value = "Mic/Aux")]
        input: String,
    },
    SetScene {
        scene: String,
    },
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

    let client_res = Client::connect("localhost", 4455, pw).await;
    let client = match client_res {
        Ok(client) => {
            let version = client
                .general()
                .version()
                .await
                .context("get OBS version")?;
            eprintln!(
                "Connected to OBS: {} / {}",
                version.obs_version, version.obs_web_socket_version
            );
            client
        }
        Err(error) => {
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
    };

    match args.cmd {
        Command::ToggleStream => {
            client
                .streaming()
                .toggle()
                .await
                .context("toggle streaming")?;
        }
        Command::ToggleRecord => {
            client
                .recording()
                .toggle()
                .await
                .context("toggle recording")?;
        }
        Command::ToggleMute { input } => {
            client
                .inputs()
                .toggle_mute(&input)
                .await
                .context(format!("toggle-mute {input}"))?;
        }
        Command::SetScene { scene } => {
            client
                .scenes()
                .set_current_program_scene(&scene)
                .await
                .with_context(|| format!("set-scene {scene}"))?;
        }
        Command::SetVolume { input, volume } => {
            let new_volume = if let Some(db) = volume.strip_suffix("dB") {
                Volume::Db(db.parse().context("invalid dB quantity")?)
            } else {
                let volume = volume.strip_suffix("%").unwrap_or(&volume);
                Volume::Mul(volume.parse::<f32>().context("invalid % volume change")? / 100.)
            };

            client
                .inputs()
                .set_volume(&input, new_volume)
                .await
                .context(format!("set-volume {input} {volume}"))?;
        }
    }

    Ok(())
}
