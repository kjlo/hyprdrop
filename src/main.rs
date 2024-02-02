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
fn parse_arguments(cli: &Cli) -> String {
    if let Some(args) = cli.cmd_args.clone() {
        if !args.is_empty() {
            let cmd_args = args.split(',').collect::<Vec<&str>>().join(" ");
            return format!("{} --class {} -e {}", &cli.cmd, &cli.class, &cmd_args);
        }
    }
    format!("{} --class {} ", &cli.cmd, &cli.class)
}

/// Handle errors.
fn handle_error(e: &str, debug: &bool) {
    error!("{}", e);
    if *debug {
        notify(e)
    };
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

    let regex_class = format!("^{}$", &cli.class);

    let clients = Clients::get().unwrap();
    debug!("Clients: {:?}", clients);
    let active_workspace_id = Workspace::get_active().unwrap().id;
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
                    let res = Dispatch::call(DispatchType::MoveToWorkspaceSilent(
                        WorkspaceIdentifierWithSpecial::Special(Some(SPECIAL_WORKSPACE)),
                        Some(WindowIdentifier::ClassRegularExpression(&regex_class)),
                    ));
                    match res {
                        Ok(_) => debug!(
                            "Moved {}:{} to special workspace: {}",
                            cli.cmd, cli.class, SPECIAL_WORKSPACE
                        ),
                        Err(e) => {
                            error!(
                                "Failed to move {}:{} to special workspace: {}",
                                cli.cmd, cli.class, e
                            );
                            if cli.debug {
                                notify(&format!(
                                    "Failed to move {}:{} to special workspace: {}",
                                    &cli.cmd, &cli.class, e
                                ));
                            }
                        }
                    };
                }

                // Moving to current active workspace
                let res = Dispatch::call(DispatchType::MoveToWorkspace(
                    WorkspaceIdentifierWithSpecial::Id(active_workspace_id),
                    Some(WindowIdentifier::ClassRegularExpression(&regex_class)),
                ));
                match res {
                    Ok(_) => debug!(
                        "Moved {}:{} to active workspace id: {}",
                        cli.cmd, cli.class, active_workspace_id
                    ),
                    Err(e) => {
                        handle_error(
                            &format!(
                                "Failed to move {}:{} to active workspace id: {}",
                                cli.cmd, cli.class, e
                            ),
                            &cli.debug,
                        );
                    }
                }

                // Bring to the front the current window. This fix the issue in case there are two
                // floating windows in the same workspace
                // NOTE: BringActiveToTop will be deprecated in the future by AlterZOrder.
                // NOTE: There is no way to determine if the focused window is already on the front.
                let res = Dispatch::call(DispatchType::BringActiveToTop);
                match res {
                    Ok(_) => debug!("Active window brought to the top"),
                    Err(e) => {
                        handle_error(
                            &format!("Failed to bring active window to the top: {}", e),
                            &cli.debug,
                        );
                    }
                }
            } else {
                // Case 2: There is a client with the same class in the current workspace.
                // Move to the special workspace (hide it)
                let res = Dispatch::call(DispatchType::MoveToWorkspaceSilent(
                    WorkspaceIdentifierWithSpecial::Special(Some(SPECIAL_WORKSPACE)),
                    Some(WindowIdentifier::ClassRegularExpression(&regex_class)),
                ));
                match res {
                    Ok(_) => debug!(
                        "Moved {}:{} to special workspace: {}",
                        &cli.cmd, &cli.class, SPECIAL_WORKSPACE
                    ),
                    Err(e) => {
                        handle_error(
                            &format!(
                                "Failed to move {}:{} to special workspace: {}",
                                &cli.cmd, &cli.class, e
                            ),
                            &cli.debug,
                        );
                    }
                }
            }
        }
        None => {
            // Case 3: No client with the specified class found, execute command
            let final_cmd = parse_arguments(&cli);
            debug!(
                "No previous matching app was found, executing command: {}",
                &final_cmd
            );
            let res = Dispatch::call(DispatchType::Exec(&final_cmd));
            match res {
                Ok(_) => {
                    debug!("Executed command: {}", &final_cmd);
                }

                Err(e) => {
                    handle_error(&format!("Failed to execute command: {}", e), &cli.debug);
                }
            }
        }
    };
}
