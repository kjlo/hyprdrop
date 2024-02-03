use hyprland::{
    data::{Client, Clients, Workspace},
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
    about = "Generate an Hyprland window, relocate it to a dropdown, and seamlessly toggle its visibility across various workspaces."
)]
struct Cli {
    #[structopt(name = "COMMAND", help = "Command to execute")]
    cmd: String,

    #[structopt(
        short,
        long,
        help = "Command Class. This argument is at user's election when applications allow you to modify the class or title of a window, case contrary use the defined by the app you want to launch."
    )]
    class: String,

    #[structopt(
        name = "ARGS",
        short = "a",
        long = "args",
        help = "Command arguments, you must use comma-separated values."
    )]
    cmd_args: Option<String>,

    #[structopt(short = "b", long, help = "Launch in the background")]
    background: bool,

    #[structopt(short, long, help = "Enable debug mode")]
    debug: bool,
}

/// Send a notification with notify-send.
fn notify(msg: &str) {
    if let Err(e) = Dispatch::call(DispatchType::Exec(&format!("notify-send {}", msg))) {
        error!("Failed to notify: {}", e);
    }
}

/// Handle errors.
fn handle_error(e: &str, debug: bool) {
    error!("{}", e);
    if debug {
        notify(e)
    };
}

trait LocalCLient {
    /// Get the title or class of the client according to the given command
    fn get_title_or_class(&self, cmd: &str) -> &str;
}

impl LocalCLient for Client {
    fn get_title_or_class(&self, cmd: &str) -> &str {
        match cmd {
            "foot" => &self.title,
            _ => &self.class,
        }
    }
}

impl Cli {
    /// Convert the class give from CLI to a regex string
    fn to_regex(&self) -> String {
        format!("^{}$", self.class)
    }
    /// Get the window identifier
    fn get_window_identifier<'a>(&self, regex_class: &'a str) -> Option<WindowIdentifier<'a>> {
        match self.cmd.as_str() {
            "foot" => Some(WindowIdentifier::Title(regex_class)),
            "alacritty" | "kitty" => Some(WindowIdentifier::ClassRegularExpression(regex_class)),
            _ => None,
        }
    }
    /// Silently move the window to the special workspace.
    fn move_to_workspace_silent(&self, regex_class: &str) {
        let res = Dispatch::call(DispatchType::MoveToWorkspaceSilent(
            WorkspaceIdentifierWithSpecial::Special(Some(SPECIAL_WORKSPACE)),
            self.get_window_identifier(regex_class),
        ));
        match res {
            Ok(_) => debug!(
                "Moved {}:{} to workspace: {}",
                self.cmd, &self.class, SPECIAL_WORKSPACE
            ),
            Err(e) => {
                handle_error(
                    &format!(
                        "Failed to move {}:{} to workspace: {}",
                        self.cmd, self.class, e
                    ),
                    self.debug,
                );
            }
        }
    }
    /// Move the window to the active workspace.
    fn move_to_workspace(&self, regex_class: &str, workspace_id: i32) {
        let res = Dispatch::call(DispatchType::MoveToWorkspace(
            WorkspaceIdentifierWithSpecial::Id(workspace_id),
            self.get_window_identifier(regex_class),
        ));
        match res {
            Ok(_) => debug!(
                "Moved {}:{} to active workspace id: {}",
                self.cmd, self.class, workspace_id
            ),
            Err(e) => {
                handle_error(
                    &format!(
                        "Failed to move {}:{} to active workspace id: {}",
                        self.cmd, self.class, e
                    ),
                    self.debug,
                );
            }
        }
    }
    /// Build the execution command.
    fn arrange_execution_cmd(&self) -> String {
        // If there are arguments, add them to the command
        if let Some(args) = self.cmd_args.clone() {
            if !args.is_empty() {
                let cmd_args = args.split(',').collect::<Vec<&str>>().join(" ");
                return match self.cmd.as_str() {
                    "alacritty" | "kitty" => {
                        format!("{} --class={} -e {}", self.cmd, self.class, &cmd_args)
                    }
                    "foot" => format!(
                        "{} --title={} --override locked-title=yes -e {}",
                        self.cmd, self.class, &cmd_args
                    ),
                    // TODO: Add here other commands

                    // The default command for every other application is {cmd} + {arguments}
                    _ => format!("{} {}", self.cmd, &cmd_args),
                };
            }
        }
        // No arguments given
        match self.cmd.as_str() {
            "alacritty" | "kitty" => format!("{} --class={}", self.cmd, self.class),
            "foot" => format!(
                "{} --title={} --override locked-title=yes",
                self.cmd, self.class
            ),
            // TODO: Add here other commands

            // The default command for every other application is {cmd}
            _ => self.cmd.clone(),
        }
    }
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

    let regex_match = cli.to_regex();

    let clients = Clients::get().unwrap();
    let active_workspace_id = Workspace::get_active().unwrap().id;
    match clients
        .iter()
        .find(|client| client.get_title_or_class(&cli.cmd) == cli.class)
    {
        Some(client) => {
            // Case 1: There is a client with the same class in a different workspace
            if client.workspace.id != active_workspace_id {
                // Move from special workspace or another workspace to the current one (show it)

                // Avoiding moving to the special workspace if it's already there
                if client.workspace.name != SPECIAL_WORKSPACE {
                    // NOTE: It seems weird to first move the client to the special workspace and then
                    // moving it to the active workspace but this is the only way to prevent
                    // the freezing when retrieving from another non-special workspace.
                    cli.move_to_workspace_silent(&regex_match);
                }

                // Moving to current active workspace
                cli.move_to_workspace(&regex_match, active_workspace_id);

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
                            cli.debug,
                        );
                    }
                }
            } else {
                // Case 2: There is a client with the same class in the current workspace.
                // Move to the special workspace (hide it)
                cli.move_to_workspace_silent(&regex_match);
            }
        }
        None => {
            // Case 3: There is no client with the same class.
            let parsed_args = cli.arrange_execution_cmd();
            let final_cmd = format!(
                "{} {}",
                if cli.background {
                    "[workspace special:hyprdrop silent]"
                } else {
                    ""
                },
                &parsed_args
            );
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
                    handle_error(&format!("Failed to execute command: {}", e), cli.debug);
                }
            }
        }
    };
    info!("Hyprdrop finished");
}
