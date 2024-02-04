use std::{
    borrow::Cow,
    fs::{File, OpenOptions},
    io::{self, BufRead, BufReader, Write},
};

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
const ADDRESSES_PATH_FILE: &str = "/tmp/hyprdrop";

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

struct ClientWithAddress {
    regex_match: String,
    address: Address,
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
    /// Check if the client matches the criteria
    fn check_title_or_class_or_address(&self, cli: &Cli, addresses: &[ClientWithAddress]) -> bool;
}

impl LocalCLient for Client {
    fn check_title_or_class_or_address(&self, cli: &Cli, addresses: &[ClientWithAddress]) -> bool {
        match cli.cmd.as_str() {
            "foot" => self.title == cli.class,
            // NOTE: gnome-terminal ignores assigning class and name variables. At tests, only
            // worked the initial title which is assigned with the `title` flag, but when the
            // terminal is opened the title is changed. Besides, hyprland-rs doesn't support the
            // WindowIdentifier by initial_title, so we have to use the address instead.
            "gnome-terminal" => check_addresses(&cli.class, addresses, self),
            // TODO: Add here other commands

            // now to be the same for most applications
            // Alacritty, Kitty and Wezterm all accept class name as parameter, and is assumed for
            _ => self.class == cli.class,
        }
    }
}

fn write_address_in_file(regex_match: &str, address: Address) {
    // Read the file
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(ADDRESSES_PATH_FILE)
        .ok()
        .unwrap();

    let mut lines: Vec<String> = BufReader::new(&file)
        .lines()
        .map(|line| line.unwrap())
        .collect();

    // Find the line with the specified regex_match
    if let Some(index) = lines.iter().position(|line| line.starts_with(regex_match)) {
        // Replace the second column with the new value
        let new_line = format!("{}:{}", regex_match, address);
        lines[index] = new_line;

        // Write the modified lines back to the file
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(ADDRESSES_PATH_FILE)
            .ok()
            .unwrap();

        file.write_all(lines.join("\n").as_bytes()).ok().unwrap();
        debug!(
            "Updated {}:{} in file: {}",
            regex_match, address, ADDRESSES_PATH_FILE
        );
    }
}

/// Read from file the address based on regex_match and return it if found
fn read_address_from_file(regex_match: &str, clients: &Clients) -> Option<Address> {
    File::open(ADDRESSES_PATH_FILE).ok().and_then(|file| {
        io::BufReader::new(file)
            .lines()
            .find(|line| line.as_ref().map_or(false, |l| l.starts_with(regex_match)))
            .and_then(|line| {
                let binding = line.ok()?;
                let address = binding.split(':').nth(1)?.trim();
                // Check if address is already in file list
                let found_address = clients
                    .iter()
                    .any(|client| client.address.to_string() == address);
                if found_address {
                    Some(Address::new(address))
                } else {
                    None
                }
            })
    })
}

fn check_addresses(regex_match: &str, addresses: &[ClientWithAddress], client: &Client) -> bool {
    addresses
        .iter()
        .any(|address| client.address == address.address && regex_match == address.regex_match)
}

/// Open file and append into client the content of the file
fn get_addresses_file() -> Vec<ClientWithAddress> {
    let mut client: Vec<ClientWithAddress> = Vec::new();
    if let Ok(file) = File::open(ADDRESSES_PATH_FILE) {
        io::BufReader::new(file).lines().for_each(|line| {
            if let Ok(binding) = line {
                let columns: Vec<&str> = binding.split(':').collect();
                client.push(ClientWithAddress {
                    regex_match: columns[0].to_string(),
                    address: Address::new(columns[1].to_string()),
                });
            }
        });
    }
    client
    // let mut client: Vec<ClientWithAddress> = Vec::new();
    // File::open(ADDRESSES_PATH_FILE).ok().and_then(|file| {
    //     io::BufReader::new(file).lines().map(|line| {
    //         let binding = line.ok().unwrap();
    //         let columns: Vec<&str> = binding.split(':').collect();
    //         // appends the columns to client
    //         client.push(ClientWithAddress {
    //             regex_match: columns[0].to_string(),
    //             address: Address::new(columns[1].to_string()),
    //         });
    //     })
    // });
    // client
}
// fn read_addres_from_file(regex_match: &str) -> Option<String> {
//
//     if let Ok(file) = File::open(ADDRESSES_PATH_FILE) {
//         let reader = io::BufReader::new(file);
//
//         let mut address = "".to_string();
//         reader.lines().for_each(|line| {
//             if let Ok(l) = line {
//                 let columns: Vec<&str> = l.split(':').collect();
//                 // Check if the first column matches the target string
//                 if let Some(first_column) = columns.first() {
//                     if *first_column == regex_match {
//                         // Return the value from the second column
//                         if let Some(second_column) = columns.get(1) {
//                             address = second_column.to_string();
//                         }
//                     }
//                 }
//             }
//         });
//         if !address.is_empty() {
//             return Some(address);
//         }
//     }
//     None
// }

impl Cli {
    /// Convert the class give from CLI to a regex string
    fn to_regex(&self) -> String {
        format!("^{}$", self.class)
    }
    /// Get the window identifier
    fn get_window_identifier<'a>(&self, regex_match: &'a str) -> Option<WindowIdentifier<'a>> {
        match self.cmd.as_str() {
            "alacritty" | "kitty" | "wezterm" => {
                Some(WindowIdentifier::ClassRegularExpression(regex_match))
            }
            "foot" | "gnome-terminal" => Some(WindowIdentifier::Title(regex_match)),
            // It will be assumed that every other application has a class identifier
            _ => Some(WindowIdentifier::ClassRegularExpression(regex_match)),
        }
    }
    /// Silently move the window to the special workspace.
    fn move_to_workspace_silent(&self, regex_match: &str) {
        let res = Dispatch::call(DispatchType::MoveToWorkspaceSilent(
            WorkspaceIdentifierWithSpecial::Special(Some(SPECIAL_WORKSPACE)),
            self.get_window_identifier(regex_match),
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
    fn move_to_workspace(&self, regex_match: &str, workspace_id: i32) {
        let res = Dispatch::call(DispatchType::MoveToWorkspace(
            WorkspaceIdentifierWithSpecial::Id(workspace_id),
            self.get_window_identifier(regex_match),
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
                    "wezterm" => {
                        format!("{} start --class={} -- {}", self.cmd, self.class, &cmd_args)
                    }
                    "gnome-terminal" => {
                        format!("{} --title={} -- {}", self.cmd, self.class, &cmd_args)
                    }
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
            "wezterm" => format!("{} start --class={}", self.cmd, self.class),
            "gnome-terminal" => format!("{} --title={}", self.cmd, self.class),
            // TODO: Add here other commands

            // The default command for every other application is {cmd}
            _ => self.cmd.clone(),
        }
    }

    /// Get the address based on the initial title of the window
    fn get_address_by_initial_title(&self, regex_match: &str) -> Option<Address> {
        debug!(
            "Getting address by initial title: {:#?}",
            Clients::get().unwrap()
        );
        Clients::get()
            .unwrap()
            .clone()
            .iter()
            .find(|client| client.initial_title == regex_match)
            .map(|client| client.address.clone())
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
    let addresses = get_addresses_file();
    let active_workspace_id = Workspace::get_active().unwrap().id;
    match clients
        .iter()
        .find(|client| client.check_title_or_class_or_address(&cli, &addresses))
    {
        Some(client) => {
            // Case 1: There is a client with the same class in a different workspace
            // Move from special workspace or another workspace to the current one (show it)
            if client.workspace.id != active_workspace_id {
                // Avoid moving to the special workspace if it's already there
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
                    // Write the address in the addresses file
                    write_address_in_file(
                        &regex_match,
                        cli.get_address_by_initial_title(&regex_match).unwrap(),
                    );
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
