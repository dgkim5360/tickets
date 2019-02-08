use std::fmt;
use std::fs;
use std::io::{self, BufRead};
use std::path;
use std::time;

fn identify_id(id: &str) -> Result<(String, Option<String>), String> {
    let count = id.matches("/").count();
    match count {
        1 => {
            let split: Vec<&str> = id.split("/").collect();
            let category = split[0];
            let ticket_id = if split[1].is_empty() {
                None
            } else {
                Some(split[1].to_string())
            };
            Ok((category.to_string(), ticket_id))
        },
        _ => {
            Err(format!("Invalid identifier {}", id))
        },
    }
}

#[derive(Debug)]
pub struct Ticket {
    pub path: path::PathBuf,
    pub id: Option<String>,
    pub category: String,
    pub title: Option<String>,
    pub message: Option<String>,
    pub is_dir: bool,
    pub modified_at: Option<time::SystemTime>,
}

impl fmt::Display for Ticket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_dir {
            let vec_tickets = self.collect().unwrap();
            if vec_tickets.is_empty() {
                write!(f, "NO TICKETS.")
            }
            else {
                let vec_id: Vec<String> = vec_tickets
                    .iter()
                    .map(|tic| format!("{}", tic))
                    .collect();
                let str_tickets = vec_id.join("\n");
                write!(f, "{}", str_tickets)
            }
        }
        else {
            let id = match &self.id {
                Some(id_) => format!("[{}]", id_),
                None => String::new(),
            };
            let title = match &self.title {
                Some(title_) => title_,
                None => "",
            };
            write!(f, "{}{}", id, title)
        }
    }
}

impl Ticket {
    pub fn from(str_path: String,
                title: Option<String>) -> Result<Ticket, String> {
        let (category, id) = identify_id(&str_path)?;
        let path = super::get_path_root()
            .join(str_path);
        let is_dir = id == None;
        Ok(
            Ticket {
                path,
                id,
                category,
                title,
                message: None,
                is_dir,
                modified_at: None,
            }
        )
    }

    pub fn new(&self) -> io::Result<()> {
        if self.path.exists() {
            return Err(io::Error::new(io::ErrorKind::AlreadyExists,
                                      "The path already exists."));
        }
        if self.is_dir {
            self.initialize_directory()?;
            return Ok(());
        }

        if let Some(parent) = self.path.parent() {
            if !parent.exists() {
                return Err(io::Error::new(io::ErrorKind::NotFound,
                                          "The category does not exist."));
            }
        }

        self.write()
    }

    pub fn edit(&self) -> io::Result<()> {
        if !self.path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound,
                                      "The path does not exist."));
        }
        if self.is_dir {
            return Err(io::Error::new(io::ErrorKind::InvalidInput,
                                      "The category cannot be edited."));
        }
        self.write()
    }

    pub fn write(&self) -> io::Result<()> {
        if let Some(title) = &self.title {
            fs::write(&self.path, title)?;
        }
        else {
            super::open_editor(&self.path)?;
        }
        Ok(())
    }

    pub fn initialize_directory(&self) -> io::Result<()> {
        let mut path = super::get_path_root();
        path.push(&self.category);

        if path.exists() {
            return Err(io::Error::new(io::ErrorKind::AlreadyExists,
                                      "The path already exists"));
        }
        fs::create_dir_all(&path)?;
        Ok(())
    }

    pub fn read(&mut self) -> io::Result<()> {
        let file = fs::File::open(&self.path)?;
        if !self.is_dir {
            let mut reader = io::BufReader::new(&file);
            let mut title = String::new();
            let mut vec_buf = vec![];
            reader.read_line(&mut title)?;
            reader.read_until(0, &mut vec_buf)?;
            let message = String::from_utf8(vec_buf).unwrap();

            self.title = Some(title.trim().to_string());
            if !message.is_empty() {
                self.message = Some(message.trim().to_string());
            }
        }

        let metadata = file.metadata()?;
        let modified = metadata.modified().unwrap();
        self.modified_at = Some(modified);
        Ok(())
    }

    pub fn collect(&self) -> io::Result<Vec<Ticket>> {
        if !self.is_dir {
            return Err(
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "A path for the file is given, expected a directory.",
                    )
                );
        }
        let path_root = super::get_path_root();
        let mut tickets: Vec<Ticket> = Vec::new();
        let iter_dir = fs::read_dir(&self.path)?;
        for entry in iter_dir {
            let entry = entry?;
            let path_entry = entry.path();
            let id_ticket = path_entry.strip_prefix(&path_root).unwrap();
            let mut ticket = Ticket::from(
                id_ticket.to_string_lossy().into_owned(),
                None)
                .unwrap();
            ticket.read()?;
            tickets.push(ticket);
        }
        tickets.sort_by_key(|tic| tic.modified_at);
        // println!("{:?}", tickets);
        Ok(tickets)
    }

    pub fn move_(&self, dest_ticket: &Ticket) -> io::Result<()> {
        let self_id = match &self.id {
            Some(id) => id,
            None => "",
        };
        let mut dest_path: path::PathBuf = dest_ticket.path.clone();
        if dest_ticket.is_dir {
            dest_path.push(self_id);
        }
        fs::copy(&self.path, dest_path)?;
        fs::remove_file(&self.path)?;
        Ok(())
    }

    pub fn move_all(&self, dest_dir: &Ticket) -> io::Result<()> {
        if !self.is_dir {
            return Err(
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "The source path for the file is given, expected a directory.",
                    )
                );
        }
        if !dest_dir.is_dir {
            return Err(
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "The destination path for the file is given, expected a directory.",
                    )
                );
        }
        let path_root = super::get_path_root();
        let iter_dir = fs::read_dir(&self.path)?;
        for entry in iter_dir {
            let entry = entry?;
            let path_entry = entry.path();
            let id_ticket = path_entry.strip_prefix(&path_root).unwrap();
            let ticket = Ticket::from(
                id_ticket.to_string_lossy().into_owned(),
                None)
                .unwrap();
            ticket.move_(&dest_dir)?;
        }
        Ok(())
    }

    pub fn remove(&self) -> io::Result<()> {
        if self.is_dir {
            fs::remove_dir_all(&self.path)?;
        }
        else {
            fs::remove_file(&self.path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use std::thread;

    // ========== identify_id ==========
    #[test]
    fn identify_id() {
        assert_eq!(
            super::identify_id("where/hello-1234"),
            Ok((String::from("where"), Some(String::from("hello-1234"))))
        );
    }

    #[test]
    fn fail_to_identify_id_without_slash() {
        let result = super::identify_id("hello-1234");
        assert!(result.is_err());
        match result {
            Ok((_category, _id)) => {
                panic!("This should naver happen.");
            },
            Err(error) => {
                assert_eq!(error, "Invalid identifier hello-1234");
            },
        }
    }

    #[test]
    fn fail_to_identify_a_bad_id() {
        assert_eq!(super::identify_id("really/bad/id"),
                   Err(String::from("Invalid identifier really/bad/id")));
    }

    #[test]
    fn identify_directory() {
        assert_eq!(super::identify_id("where/"),
                   Ok((String::from("where"), None)));
    }

    // ========== ticket::from ==========
    #[test]
    fn instantiate_a_ticket_from_a_valid_ticket() {
        let ticket = super::Ticket::from("valid/ticket".to_string(), None);
        assert!(ticket.is_ok());
        let ticket = ticket.unwrap();
        assert_eq!(ticket.is_dir, false);
    }

    #[test]
    fn ticket_from_a_valid_category() {
        let ticket = super::Ticket::from("valid/".to_string(), None);
        assert!(ticket.is_ok());
        let ticket = ticket.unwrap();
        assert_eq!(ticket.is_dir, true);
    }

    #[test]
    fn fail_to_ticket_from_id_without_slash() {
        let ticket = super::Ticket::from("ticket".to_string(), None);
        assert!(ticket.is_err());
        match ticket {
            Err(err) => {
                assert_eq!(err.to_string(), "Invalid identifier ticket".to_string());
            },
            Ok(_) => {
                panic!("This should never happen.");
            },
        }
    }

    #[test]
    fn fail_to_ticket_from_invalid_id() {
        let ticket = super::Ticket::from("w/t/f/ticket".to_string(), None);
        assert!(ticket.is_err());
        match ticket {
            Err(err) => {
                assert_eq!(err.to_string(),
                           "Invalid identifier w/t/f/ticket".to_string());
            },
            Ok(_) => {
                panic!("This should never happen.");
            },
        }
    }

    // ========== ticket.new ==========
    #[test]
    fn ticket_new_a_category() {
        super::super::purge().unwrap_or(());
        super::super::initialize_root().unwrap_or(());
        let ticket = super::Ticket::from("new/".to_string(), None)
            .unwrap();
        let result_new = ticket.new();
        assert!(result_new.is_ok());
        assert!(ticket.path.exists());
        assert!(ticket.path.is_dir());
    }

    #[test]
    fn ticket_new_a_ticket() {
        super::super::purge().unwrap_or(());
        super::super::initialize_root().unwrap_or(());
        let ticket = super::Ticket::from("hello/new_ticket".to_string(),
                                         Some("Some contents".to_string()))
            .unwrap();
        ticket.initialize_directory().unwrap();
        let result_new = ticket.new();
        assert!(result_new.is_ok());
        assert!(ticket.path.exists());

        let title = super::fs::read_to_string(ticket.path).unwrap();
        assert_eq!(ticket.title.unwrap(), title);
    }

    #[test]
    fn fail_to_ticket_new_a_ticket_without_directory() {
        super::super::purge().unwrap_or(());
        super::super::initialize_root().unwrap_or(());
        let ticket = super::Ticket::from("never-exists/new_ticket".to_string(),
                                         Some("Some contents".to_string()))
            .unwrap();
        let result_new = ticket.new();
        assert!(result_new.is_err());
        assert!(!ticket.path.exists());

        match result_new {
            Ok(()) => {
                panic!("This should never happen");
            },
            Err(error) => {
                assert_eq!(error.to_string(), "The category does not exist.");
            },
        }
    }

    // ========== ticket.edit ==========
    #[test]
    fn ticket_edit_a_ticket_with_message_given() {
        super::super::purge().unwrap_or(());
        super::super::initialize_root().unwrap_or(());
        let path_root = super::super::get_path_root();
        let path_dir = path_root.join("hello/");
        super::fs::create_dir(&path_dir).unwrap();
        let path = path_dir.join("ticket");
        super::fs::write(path, "ticket title\n\nand some content").unwrap();

        let ticket = super::Ticket::from("hello/ticket".to_string(),
                                         Some("Edited title".to_string()))
            .unwrap();
        let result_edit = ticket.edit();
        assert!(result_edit.is_ok());
        assert!(ticket.path.exists());

        let title = super::fs::read_to_string(ticket.path).unwrap();
        assert_eq!(ticket.title.unwrap(), title);
    }

    #[test]
    #[ignore]
    fn ticket_edit_a_ticket_through_editor() {
        super::super::purge().unwrap_or(());
        super::super::initialize_root().unwrap_or(());
    }

    #[test]
    fn fail_to_ticket_edit_a_non_existing_ticket() {
        super::super::purge().unwrap_or(());
        super::super::initialize_root().unwrap_or(());
        let path_root = super::super::get_path_root();
        let path_dir = path_root.join("hello/");
        super::fs::create_dir(&path_dir).unwrap();

        let ticket = super::Ticket::from("hello/ticket".to_string(),
                                         Some("Edited title".to_string()))
            .unwrap();
        let result_edit = ticket.edit();
        assert!(result_edit.is_err());
        assert!(!ticket.path.exists());
        match result_edit {
            Ok(()) => {
                panic!("This should never happen.");
            },
            Err(error) => {
                assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
                assert_eq!(error.to_string(),
                           "The path does not exist.".to_string());
            },
        };
    }

    #[test]
    fn fail_to_ticket_edit_a_directory() {
        super::super::purge().unwrap_or(());
        super::super::initialize_root().unwrap_or(());
        let path_root = super::super::get_path_root();
        let path_dir = path_root.join("hello/");
        super::fs::create_dir(&path_dir).unwrap();

        let ticket = super::Ticket::from("hello/".to_string(),
                                         Some("Edited title".to_string()))
            .unwrap();
        let result_edit = ticket.edit();
        assert!(result_edit.is_err());
        match result_edit {
            Ok(()) => {
                panic!("This should never happen.");
            },
            Err(error) => {
                assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
                assert_eq!(error.to_string(),
                           "The category cannot be edited.".to_string());
            },
        };
    }

    // ========== ticket.read ==========
    #[test]
    fn ticket_read_a_title() {
        super::super::purge().unwrap_or(());
        super::super::initialize_root().unwrap_or(());
        let mut ticket = super::Ticket::from("test/new_ticket".to_string(),
                                             None)
            .unwrap();
        ticket.initialize_directory().unwrap();

        let path = super::super::get_path_root()
            .join("test/new_ticket");
        super::fs::write(path, "title").unwrap();

        let result_read = ticket.read();
        assert!(result_read.is_ok());
        assert_eq!(ticket.title, Some("title".to_string()));
        assert_eq!(ticket.message, None);
    }

    #[test]
    fn ticket_read_a_title_and_a_message() {
        super::super::purge().unwrap_or(());
        super::super::initialize_root().unwrap_or(());
        let mut ticket = super::Ticket::from("test/new_ticket".to_string(),
                                             None)
            .unwrap();
        ticket.initialize_directory().unwrap();

        let path = super::super::get_path_root()
            .join("test/new_ticket");
        super::fs::write(path, "title\n\nand some content").unwrap();

        let result_read = ticket.read();
        assert!(result_read.is_ok());
        assert_eq!(ticket.title, Some("title".to_string()));
        assert_eq!(ticket.message, Some("and some content".to_string()));
    }

    // ========== ticket.collect ==========
    #[test]
    fn ticket_collect_an_empty_directory() {
        super::super::purge().unwrap_or(());
        super::super::initialize_root().unwrap_or(());
        let path_dir = super::super::get_path_root()
            .join("empty/");
        super::fs::create_dir(path_dir).unwrap();

        let ticket = super::Ticket::from("empty/".to_string(), None)
            .unwrap();

        let tickets = ticket.collect();
        assert!(tickets.is_ok());
        let tickets = tickets.unwrap();
        assert_eq!(tickets.len(), 0);
    }

    #[test]
    fn ticket_collect_a_nonempty_directory() {
        super::super::purge().unwrap_or(());
        super::super::initialize_root().unwrap_or(());
        let millis100 = Duration::from_millis(100);
        let path_root = super::super::get_path_root();
        let path_dir = path_root.join("nonempty/");
        super::fs::create_dir(&path_dir).unwrap();
        let path = path_dir.join("ticket1");
        super::fs::write(path, "ticket no.1\n\nand some content").unwrap();
        thread::sleep(millis100);
        let path = path_dir.join("ticket2");
        super::fs::write(path, "ticket no.2\n\nand some content").unwrap();
        thread::sleep(millis100);
        let path = path_dir.join("ticket3");
        super::fs::write(path, "ticket no.3\n\nand some content").unwrap();

        let ticket = super::Ticket::from("nonempty/".to_string(), None)
            .unwrap();

        let tickets = ticket.collect();
        assert!(tickets.is_ok());
        let tickets = tickets.unwrap();
        assert_eq!(tickets.len(), 3);

        let ticket1 = &tickets[0];
        assert_eq!(ticket1.id, Some(String::from("ticket1")));
        let ticket2 = &tickets[1];
        assert_eq!(ticket2.id, Some(String::from("ticket2")));
        let ticket3 = &tickets[2];
        assert_eq!(ticket3.id, Some(String::from("ticket3")));
    }

    // ========== ticket.move_ ==========
    #[test]
    fn ticket_move_within_category() {
        super::super::purge().unwrap_or(());
        super::super::initialize_root().unwrap_or(());
        let path_root = super::super::get_path_root();
        let path_dir = path_root.join("hello/");
        super::fs::create_dir(&path_dir).unwrap();
        let path = path_dir.join("ticket");
        super::fs::write(path, "ticket to be moved\n\nand some content").unwrap();

        let start_ticket = super::Ticket::from("hello/ticket".to_string(),
                                               None).unwrap();
        let dest_ticket = super::Ticket::from("hello/tucker".to_string(),
                                               None).unwrap();
        let result_move = start_ticket.move_(&dest_ticket);
        assert!(result_move.is_ok());
        assert!(!start_ticket.path.exists());

        let mut dest_ticket = super::Ticket::from("hello/tucker".to_string(),
                                               None).unwrap();
        dest_ticket.read().unwrap();
        assert!(dest_ticket.path.exists());
        assert_eq!(dest_ticket.id, Some("tucker".to_string()));
        assert_eq!(dest_ticket.category, "hello".to_string());
        assert_eq!(dest_ticket.title, Some("ticket to be moved".to_string()));
        assert_eq!(dest_ticket.message, Some("and some content".to_string()));
    }

    #[test]
    fn ticket_move_between_categories() {
        super::super::purge().unwrap_or(());
        super::super::initialize_root().unwrap_or(());
        let path_root = super::super::get_path_root();
        let path_dir_start = path_root.join("hello/");
        super::fs::create_dir(&path_dir_start).unwrap();
        let path_dir_dest = path_root.join("world/");
        super::fs::create_dir(&path_dir_dest).unwrap();
        let path = path_dir_start.join("ticket");
        super::fs::write(path, "ticket to be moved\n\nand some content").unwrap();

        let start_ticket = super::Ticket::from("hello/ticket".to_string(),
                                               None).unwrap();
        let dest_ticket = super::Ticket::from("world/ticket".to_string(),
                                               None).unwrap();
        let result_move = start_ticket.move_(&dest_ticket);
        assert!(result_move.is_ok());
        assert!(!start_ticket.path.exists());
        let mut dest_ticket = super::Ticket::from("world/ticket".to_string(),
                                               None).unwrap();
        dest_ticket.read().unwrap();
        assert!(dest_ticket.path.exists());
        assert_eq!(dest_ticket.id, Some("ticket".to_string()));
        assert_eq!(dest_ticket.category, "world".to_string());
        assert_eq!(dest_ticket.title, Some("ticket to be moved".to_string()));
        assert_eq!(dest_ticket.message, Some("and some content".to_string()));
    }

    // ========== ticket.move_all ==========
    #[test]
    fn ticket_move_all() {
        super::super::purge().unwrap_or(());
        super::super::initialize_root().unwrap_or(());
        let path_root = super::super::get_path_root();
        let path_dir = path_root.join("hello/");
        super::fs::create_dir(&path_dir).unwrap();
        let path = path_dir.join("ticket1");
        super::fs::write(path, "ticket no.1\n\nand some content").unwrap();
        let path = path_dir.join("ticket2");
        super::fs::write(path, "ticket no.2\n\nand some content").unwrap();
        let path = path_dir.join("ticket3");
        super::fs::write(path, "ticket no.3\n\nand some content").unwrap();
        let path_dir = path_root.join("world/");
        super::fs::create_dir(&path_dir).unwrap();

        let start_ticket = super::Ticket::from("hello/".to_string(), None)
            .unwrap();
        let dest_ticket = super::Ticket::from("world/".to_string(), None)
            .unwrap();
        let result_move = start_ticket.move_all(&dest_ticket);
        assert!(result_move.is_ok());
        
        let vec_empty = start_ticket.collect().unwrap();
        assert!(vec_empty.is_empty());

        let vec_moved = dest_ticket.collect().unwrap();
        assert_eq!(vec_moved.len(), 3);
    }

    // ========== ticket.remove ==========
    #[test]
    fn ticket_remove_one() {
        super::super::purge().unwrap_or(());
        super::super::initialize_root().unwrap_or(());
        let path_root = super::super::get_path_root();
        let path_dir = path_root.join("hello/");
        super::fs::create_dir(&path_dir).unwrap();

        let path = path_dir.join("ticket1");
        super::fs::write(path, "ticket title\n\nand some content").unwrap();

        let ticket = super::Ticket::from("hello/ticket1".to_string(), None)
            .unwrap();
        let result_remove = ticket.remove();
        assert!(result_remove.is_ok());

        let ticket_dir = super::Ticket::from("hello/".to_string(), None)
            .unwrap();
        let vec_empty = ticket_dir.collect().unwrap();
        assert!(vec_empty.is_empty());
    }

    #[test]
    fn fail_to_ticket_remove_non_existing_one() {
        super::super::purge().unwrap_or(());
        super::super::initialize_root().unwrap_or(());
        let path_root = super::super::get_path_root();
        let path_dir = path_root.join("hello/");
        super::fs::create_dir(&path_dir).unwrap();

        let ticket = super::Ticket::from("hello/ticket1".to_string(), None)
            .unwrap();
        let result_remove = ticket.remove();
        assert!(result_remove.is_err());
    }

    #[test]
    fn ticket_remove_all() {
        super::super::purge().unwrap_or(());
        super::super::initialize_root().unwrap_or(());
        let path_root = super::super::get_path_root();
        let path_dir = path_root.join("hello/");
        super::fs::create_dir(&path_dir).unwrap();
        let path = path_dir.join("ticket1");
        super::fs::write(path, "ticket1 title\n\nand some content").unwrap();
        let path = path_dir.join("ticket2");
        super::fs::write(path, "ticket2 title\n\nand some content").unwrap();

        let ticket = super::Ticket::from("hello/".to_string(), None)
            .unwrap();
        let result_remove = ticket.remove();
        assert!(result_remove.is_ok());
        assert!(!ticket.path.exists());
    }
}
