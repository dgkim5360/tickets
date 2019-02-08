extern crate dirs;
extern crate exitcode;

use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;

pub mod opt;
pub mod ticket;

fn format_header_init() -> String {
    String::from("tickets :: init")
}

fn format_header_new(ticket: &ticket::Ticket) -> String {
    let id = match &ticket.id {
        Some(id_) => id_,
        None => "",
    };
    format!("tickets :: new :: {}/{}",
            ticket.category,
            id)
}

fn format_header_show(ticket: &ticket::Ticket) -> String {
    let id = match &ticket.id {
        Some(id_) => id_,
        None => "",
    };
    format!("tickets :: show :: {}/{}",
            ticket.category,
            id)
}

fn format_header_edit(ticket: &ticket::Ticket) -> String {
    let id = match &ticket.id {
        Some(id_) => id_,
        None => "",
    };
    format!("tickets :: edit :: {}/{}",
            ticket.category,
            id)
}

fn format_header_move(start_ticket: &ticket::Ticket,
                      dest_ticket: &ticket::Ticket) -> String {
    let start_id = match &start_ticket.id {
        Some(id) => id,
        None => "",
    };
    let dest_id = match &dest_ticket.id {
        Some(id) => id,
        None => "",
    };
    format!("tickets :: move :: {}/{} => {}/{}",
            start_ticket.category, start_id,
            dest_ticket.category, dest_id)
}

fn format_header_remove(ticket: &ticket::Ticket) -> String {
    let id = match &ticket.id {
        Some(id_) => id_,
        None => "",
    };
    format!("tickets :: remove :: {}/{}",
            ticket.category,
            id)
}

pub fn purge() -> io::Result<()> {
    let root = get_path_root();
    fs::remove_dir_all(root)?;
    Ok(())
}

fn get_path_root() -> PathBuf {
    let home = dirs::home_dir().unwrap();
    let root = home.join(".tickets");
    root
}

fn initialize_root() -> io::Result<()> {
    let root = get_path_root();
    if root.exists() {
        return Err(io::Error::new(io::ErrorKind::AlreadyExists,
                                  "The root ~/.tickets already exists."));
    }
    fs::create_dir(root)?;
    Ok(())
}

// TODO: propagate smoothly the native env::Error
fn open_editor(path: &PathBuf) -> io::Result<()> {
    let result_env_editor = std::env::var("EDITOR");
    let cmd_editor;
    match result_env_editor {
        Ok(env_value) => {
            cmd_editor = env_value;
        },
        Err(error) => match error {
            std::env::VarError::NotPresent => {
                return Err(
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        "The environment variable EDITOR is not set.",
                    )
                );
            },
            std::env::VarError::NotUnicode(_) => {
                return Err(
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        "The environment variable EDITOR \
                        is not a valid unicode.",
                    )
                );
            }
        },
    };
    let mut cmd = Command::new(cmd_editor);
    cmd
        .arg(path)
        .status()
        .expect("Failed to open your EDITOR.");
    Ok(())
}

fn show_all() -> io::Result<String> {
    let root = get_path_root();
    let mut strings_display: Vec<String> = Vec::new();
    let mut categories: Vec<ticket::Ticket> = Vec::new();

    let iter_dir = fs::read_dir(&root)?;
    for entry in iter_dir {
        let entry = entry?;
        let path_entry = entry.path();
        if path_entry.is_dir() {
            let id_ticket = path_entry.strip_prefix(&root).unwrap();
            let id_ticket = format!("{}/", id_ticket.to_string_lossy());
            let mut ticket = ticket::Ticket::from(id_ticket, None)
                .unwrap();
            ticket.read()?;
            categories.push(ticket);
        }
    }
    if categories.is_empty() {
        return Ok("NO TICKETS.".to_string());
    }
    categories.sort_by_key(|tic| tic.modified_at);

    for ticket in &categories {
        let str_category = format!("{}/\n{}",
                                   ticket.category,
                                   ticket);
        strings_display.push(str_category);
    }
    if strings_display.is_empty() {
        return Ok("NO TICKETS.".to_string());
    }

    Ok(strings_display.join("\n\n"))
}

pub fn die(status: exitcode::ExitCode, message: String) {
    if exitcode::is_error(status) {
        eprintln!("{}", message);
    }
    else {
        println!("{}", message);
    }
    std::process::exit(status);
}

pub fn match_action(opt: opt::Opt) -> (exitcode::ExitCode, String) {
    let exit_code: exitcode::ExitCode;
    let sys_message: String;

    if let Some(action) = opt.action {
        match action {
            opt::Action::Init => {
                let format_header = format_header_init();
                match initialize_root() {
                    Ok(()) => {
                        exit_code = exitcode::OK;
                        sys_message = format!("{}\n\nSUCCEEDED.",
                                              format_header);
                    },
                    Err(error) => {
                        exit_code = exitcode::IOERR;
                        sys_message = format!("{}\n\nERROR: {}",
                                              format_header,
                                              error);
                    },
                }
            },
            opt::Action::New { id, message } => {
                let result_ticket = ticket::Ticket::from(id, message);
                match result_ticket {
                    Ok(ticket) => {
                        let format_header = format_header_new(&ticket);
                        match ticket.new() {
                            Ok(()) => {
                                exit_code = exitcode::OK;
                                sys_message = format!("{}\n\nSUCCEEDED.",
                                                      format_header);
                            },
                            Err(error) => {
                                exit_code = exitcode::IOERR;
                                sys_message = format!("{}\n\nERROR: {}",
                                                      format_header,
                                                      error);
                            },
                        };
                    },
                    Err(error) => {
                        exit_code = exitcode::IOERR;
                        sys_message = format!("tickets :: new\n\nERROR: {}",
                                              error);
                    },
                };
            },
            opt::Action::Edit { id, message } => {
                let result_ticket = ticket::Ticket::from(id, message);
                match result_ticket {
                    Ok(ticket) => {
                        let format_header = format_header_edit(&ticket);
                        match ticket.edit() {
                            Ok(()) => {
                                exit_code = exitcode::OK;
                                sys_message = format!("{}\n\nSUCCEEDED.",
                                                      format_header);
                            },
                            Err(error) => {
                                exit_code = exitcode::IOERR;
                                sys_message = format!("{}\n\nERROR: {}",
                                                      format_header,
                                                      error);
                            },
                        };
                    },
                    Err(error) => {
                        exit_code = exitcode::IOERR;
                        sys_message = format!("tickets :: edit\n\nERROR: {}",
                                              error);
                    },
                };
            },
            opt::Action::Show { id } => {
                let result_ticket = ticket::Ticket::from(id, None);
                match result_ticket {
                    Ok(mut ticket) => {
                        let format_header = format_header_show(&ticket);
                        if !ticket.path.exists() {
                            exit_code = exitcode::IOERR;
                            sys_message = format!("{}\n\nERROR: NOT FOUND.",
                                                  format_header);
                        }
                        else if ticket.is_dir {
                            exit_code = exitcode::OK;
                            sys_message = format!("{}\n\n{}",
                                                  format_header,
                                                  ticket);
                        }
                        else {
                            // TODO: Display includes this individual read
                            ticket.read().unwrap();
                            let ticket_title = match &ticket.title {
                                Some(title) => &title[..],
                                None => "",
                            };
                            let ticket_message = match ticket.message {
                                Some(message) => format!("\n\n{}", message),
                                None => String::new(),
                            };
                            exit_code = exitcode::OK;
                            sys_message = format!("{}\n\n{}{}",
                                                  format_header,
                                                  ticket_title,
                                                  ticket_message);
                        }
                    },
                    Err(error) => {
                        exit_code = exitcode::IOERR;
                        sys_message = format!("tickets :: show\n\nERROR: {}",
                                              error);
                    },
                };
            },
            opt::Action::Move { id, dest_id } => {
                let result_start_ticket = ticket::Ticket::from(id, None);
                let result_dest_ticket = ticket::Ticket::from(dest_id, None);
                let start_ticket;
                let dest_ticket;
                match result_start_ticket {
                    Ok(ticket) => {
                        start_ticket = ticket;
                    },
                    Err(error) => {
                        exit_code = exitcode::IOERR;
                        sys_message = format!("tickets :: move\n\nERROR: {}",
                                              error);
                        return (exit_code, sys_message);
                    },
                };
                match result_dest_ticket {
                    Ok(ticket) => {
                        dest_ticket = ticket;
                    },
                    Err(error) => {
                        exit_code = exitcode::IOERR;
                        sys_message = format!("tickets :: move\n\nERROR: {}",
                                              error);
                        return (exit_code, sys_message);
                    },
                };
                let format_header = format_header_move(&start_ticket,
                                                       &dest_ticket);
                if start_ticket.is_dir {
                    match start_ticket.move_all(&dest_ticket) {
                        Ok(()) => {
                            exit_code = exitcode::OK;
                            sys_message = format!("{}\n\nSUCCEEDED.",
                                                  format_header);
                        },
                        Err(error) => {
                            exit_code = exitcode::IOERR;
                            sys_message = format!("{}\n\nERROR: {}",
                                                  format_header,
                                                  error);
                        },
                    };
                }
                else {
                    match start_ticket.move_(&dest_ticket) {
                        Ok(()) => {
                            exit_code = exitcode::OK;
                            sys_message = format!("{}\n\nSUCCEEDED.",
                                                  format_header);
                        },
                        Err(error) => {
                            exit_code = exitcode::IOERR;
                            sys_message = format!("{}\n\nERROR: {}",
                                                  format_header,
                                                  error);
                        },
                    }
                }
            },
            opt::Action::Remove { id } => {
                let result_ticket = ticket::Ticket::from(id, None);
                let ticket;
                match result_ticket {
                    Ok(ticket_) => {
                        ticket = ticket_;
                    },
                    Err(error) => {
                        exit_code = exitcode::IOERR;
                        sys_message = format!("tickets :: remove\n\nERROR: {}",
                                              error);
                        return (exit_code, sys_message);
                    },
                };
                let format_header = format_header_remove(&ticket);
                match ticket.remove() {
                    Ok(()) => {
                        exit_code = exitcode::OK;
                        sys_message = format!("{}\n\nSUCCEEDED.", format_header);
                    },
                    Err(error) => {
                        exit_code = exitcode::IOERR;
                        sys_message = format!("{}\n\nERROR: {}",
                                              format_header,
                                              error);
                    },
                };
            },
        };
    }
    else {
        let pathbuf_root = get_path_root();
        if !pathbuf_root.exists() {
            exit_code = exitcode::IOERR;
            sys_message = String::from("ERROR: NOT INITIALIZED, PLEASE init.");
            return (exit_code, sys_message);
        }
        match show_all() {
            Ok(str_show_all) => {
                exit_code = exitcode::OK;
                sys_message = format!("{}", str_show_all);
            },
            Err(error) => {
                exit_code = exitcode::IOERR;
                sys_message = format!("ERROR: {}", error);
            }
        };
    }
    (exit_code, sys_message)
}

#[cfg(test)]
mod tests{
    use std::path::PathBuf;

    #[test]
    fn get_path_root_gives_the_constant_root_path() {
        let root = super::get_path_root();
        let answer = dirs::home_dir().unwrap().join(".tickets");
        assert_eq!(root, answer);
    }

    #[test]
    fn initialize_root() {
        super::purge().unwrap_or(());
        let result_init = super::initialize_root();
        assert!(result_init.is_ok());

        let root = super::get_path_root();
        assert!(root.exists());
    }

    #[test]
    fn fail_to_initialize_root_when_the_root_exists() {
        super::purge().unwrap_or(());
        super::initialize_root().unwrap();

        let result_init = super::initialize_root();
        assert!(result_init.is_err());
        match result_init {
            Ok(()) => {
                panic!("This should never happen.");
            },
            Err(error) => {
                assert_eq!(error.to_string(),
                           "The root ~/.tickets already exists.");
            },
        };
    }

    #[test]
    fn purge() {
        super::purge().unwrap_or(());
        let root = super::get_path_root();
        super::initialize_root().unwrap();
        let result_purge = super::purge();
        assert!(result_purge.is_ok());
        assert!(!root.exists());
    }

    #[test]
    fn fail_to_purge_when_the_root_does_not_exist() {
        super::purge().unwrap_or(());
        let result_purge = super::purge();
        assert!(result_purge.is_err());
        match result_purge {
            Ok(()) => {
                panic!("This should never happen.");
            },
            Err(error) => {
                assert_eq!(error.to_string(),
                           "No such file or directory (os error 2)");
            },
        }
    }

    #[test]
    fn fail_to_open_editor_without_env_var_editor() {
        std::env::remove_var("EDITOR");
        let path = PathBuf::from("never/existing/path");
        let result = super::open_editor(&path);
        assert!(result.is_err());

        match result {
            Ok(()) => {
                panic!("This should never happen.");
            },
            Err(error) => {
                assert_eq!(error.to_string(),
                           "The environment variable EDITOR is not set.");
            },
        };
    }

    // TODO: figure out how to test the external command
    // without actually executing it.
    #[test]
    #[ignore]
    fn open_editor() {
    }
}
