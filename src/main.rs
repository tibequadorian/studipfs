use std::ffi::OsStr;
use fuse::{Filesystem, Request, ReplyData, ReplyEntry, ReplyAttr, ReplyDirectory, FileAttr, FUSE_ROOT_ID};

mod api;
mod logins;

struct Entry {
    id: String,
    attr: Option<EntryAttr>
}

struct EntryAttr {
    name: String,
    kind: EntryType
}

impl Entry {
    fn from_folder(folder: api::Subfolder) -> Entry {
        Entry { id: folder.id, attr: None }
    }
    fn from_file(file: api::FileRef) -> Entry {
        Entry { id: file.id, attr: None }
    }
}

enum EntryType {
    File,
    Folder(Option<Vec<u64>>)
}

struct StudIPFS {
    client: api::StudIPClient,
    inodes: HashMap<u64, Entry>,
    next_ino: u64,
}

impl StudIPFS {
    fn new(client: api::StudIPClient, root: String) -> StudIPFS {
        let mut fs = StudIPFS {
            client,
            inodes: HashMap::new(),
            next_ino: FUSE_ROOT_ID
        };
        fs.add_entry(Entry { id: root, attr: None });
        return fs;
    }

    fn add_entry(&mut self, entry: Entry) {
        self.inodes.insert(self.next_ino, entry);
        self.next_ino += 1;
    }
    
    fn populate(&mut self, ino: u64) {
        let entry = self.inodes.get_mut(&ino).unwrap();
        let folder = self.client.get_folder(&entry.id).unwrap();
        //
        for subfolder in folder.subfolders {
            self.add_entry(Entry::from_folder(subfolder));
        }
        for file in folder.file_refs {
            self.add_entry(Entry::from_file(file));
        }
    }
}

impl Filesystem for StudIPFS {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, _reply: ReplyEntry) {
        println!("lookup({}, {})", parent, name.to_str().unwrap());
        
    }

    fn getattr(&mut self, _req: &Request, ino: u64, _reply: ReplyAttr) {
        println!("getattr({}))", ino);
    }

    fn read(&mut self, _req: &Request, ino: u64, _fh: u64, _offset: i64, _size: u32, _reply: ReplyData) {
        println!("read({})", ino);
    }

    fn readdir(&mut self, _req: &Request, ino: u64, _fh: u64, _offset: i64, mut _reply: ReplyDirectory) {
        println!("readdir({})", ino);
    }
}

use std::collections::HashMap;

fn main() -> Result<(), minreq::Error> {
    let client = api::StudIPClient {
        api_url: String::from(logins::API_URL),
        auth: String::from(logins::AUTH)
    };
    let root = String::from(logins::ROOT);
    let mut fs = StudIPFS::new(client, root);
    fs.populate(1);

    for e in fs.inodes.values() {
        println!("{}", e.id);
    }
    Ok(())
}
