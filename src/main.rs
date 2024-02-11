use anyhow::Context;
use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use obws::Client;

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
    ToggleMute,
    SetScene { scene: String },
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
        Ok(true) => {
            Some(
                tokio::fs::read_to_string(&cfg)
                    .await
                    .unwrap()
                    .trim()
                    .to_string(),
            )
        Ok(false) => {
            eprintln!("Attempting to connect to OBS in password-less mode.");
            None
        }
        Err(e) => {
            anyhow::bail!("Failed to read OBS WebSocket password file {}: {e:?}", cfg.display());
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
        Command::ToggleMute => {
            client
                .inputs()
                .toggle_mute("Mic/Aux")
                .await
                .context("toggle-mute Mic/Aux")?;
        }
        Command::SetScene { scene } => {
            client
                .scenes()
                .set_current_program_scene(&scene)
                .await
                .with_context(|| format!("set-scene {scene}"))?;
        }
    }

    Ok(())
}
