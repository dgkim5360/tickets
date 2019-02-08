extern crate assert_cmd;

use std::process::Command;
use std::time;
use std::thread;
use assert_cmd::prelude::*;

use tickets::purge;

// ================= INIT =================
#[test]
fn the_very_first_run_requires_init() {
    purge().unwrap_or(());
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .assert()
        .failure()
        .stderr("ERROR: NOT INITIALIZED, PLEASE init.\n");
}

#[test]
fn init_successfully() {
    purge().unwrap_or(());
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("init")
        .assert()
        .success()
        .stdout("tickets :: init

SUCCEEDED.
");
}

#[test]
fn fail_to_re_init() {
    purge().unwrap_or(());
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("init")
        .assert()
        .success();

    cmd
        .assert()
        .failure()
        .stderr("tickets :: init

ERROR: The root ~/.tickets already exists.
");
}

// ================= NEW =================
#[test]
fn create_a_new_category() {
    purge().unwrap_or(());
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("init")
        .assert()
        .success();

    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("test/")
        .assert()
        .success()
        .stdout("tickets :: new :: test/

SUCCEEDED.
");
}

#[test]
fn do_not_create_the_same_category_again() {
    purge().unwrap_or(());
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("init")
        .assert()
        .success();
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("test/")
        .assert()
        .success();

    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("test/")
        .assert()
        .failure()
        .stderr("tickets :: new :: test/

ERROR: The path already exists.
");
}

#[test]
fn create_a_new_ticket() {
    purge().unwrap_or(());
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("init")
        .assert()
        .success();
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("test/")
        .assert()
        .success();

    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("test/ID-1234")
        .arg("--message")
        .arg("A fresh new ticket.")
        .assert()
        .success()
        .stdout("tickets :: new :: test/ID-1234

SUCCEEDED.
");
}

#[test]
fn fail_to_create_a_ticket_without_category() {
    purge().unwrap_or(());
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("init")
        .assert()
        .success();

    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("without-you")
        .arg("--message")
        .arg("A fresh new ticket.")
        .assert()
        .failure()
        .stderr("tickets :: new

ERROR: Invalid identifier without-you
");
}

#[test]
fn fail_to_create_a_ticket_with_non_existing_category() {
    purge().unwrap_or(());
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("init")
        .assert()
        .success();

    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("never-exists/ID-1234")
        .arg("--message")
        .arg("It will never happen.")
        .assert()
        .failure()
        .stderr("tickets :: new :: never-exists/ID-1234

ERROR: The category does not exist.
");
}

// ================= SHOW =================
#[test]
fn show_a_valid_ticket() {
    purge().unwrap_or(());
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("init")
        .assert()
        .success();
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("test/")
        .assert()
        .success();

    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("test/ID-1234")
        .arg("--message")
        .arg("A fresh new ticket.")
        .assert()
        .success();

    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("show")
        .arg("test/ID-1234")
        .assert()
        .success()
        .stdout("tickets :: show :: test/ID-1234

A fresh new ticket.
");
}

#[test]
fn show_a_invalid_ticket_id() {
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("show")
        .arg("invalid-ticket")
        .assert()
        .failure()
        .stderr("tickets :: show

ERROR: Invalid identifier invalid-ticket
");
}

#[test]
fn fail_to_show_a_non_existing_ticket() {
    purge().unwrap_or(());
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("show")
        .arg("never/exists")
        .assert()
        .failure()
        .stderr("tickets :: show :: never/exists

ERROR: NOT FOUND.
");
}

#[test]
fn show_an_empty_category() {
    purge().unwrap_or(());
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("init")
        .assert()
        .success();
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("test/")
        .assert()
        .success();

    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("show")
        .arg("test/")
        .assert()
        .success()
        .stdout("tickets :: show :: test/

NO TICKETS.
");
        
}

#[test]
fn failed_to_show_a_non_existing_category() {
    purge().unwrap_or(());
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("init")
        .assert()
        .success();

    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("show")
        .arg("never-exists/")
        .assert()
        .failure()
        .stderr("tickets :: show :: never-exists/

ERROR: NOT FOUND.
");
}

#[test]
fn show_a_category_with_multiple_tickets() {
    purge().unwrap_or(());
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("init")
        .assert()
        .success();

    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("test/")
        .assert()
        .success();

    let millis100 = time::Duration::from_millis(100);

    // create 3 tickets
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("test/ID-1")
        .arg("--message")
        .arg("A fresh new ticket.")
        .assert()
        .success();

    thread::sleep(millis100);
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("test/ID-2")
        .arg("--message")
        .arg("The second ticket.")
        .assert()
        .success();

    thread::sleep(millis100);
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("test/ID-3")
        .arg("--message")
        .arg("A beautiful 3rd ticket.")
        .assert()
        .success();

    // the main test begin
    thread::sleep(millis100);
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("show")
        .arg("test/")
        .assert()
        .success()
        .stdout("tickets :: show :: test/

[ID-1]A fresh new ticket.
[ID-2]The second ticket.
[ID-3]A beautiful 3rd ticket.
");
}

// ================= EDIT =================
#[test]
fn edit_a_ticket() {
    purge().unwrap_or(());
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("init")
        .assert()
        .success();
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("hello/")
        .assert()
        .success();
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("hello/ID-1234")
        .arg("--message")
        .arg("To be edited...")
        .assert()
        .success();

    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("edit")
        .arg("hello/ID-1234")
        .arg("--message")
        .arg("Now edited!")
        .assert()
        .success()
        .stdout("tickets :: edit :: hello/ID-1234

SUCCEEDED.
");

    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("show")
        .arg("hello/ID-1234")
        .assert()
        .success()
        .stdout("tickets :: show :: hello/ID-1234

Now edited!
");
}

#[test]
fn fail_to_edit_a_non_existing_ticket() {
    purge().unwrap_or(());
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("init")
        .assert()
        .success();
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("hello/")
        .assert()
        .success();

    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("edit")
        .arg("hello/ID-1234")
        .arg("--message")
        .arg("Now edited!")
        .assert()
        .failure()
        .stderr("tickets :: edit :: hello/ID-1234

ERROR: The path does not exist.
");
}

#[test]
fn fail_to_edit_a_category() {
    purge().unwrap_or(());
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("init")
        .assert()
        .success();
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("hello/")
        .assert()
        .success();

    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("edit")
        .arg("hello/")
        .arg("--message")
        .arg("Hope to edit!")
        .assert()
        .failure()
        .stderr("tickets :: edit :: hello/

ERROR: The category cannot be edited.
");
}

// ================= MOVE =================
#[test]
fn move_a_ticket() {
    purge().unwrap_or(());
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("init")
        .assert()
        .success();
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("test1/")
        .assert()
        .success();
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("test2/")
        .assert()
        .success();
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("test1/ID-1234")
        .arg("--message")
        .arg("To be moved...")
        .assert()
        .success();

    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("move")
        .arg("test1/ID-1234")
        .arg("test2/ID-1234")
        .assert()
        .success()
        .stdout("tickets :: move :: test1/ID-1234 => test2/ID-1234

SUCCEEDED.
");

    // Move back again,
    // but the destination representation is directory.
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("move")
        .arg("test2/ID-1234")
        .arg("test1/")
        .assert()
        .success()
        .stdout("tickets :: move :: test2/ID-1234 => test1/

SUCCEEDED.
");
}

#[test]
fn reject_to_move_from_invalid_id() {
    purge().unwrap_or(());
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("init")
        .assert()
        .success();

    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("move")
        .arg("in/valid/ID-1234")
        .arg("test2/")
        .assert()
        .failure()
        .stderr("tickets :: move

ERROR: Invalid identifier in/valid/ID-1234
");
}

#[test]
fn reject_to_move_from_invalid_destination_id() {
    purge().unwrap_or(());
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("init")
        .assert()
        .success();
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("test1/")
        .assert()
        .success();
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("test1/ID-1234")
        .arg("--message")
        .arg("To be moved...")
        .assert()
        .success();

    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("move")
        .arg("test1/ID-1234")
        .arg("in/valid/id")
        .assert()
        .failure()
        .stderr("tickets :: move

ERROR: Invalid identifier in/valid/id
");
}

#[test]
fn fail_to_move_a_non_existing_ticket() {
    purge().unwrap_or(());
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("init")
        .assert()
        .success();
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("test1/")
        .assert()
        .success();

    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("move")
        .arg("test1/ID-1234")
        .arg("test2/")
        .assert()
        .failure()
        .stderr("tickets :: move :: test1/ID-1234 => test2/

ERROR: the source path is not an existing regular file
");
}

#[test]
fn fail_to_move_a_ticket_to_non_existing_category() {
    purge().unwrap_or(());
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("init")
        .assert()
        .success();
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("test1/")
        .assert()
        .success();
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("test1/ID-1234")
        .arg("--message")
        .arg("To be moved...")
        .assert()
        .success();

    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("move")
        .arg("test1/ID-1234")
        .arg("test2/ID-1234")
        .assert()
        .failure()
        .stderr("tickets :: move :: test1/ID-1234 => test2/ID-1234

ERROR: No such file or directory (os error 2)
");
}

#[test]
fn move_a_whole_category() {
    purge().unwrap_or(());
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("init")
        .assert()
        .success();
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("test1/")
        .assert()
        .success();
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("test2/")
        .assert()
        .success();
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("test1/ID-1233")
        .arg("--message")
        .arg("To be moved...")
        .assert()
        .success();
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("test1/ID-1234")
        .arg("--message")
        .arg("To be moved too...")
        .assert()
        .success();

    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("move")
        .arg("test1/")
        .arg("test2/")
        .assert()
        .success()
        .stdout("tickets :: move :: test1/ => test2/

SUCCEEDED.
");

    // additional check: show them!
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("show")
        .arg("test2/")
        .assert()
        .success()
        .stdout("tickets :: show :: test2/

[ID-1233]To be moved...
[ID-1234]To be moved too...
");
}

// ================= REMOVE =================
#[test]
fn remove_a_ticket() {
    purge().unwrap_or(());
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("init")
        .assert()
        .success();
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("hello/")
        .assert()
        .success();
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("hello/ID-1233")
        .arg("--message")
        .arg("To be removed...")
        .assert()
        .success();

    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("remove")
        .arg("hello/ID-1233")
        .assert()
        .success()
        .stdout("tickets :: remove :: hello/ID-1233

SUCCEEDED.
");

    // check the ticket is removed.
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("show")
        .arg("hello/ID-1233")
        .assert()
        .failure()
        .stderr("tickets :: show :: hello/ID-1233

ERROR: NOT FOUND.
");
}

#[test]
fn fail_to_remove_non_existing_ticket() {
    purge().unwrap_or(());
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("init")
        .assert()
        .success();
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("hello/")
        .assert()
        .success();

    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("remove")
        .arg("hello/ID-1233")
        .assert()
        .failure()
        .stderr("tickets :: remove :: hello/ID-1233

ERROR: No such file or directory (os error 2)
");
}

#[test]
fn remove_a_whole_category() {
    purge().unwrap_or(());
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("init")
        .assert()
        .success();
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("hello/")
        .assert()
        .success();
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("hello/ID-1233")
        .arg("--message")
        .arg("To be removed...")
        .assert()
        .success();
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("hello/ID-1234")
        .arg("--message")
        .arg("To be removed...")
        .assert()
        .success();

    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("remove")
        .arg("hello/")
        .assert()
        .success()
        .stdout("tickets :: remove :: hello/

SUCCEEDED.
");

    // check the hello category is removed.
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("show")
        .arg("hello/")
        .assert()
        .failure()
        .stderr("tickets :: show :: hello/

ERROR: NOT FOUND.
");
}

// ================= tickets =================
#[test]
fn tickets () {
    purge().unwrap_or(());
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("init")
        .assert()
        .success();
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("hello/")
        .assert()
        .success();
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("hello/ID-1233")
        .arg("--message")
        .arg("Final Tests")
        .assert()
        .success();
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("hello/ID-1234")
        .arg("--message")
        .arg("Real Final Tests")
        .assert()
        .success();
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("world/")
        .assert()
        .success();
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("world/WTF-1")
        .arg("--message")
        .arg("What The ...?")
        .assert()
        .success();
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("world/WTF-2")
        .arg("--message")
        .arg("Don't Do That")
        .assert()
        .success();

    let mut cmd = Command::main_binary().unwrap();
    cmd
        .assert()
        .success()
        .stdout("hello/
[ID-1233]Final Tests
[ID-1234]Real Final Tests

world/
[WTF-1]What The ...?
[WTF-2]Don't Do That
");
}

#[test]
fn fail_to_ticket_without_init() {
    purge().unwrap_or(());

    let mut cmd = Command::main_binary().unwrap();
    cmd
        .assert()
        .failure()
        .stderr("ERROR: NOT INITIALIZED, PLEASE init.\n");
}

#[test]
fn ticket_with_no_categories() {
    purge().unwrap_or(());
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("init")
        .assert()
        .success();

    let mut cmd = Command::main_binary().unwrap();
    cmd
        .assert()
        .success()
        .stdout("NO TICKETS.\n");
}

#[test]
fn ticket_with_categories_but_no_tickets() {
    purge().unwrap_or(());
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("init")
        .assert()
        .success();
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("hello/")
        .assert()
        .success();
    let mut cmd = Command::main_binary().unwrap();
    cmd
        .arg("new")
        .arg("world/")
        .assert()
        .success();

    let mut cmd = Command::main_binary().unwrap();
    cmd
        .assert()
        .success()
        .stdout("hello/
NO TICKETS.

world/
NO TICKETS.
");
}
