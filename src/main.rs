use hyprland::{
    data::{Client, Clients, Workspace},
    dispatch::{Dispatch, DispatchType, WindowIdentifier, WorkspaceIdentifierWithSpecial},
    shared::{Address, HyprData, HyprDataActive},
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
        short = "i",
        long = "identifier",
        help = "Command identifier. This argument is at the user's discretion when applications allow modification of the class/title of a window. Otherwise, use the one defined by the app you want to launch."
    )]
    identifier: String,

    #[structopt(
        name = "ARGS",
        short = "a",
        long = "args",
        help = "Command arguments, you must use comma-separated values."
    )]
    cmd_args: Option<String>,

    #[structopt(short = "b", long, help = "Launch in the background")]
    background: bool,

    #[structopt(short = "d", long, help = "Enable debug mode")]
    debug: bool,
}

// struct ClientWithAddress {
//     regex_match: String,
//     address: Address,
// }

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
    /// Check if the client matches the criteria
    fn check_title_or_class_or_address(&self, cli: &Cli, address: &Window) -> bool;
}

impl LocalCLient for Client {
    fn check_title_or_class_or_address(&self, cli: &Cli, address: &Window) -> bool {
        match cli.cmd.as_str() {
            "foot" => self.title == cli.identifier,
            // NOTE: gnome-terminal ignores assigning class and name variables. At tests, only
            // worked the initial title which is assigned with the `title` flag, but when the
            // terminal is opened the title is changed. Besides, hyprland-rs doesn't support the
            // WindowIdentifier by initial_title, so we have to use the address instead.
            "gnome-terminal" => {
                &self.address == address.get_address().as_ref().unwrap_or(&Address::new(""))
            }
            "konsole" => self.title.contains(&cli.identifier),
            // TODO: Add here other commands

            // Alacritty, Kitty and Wezterm all accept class name as parameter, and is assumed for
            // now to be the same for most applications
            _ => self.class == cli.identifier,
        }
    }
}

#[derive(Debug)]
enum Window<'a> {
    /// Normal window that only contains the WindowIdentifier Enum
    Normal(Option<WindowIdentifier<'a>>),
    /// Special window. It's used to adds more information to the Window data. Only
    /// gnome-terminal requires it because this app doesn't support any conventional way of
    /// identifying the window so must be added the address.
    Special((Option<WindowIdentifier<'a>>, Option<Address>)),
}

impl<'a> Window<'a> {
    /// Extract the identifier from Window Enum
    fn get_window_identifier(&self) -> Option<WindowIdentifier> {
        match self {
            Window::Normal(identifier) => identifier.as_ref().cloned(),
            Window::Special((identifier, _)) => identifier.as_ref().cloned(),
        }
    }
    /// Extract the address from Window Enum
    fn get_address(&self) -> Option<Address> {
        match self {
            Window::Special((_, address)) => address.as_ref().cloned(),
            Window::Normal(_) => None,
        }
    }
}

impl Cli {
    /// Convert the class give from CLI to a regex string
    fn to_pattern_match(&self) -> String {
        match self.cmd.as_str() {
            "konsole" => format!("{} — Konsole", self.identifier),
            _ => format!("^{}$", self.identifier),
        }
    }
    /// Get the window identifier
    fn get_window_identifier<'a>(
        &'a self,
        clients: &'a Clients,
        pattern_match: &'a str,
    ) -> Window<'a> {
        match self.cmd.as_str() {
            "alacritty" | "kitty" | "wezterm" => Window::Normal(Some(
                WindowIdentifier::ClassRegularExpression(pattern_match),
            )),
            "foot" => Window::Normal(Some(WindowIdentifier::Title(pattern_match))),
            "gnome-terminal" => self.get_window_identifier_by_address(clients, &self.identifier),
            "konsole" => Window::Normal(Some(WindowIdentifier::Title(pattern_match))),
            // It will be assumed that every other application has a class identifier. Maybe this
            // could change in the future if needed to make it more flexible
            _ => Window::Normal(Some(WindowIdentifier::ClassRegularExpression(
                pattern_match,
            ))),
        }
    }
    /// Silently move the window to the special workspace.
    fn move_to_workspace_silent(&self, window_identifier: &Window) {
        let res = Dispatch::call(DispatchType::MoveToWorkspaceSilent(
            WorkspaceIdentifierWithSpecial::Special(Some(SPECIAL_WORKSPACE)),
            window_identifier.get_window_identifier(),
        ));
        match res {
            Ok(_) => debug!(
                "Moved {}:{} to workspace: {}",
                self.cmd, &self.identifier, SPECIAL_WORKSPACE
            ),
            Err(e) => {
                handle_error(
                    &format!(
                        "Failed to move {}:{} to workspace: {}",
                        self.cmd, self.identifier, e
                    ),
                    self.debug,
                );
            }
        }
    }
    /// Move the window to the active workspace.
    fn move_to_workspace(&self, window_identifier: &Window, workspace_id: i32) {
        let res = Dispatch::call(DispatchType::MoveToWorkspace(
            WorkspaceIdentifierWithSpecial::Id(workspace_id),
            window_identifier.get_window_identifier(),
        ));
        match res {
            Ok(_) => debug!(
                "Moved {}:{} to active workspace id: {}",
                self.cmd, self.identifier, workspace_id
            ),
            Err(e) => {
                handle_error(
                    &format!(
                        "Failed to move {}:{} to active workspace id: {}",
                        self.cmd, self.identifier, e
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
                        format!("{} --class={} -e {}", self.cmd, self.identifier, &cmd_args)
                    }
                    "foot" => format!(
                        "{} --title={} --override locked-title=yes -e {}",
                        self.cmd, self.identifier, &cmd_args
                    ),
                    "wezterm" => {
                        format!(
                            "{} start --class={} -- {}",
                            self.cmd, self.identifier, &cmd_args
                        )
                    }
                    "gnome-terminal" => {
                        format!("{} --title={} -- {}", self.cmd, self.identifier, &cmd_args)
                    }
                    "konsole" => {
                        format!(
                            "{} -p tabtitle={} -e {}",
                            self.cmd, self.identifier, &cmd_args
                        )
                    }
                    // TODO: Add here other commands

                    // The default command for every other application is {cmd} + {arguments}
                    _ => format!("{} {}", self.cmd, &cmd_args),
                };
            }
        }
        // No arguments given
        match self.cmd.as_str() {
            "alacritty" | "kitty" => format!("{} --class={}", self.cmd, self.identifier),
            "foot" => format!(
                "{} --title={} --override locked-title=yes",
                self.cmd, self.identifier
            ),
            "wezterm" => format!("{} start --class={}", self.cmd, self.identifier),
            "gnome-terminal" => format!("{} --title={}", self.cmd, self.identifier),
            "konsole" => format!("{} -p tabtitle={}", self.cmd, self.identifier),
            // TODO: Add here other commands

            // The default command for every other application is {cmd}
            _ => self.cmd.clone(),
        }
    }

    /// Get the address based on the initial title of the window.
    /// NOTE: This function is only required by gnome-terminal
    fn get_window_identifier_by_address<'a>(
        &self,
        clients: &'a Clients,
        name_matching: &'a str,
    ) -> Window<'a> {
        clients
            .iter()
            .find(|client| client.initial_title == name_matching)
            .map(|client| {
                debug!(
                    "Found this address: {} associated to initial_title: {}",
                    client.address, name_matching
                );
                Window::Special((
                    Some(WindowIdentifier::Address(client.address.clone())),
                    Some(client.address.clone()),
                ))
            })
            .unwrap_or_else(|| Window::Special((None, None)))
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

    let regex_match = cli.to_pattern_match();
    let clients = Clients::get().unwrap();
    debug!("Clients: {:#?}", clients);
    let window = cli.get_window_identifier(&clients, &regex_match);
    debug!("Window identifier: {:?}", window);
    // let addresses = get_addresses_file();
    let active_workspace_id = Workspace::get_active().unwrap().id;
    match clients
        .iter()
        .find(|client| client.check_title_or_class_or_address(&cli, &window))
    {
        Some(client) => {
            // Case 1: There is a client with the same identifier in a different workspace
            // Move from special workspace or another workspace to the current one (show it)
            if client.workspace.id != active_workspace_id {
                // Avoid moving to the special workspace if it's already there
                if client.workspace.name != SPECIAL_WORKSPACE {
                    // NOTE: It seems weird to first move the client to the special workspace and then
                    // moving it to the active workspace but this is the only way to prevent
                    // the freezing when retrieving from another non-special workspace.
                    cli.move_to_workspace_silent(&window);
                }

                // Moving to current active workspace
                cli.move_to_workspace(&window, active_workspace_id);

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
                // Case 2: There is a client with the same identifier in the current workspace.
                // Move to the special workspace (hide it)
                cli.move_to_workspace_silent(&window);
            }
        }
        None => {
            // Case 3: There is no client with the same identifier.
            let parsed_args = cli.arrange_execution_cmd();
            let final_cmd = format!(
                "{}{}",
                if cli.background {
                    "[workspace special:hyprdrop silent] "
                } else {
                    ""
                },
                &parsed_args
            );
            let res = Dispatch::call(DispatchType::Exec(&final_cmd));
            match res {
                Ok(_) => {
                    debug!(
                        "No previous matching app was found, executed command: {}",
                        &final_cmd
                    );
                }
                Err(e) => {
                    handle_error(&format!("Failed to execute command: {}", e), cli.debug);
                }
            }
        }
    };
    info!("Hyprdrop finished");
}
