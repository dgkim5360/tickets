# tickets

`tickets` is a CLI program dealing with text files and managing them, through a simple representation for a ticket identification.

This project is currently in pre-alpha stage and aims to support the minimal features. Any suggestion or review is always welcome!

### Installation

Currently in order to install `tickets`, you need Rust stable and its package manager Cargo.

```
$ git clone <this repo url>
$ cd tickets
$ cargo install --path .
```

### Ticket Identifier

`tickets` needs the correct ticket identifier (ID) as an input, and its format is `CATEGORY/[TICKET_NAME]`. Ticket IDs can represent the category and the ticket as following:
- A category itself: `category/`
- A ticket (and it must belong to a category): `category/ticket-name`

And it cannot represent other cases such as:
- A ticket without category: `ticket-id` or `/ticket-name`
- A ticket with nested categories: `cat1/cat2/ticket-name`
- A category, but without slash: `category`
- The root notation: `/` or anything starts with `/`

### Get Started

`tickets` takes subcommands, like `git` does. The first subcommand to execute is `init`, which just makes a directory `~/.tickets`.

```
$ tickets
ERROR : NOT INITIALIZED, PLEASE init.

$ # ===
$ # init just makes a directory ~/.tickets
$ # ===
$ tickets init
tickets :: init

SUCCEEDED.
```

`new` subcommand creates a category directory and a ticket. When creating ticket, like Git does, you can convey the title of the ticket via `-m` option. On the other hand, without the option `-m`, `tickets` opens your default editor (set by environment variable `EDITOR`) to edit the title and the contents of the ticket.

`show` subcommand is all about listing and displaying.

```
$ tickets new open/ticket-1234 -m"A New Ticket"
tickets :: new :: open/ticket-1234

ERROR: No such file or directory (os error 2)

$ tickets new open/
ticket :: new :: open/

SUCCEEDED.

$ # ===
$ # Caution: the category ID should contain a trailing slash /
$ # ===
$ tickets show open
tickets :: show

ERROR: Invalid identifier open

$ tickets show open/
tickets :: show :: open/

NO TICKETS.

$ # ===
$ # Without -m, tickets opens your EDITOR
$ # ===
$ tickets new open/ticket-1234 -m"A New Ticket"
tickets :: new :: open/ticket-1234

SUCCEEDED.

$ ticket show open/
tickets :: show :: open/

[ticket-1234]A New Ticket

$ tickets show open/ticket-1234
tickets :: show :: open/ticket-1234

A New Ticket

$ # ===
$ # Assuming more new tickets added ...
$ # ===
$ tickets show open/
tickets :: show :: open/

[ticket-1234]A New Ticket
[hotfix-86]Incorrect calculation logic
[ticket-1238]More validations
```

`edit` subcommand can modify existing tickets. It has the same `-m` option as `new` does.

```
$ tickets edit open/ticket-1234
# ===
# (Editing this ticket in my EDITOR ...)
# ===
tickets :: edit :: open/ticket-1234

SUCCEEDED.

$ tickets show open/
tickets :: show :: open/

[ticket-1234]Suggestions for tickets
[hotfix-86]Incorrect calculation logic
[ticket-1238]More validations

$ tickets show open/ticket-1234
tickets :: show :: open/ticket-1234

Suggestions for tickets

Some suggestions:
- More detailed error messages
- Colored outputs
- Nested category structure
- Verbose option when listing tickets
- JIRA integration?
```

`move` subcommand can move a ticket to another category and rename it within its category or between categories.

```
$ # ===
$ # Let's move one to another category
$ # And I forgot to make that category first
$ # ===
$ tickets move open/ticket-1234 in-progress/
tickets :: move :: open/ticket-1234 => in-progress/

ERROR: No such file or directory (os error 2)

$ tickets new in-progress/
tickets :: new :: in-progress/

SUCCEEDED.

$ tickets move open/ticket-1234 in-progress/
tickets :: move :: open/ticket-1234 => in-progress/

SUCCEEDED.
```

Finally, simply calling `tickets` shows all.

```
$ # ===
$ # Just calling tickets shows everything
$ # ===
$ tickets
open/
[hotfix-86]Incorrect calculation logic
[ticket-1238]More validations

in-progress/
[ticket-1234]Suggestions for tickets
```

`remove` subcommand deletes a whole category or a ticket.

```
$ # ===
$ # Possible to remove a ticket and a whole category
$ # ===
$ tickets remove open/hotfix-86
tickets :: remove :: open/hotfix-86

SUCCEEDED.

$ tickets
open/
[ticket-1238]More validations

in-progress/
[ticket-1234]Suggestions for tickets

$ tickets remove open/
tickets :: remove :: open/

SUCCEEDED.

$ tickets
in-progress/
[ticket-1234]Suggestions for tickets

$ # ===
$ # If you want purge everything, just delete ~/.tickets
$ # ===
$ rm -r ~/.tickets
$ tickets
tickets

ERROR : NOT INITIALIZED, PLEASE init.
```
