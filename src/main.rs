use hyprland::{
    data::{Clients, Workspace},
    dispatch::{Dispatch, DispatchType, WindowIdentifier, WorkspaceIdentifierWithSpecial},
    shared::{HyprData, HyprDataActive},
};
use log::{debug, error, info};
use simple_logger::SimpleLogger;
use structopt::StructOpt;
use time::macros::format_description;

const SPECIAL_WORKSPACE: &str = "hyprdrop";

#[derive(StructOpt)]
#[structopt(
    name = "hyprdrop",
    about = "Generate a Hyprland window, relocate it to a dropdown, and seamlessly toggle its visibility across various workspaces."
)]
struct Cli {
    #[structopt(name = "COMMAND", help = "Command to execute")]
    cmd: String,

    #[structopt(short, long, help = "Class of command")]
    class: String,

    #[structopt(name = "ARGS", short = "a", long = "args", help = "Command arguments")]
    cmd_args: Option<String>,

    #[structopt(short, long, help = "Enable debug mode")]
    debug: bool,
}

/// Send a notification with notify-send.
fn notify(msg: &str) {
    if let Err(e) = Dispatch::call(DispatchType::Exec(&format!("notify-send {}", msg))) {
        error!("Failed to notify: {}", e);
    }
}

/// Custom parsing function for comma-delimited values
fn parse_comma_delimited(cli: &Cli) -> String {
    if let Some(args) = cli.cmd_args.clone() {
        if !args.is_empty() {
            let cmd_args = args.split(',').collect::<Vec<&str>>().join(" ");
            return format!("{} --class {} -e {}", &cli.cmd, &cli.class, &cmd_args);
        }
    }
    format!("{} --class {} ", &cli.cmd, &cli.class)
}

fn main() {
    info!("Starting Hyprdrop...");
    let cli = Cli::from_args();
    SimpleLogger::new()
        .with_level(if cli.debug {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        })
        .with_timestamp_format(format_description!(
            "[year]-[month]-[day] [hour]:[minute]:[second]"
        ))
        .init()
        .unwrap();

    let regex_class = format!("^{}$", cli.class);

    let clients = Clients::get().unwrap();
    // debug!("Clients: {:?}", clients);
    let active_workspace_id = Workspace::get_active().unwrap().id;
    // FIX: There is some freezing when there is more than one special window running. The only way
    // to fix this is closing and opening again the freezed window.
    match clients.iter().find(|client| client.class == cli.class) {
        Some(client) => {
            // Case 1: There is a client with the same class in a different workspace
            if client.workspace.id != active_workspace_id {
                // Move from special workspace or another workspace to the current one (show it)

                // Avoiding moving to the special workspace if it's already there
                if client.workspace.name != SPECIAL_WORKSPACE {
                    // NOTE: It seems weird to first move the client to the special workspace and then
                    // moving it to the active workspace but this is the only way to prevent
                    // the freezing when retrieving from another non-special workspace.
                    debug!("Moving app to {} workspace", SPECIAL_WORKSPACE);
                    if let Err(e) = Dispatch::call(DispatchType::MoveToWorkspaceSilent(
                        WorkspaceIdentifierWithSpecial::Special(Some(SPECIAL_WORKSPACE)),
                        Some(WindowIdentifier::ClassRegularExpression(&regex_class)),
                    )) {
                        error!("Failed to move app to special workspace: {}", e);
                        if cli.debug {
                            notify(&format!(
                                "Failed to move client to special workspace: {}",
                                e
                            ));
                        }
                    }
                }

                // Moving to current active workspace
                debug!("Moving app to workspace {}", active_workspace_id);
                if let Err(e) = Dispatch::call(DispatchType::MoveToWorkspaceSilent(
                    WorkspaceIdentifierWithSpecial::Id(active_workspace_id),
                    Some(WindowIdentifier::ClassRegularExpression(&regex_class)),
                )) {
                    error!("Failed to move app to workspace: {}", e);
                    if cli.debug {
                        notify(&format!("Failed to move client to workspace: {}", e));
                    }
                }
                // Focusing the retrieved window
                if let Err(e) = Dispatch::call(DispatchType::FocusWindow(
                    WindowIdentifier::ClassRegularExpression(&regex_class),
                )) {
                    error!("Failed to focus window: {}", e);
                    if cli.debug {
                        notify(&format!("Failed to focus window: {}", e));
                    }
                }

                // Bring to the front the current window. This fix the issue in case there are two
                // floating windows in the same workspace
                // NOTE: BringActiveToTop will be deprecated in the future by AlterZOrder.
                // NOTE: There is no way to determine if the focused window is already on the front.
                if let Err(e) = Dispatch::call(DispatchType::BringActiveToTop) {
                    error!("Failed to bring active window to the top: {}", e);
                    if cli.debug {
                        notify(&format!("Failed to bring active window to the top: {}", e));
                    }
                }
            } else {
                // Case 2: There is a client with the same class in the current workspace.
                // Move to the special workspace (hide it)
                debug!("Moving {} to {} workspace", cli.cmd, SPECIAL_WORKSPACE);
                if let Err(e) = Dispatch::call(DispatchType::MoveToWorkspaceSilent(
                    WorkspaceIdentifierWithSpecial::Special(Some(SPECIAL_WORKSPACE)),
                    Some(WindowIdentifier::ClassRegularExpression(&regex_class)),
                )) {
                    error!(
                        "Failed to move {}:{} to workspace: {}",
                        cli.cmd, cli.class, e
                    );
                    if cli.debug {
                        notify(&format!(
                            "Failed to move {}:{} to workspace: {}",
                            cli.cmd, cli.class, e
                        ));
                    }
                }
            }
        }
        None => {
            // Case 3: No client with the specified class found, execute command
            let final_cmd = parse_comma_delimited(&cli);
            debug!(
                "No previous matching app was found, executing command: {}",
                &final_cmd
            );
            if let Err(e) = Dispatch::call(DispatchType::Exec(&final_cmd)) {
                error!("Failed to execute command: {}", e);
                if cli.debug {
                    notify(&format!("Failed to execute command: {}", e));
                }
            }
        }
    };
}
