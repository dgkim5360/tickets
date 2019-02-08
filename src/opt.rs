use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt()]
/// Manage your tickets in the git-way
///
/// The identifier (ID) for tickets must satisfy the following format
///
/// CATEGORY/[TICKET-NAME]
///
/// If TICKET-NAME does not exists, the ID represents the category directory.
///
/// And without SUBCOMMANDS, tickets will show all of your tickets.
pub struct Opt {
    #[structopt(subcommand)]
    pub action: Option<Action>,
}

#[derive(Debug, StructOpt)]
pub enum Action {
    #[structopt(name = "init")]
    /// Make a directory ~/.tickets for initialization
    Init,

    #[structopt(name = "new")]
    /// Create a new directory or ticket
    New {
        /// Target identifier
        id: String,
        #[structopt(short = "m", long = "message")]
        /// Whole message (title and content) of the ticket
        ///
        /// Without this option, the program will open your EDITOR
        /// (from environmental variables) to make the ticket.
        message: Option<String>,
    },

    #[structopt(name = "show")]
    /// Show a certain ticket, or all tickets of a category
    Show {
        /// Target identifier
        id: String,
    },

    #[structopt(name = "edit")]
    /// Edit a certain ticket
    Edit {
        /// Target identifier
        id: String,
        #[structopt(short = "m", long = "message")]
        /// Whole message (title and content) of the ticket
        ///
        /// Without this option, the program will open your EDITOR
        /// (from environmental variables) to edit the ticket.
        message: Option<String>,
    },

    #[structopt(name = "move")]
    /// Move a certain ticket to another category, and rename if specified
    Move {
        /// Target identifier
        id: String,
        /// Destination identifier
        dest_id: String,
    },

    #[structopt(name = "remove")]
    /// Remove a certain ticket or whole category (including itself)
    Remove {
        /// Target identifier
        id: String,
    },
}
