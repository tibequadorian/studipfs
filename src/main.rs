use libc::ENOENT;
use std::env;
use std::ffi::OsStr;
use std::time::{Duration, UNIX_EPOCH};
use std::collections::HashMap;
use fuser::{Filesystem, Request, ReplyData, ReplyEntry, ReplyAttr, ReplyDirectory, FileAttr, FileType, FUSE_ROOT_ID};
use ttl_cache::TtlCache;

mod api;


const TTL: Duration = Duration::from_secs(60);

type ID = String;

struct FSEntry {
    id: ID,
    parent: ID,
    name: String,
    size: u64,
    kind: FSEntryType,
}

enum FSEntryType {
    Folder { children: Vec<ID> },
    File,
}

impl FSEntry {
    fn from_folder(folder: &api::Folder) -> FSEntry {
        FSEntry {
            id: folder.folder.id.clone(),
            parent: folder.folder.parent_id.clone(),
            name: folder.folder.name.clone(),
            size: 0,
            kind: FSEntryType::Folder {
                children: folder.subfolders.iter().map(|f| f.id.clone())
                    .chain(folder.file_refs.iter().map(|f| f.id.clone())).collect(),
            },
        }
    }

    fn from_file(file: &api::FileRef) -> FSEntry {
        FSEntry {
            id: file.id.clone(),
            parent: file.folder_id.clone(),
            name: file.name.clone(),
            size: file.size as u64,
            kind: FSEntryType::File,
        }
    }
}

struct StudIPFS {
    client: api::StudIPClient,
    inodes: HashMap<ID, u64>,
    entries: HashMap<u64, FSEntry>,
    cache: TtlCache<u64, Vec<u8>>,
    next_ino: u64,
}

impl StudIPFS {
    fn new(client: api::StudIPClient, root: &ID) -> StudIPFS {
        let mut fs = StudIPFS {
            client,
            inodes: HashMap::new(),
            entries: HashMap::new(),
            cache: TtlCache::new(50),
            next_ino: FUSE_ROOT_ID,
        };
        fs.populate(root);
        println!("fs populated with {} inodes", fs.next_ino-1);
        return fs;
    }

    fn get_from_id(&self, id: &ID) -> &FSEntry {
        self.entries.get(self.inodes.get(id).unwrap()).unwrap()
    }

    fn add(&mut self, entry: FSEntry) {
        self.inodes.insert(entry.id.clone(), self.next_ino);
        self.entries.insert(self.next_ino, entry);
        self.next_ino += 1;
    }

    fn populate(&mut self, id: &ID) {
        let folder = self.client.get_folder(&id).unwrap();
        self.add(FSEntry::from_folder(&folder));
        for subfolder in folder.subfolders {
            self.populate(&subfolder.id);
        }
        for file in folder.file_refs {
            self.add(FSEntry::from_file(&file));
        }
    }

    fn get_attr(&self, entry: &FSEntry) -> FileAttr {
        let ino = *self.inodes.get(&entry.id).unwrap();
        FileAttr {
            ino,
            size: entry.size,
            blocks: 0,
            atime: UNIX_EPOCH,
            mtime: UNIX_EPOCH,
            ctime: UNIX_EPOCH,
            crtime: UNIX_EPOCH,
            kind: match entry.kind {
                FSEntryType::Folder{..} => FileType::Directory,
                FSEntryType::File       => FileType::RegularFile,
            },
            perm: 0o555,
            nlink: 0,
            uid: 1000,
            gid: 1000,
            rdev: 0,
            blksize: 512,
            flags: 0,
        }
    }
}

impl Filesystem for StudIPFS {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        if let Some(FSEntry { kind: FSEntryType::Folder { children }, .. }) = self.entries.get(&parent) {
            match children.iter().map(|id| self.get_from_id(id)).find(|e| e.name.as_str() == name) {
                Some(e) => reply.entry(&TTL, &self.get_attr(e), 0),
                None => reply.error(ENOENT),
            };
        }
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        if let Some(entry) = self.entries.get(&ino) {
            reply.attr(&TTL, &self.get_attr(&entry));
        }
    }

    fn read(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, size: u32, _flags: i32, _lock: Option<u64>, reply: ReplyData) {
        if let Some(FSEntry { id, kind: FSEntryType::File, .. }) = self.entries.get(&ino) {
            if !self.cache.contains_key(&ino) {
                self.cache.insert(ino, self.client.read_file(id).unwrap(), TTL);
            }
            let data = self.cache.get(&ino).unwrap();
            let end: usize = std::cmp::min(offset as usize+size as usize, data.len());
            reply.data(&data[offset as usize..end]);
        }
    }

    fn readdir(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, mut reply: ReplyDirectory) {
        if let Some(FSEntry { parent, kind: FSEntryType::Folder{ children }, .. }) = self.entries.get(&ino) {
            let mut entries = vec![(ino, FileType::Directory, ".")];
            if let Some(parent_ino) = self.inodes.get(parent) {
                entries.push((*parent_ino, FileType::Directory, ".."));
            }
            entries.extend(
                children.iter().map(|id| self.get_from_id(id))
                    .map(|e| (*self.inodes.get(&e.id).unwrap(), self.get_attr(e).kind, e.name.as_str()))
            );
            for (i, e) in entries.iter().enumerate().skip(offset as usize) {
                if reply.add(e.0, (i+1) as i64, e.1, e.2) {
                    break;
                }
            }
            reply.ok();
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: studipfs <folder id> <mountpoint>");
        return;
    }
    let client = api::StudIPClient {
        api_url: env::var("STUDIP_API_URL").unwrap(),
        auth: env::var("STUDIP_TOKEN").unwrap(),
    };
    let fs = StudIPFS::new(client, &args[1]);
    fuser::mount2(fs, &args[2], &vec![]).unwrap();
}
