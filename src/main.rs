use libc::ENOENT;
use std::ffi::OsStr;
use std::time::{Duration, UNIX_EPOCH};
use std::collections::HashMap;
use fuse::{Filesystem, Request, ReplyData, ReplyEntry, ReplyAttr, ReplyDirectory, FileAttr, FileType, FUSE_ROOT_ID};
use ttl_cache::TtlCache;

mod api;
mod logins;


const TTL: Duration = Duration::from_secs(60);

type ID = String;

struct StudIPFolder {
    children: Vec<ID>,
}

struct StudIPFile {

}

struct StudIPEntry {
    id: ID,
    parent: ID,
    name: String,
    size: u64,
    kind: StudIPEntryType,
}

enum StudIPEntryType {
    Folder(StudIPFolder),
    File(StudIPFile),
}

impl StudIPEntry {
    fn from_folder(folder: &api::Folder) -> StudIPEntry {
        StudIPEntry {
            id: folder.folder.id.clone(),
            parent: folder.folder.parent_id.clone(),
            name: folder.folder.name.clone(),
            size: 0,
            kind: StudIPEntryType::Folder(StudIPFolder {
                children: folder.subfolders.iter().map(|f| f.id.clone())
                    .chain(folder.file_refs.iter().map(|f| f.id.clone())).collect(),
            }),
        }
    }

    fn from_file(file: &api::FileRef) -> StudIPEntry {
        StudIPEntry {
            id: file.id.clone(),
            parent: file.folder_id.clone(),
            name: file.name.clone(),
            size: file.size as u64,
            kind: StudIPEntryType::File(StudIPFile {}),
        }
    }
}

struct StudIPFS {
    client: api::StudIPClient,
    inodes: HashMap<ID, u64>,
    entries: HashMap<u64, StudIPEntry>,
    cache: TtlCache<u64, Vec<u8>>,
    next_ino: u64,
}

impl StudIPFS {
    fn new(client: api::StudIPClient, root: ID) -> StudIPFS {
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

    fn get_from_id(&self, id: &ID) -> &StudIPEntry {
        self.entries.get(self.inodes.get(id).unwrap()).unwrap()
    }

    fn add(&mut self, entry: StudIPEntry) {
        self.inodes.insert(entry.id.clone(), self.next_ino);
        self.entries.insert(self.next_ino, entry);
        self.next_ino += 1;
    }

    fn populate(&mut self, id: ID) {
        let folder = self.client.get_folder(&id).unwrap();
        self.add(StudIPEntry::from_folder(&folder));
        for subfolder in folder.subfolders {
            self.populate(subfolder.id);
        }
        for file in folder.file_refs {
            self.add(StudIPEntry::from_file(&file));
        }
    }

    fn get_attr(&self, entry: &StudIPEntry) -> FileAttr {
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
                StudIPEntryType::Folder(_) => FileType::Directory,
                StudIPEntryType::File(_)   => FileType::RegularFile,
            },
            perm: 0o555,
            nlink: 0,
            uid: 1000,
            gid: 1000,
            rdev: 0,
            flags: 0,
        }
    }
}

impl Filesystem for StudIPFS {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        if let Some(StudIPEntry { kind: StudIPEntryType::Folder(StudIPFolder { children }), .. }) = self.entries.get(&parent) {
            match children.iter().map(|id| self.get_from_id(id)).find(|e| e.name.as_str() == name) {
                Some(e) => reply.entry(&TTL, &self.get_attr(e), 0),
                None => reply.error(ENOENT),
            };
        } else {
            panic!(); // shouldn't happen
        }
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        if let Some(entry) = self.entries.get(&ino) {
            reply.attr(&TTL, &self.get_attr(&entry));
        } else {
            panic!(); // shouldn't happen
        }
    }

    fn read(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, size: u32, reply: ReplyData) {
        if let Some(StudIPEntry { id, kind: StudIPEntryType::File(_), .. }) = self.entries.get(&ino) {
	        println!("read({}, offset={} size={})", ino, offset, size);
	        if (!self.cache.contains_key(&ino)) {
		        self.cache.insert(ino, self.client.read_file(id).unwrap(), TTL);
	        }
	        let data = self.cache.get(&ino).unwrap();
            let end = std::cmp::min(offset as usize+size as usize, data.len());
            reply.data(&data[offset as usize..end]);
        } else {
            panic!(); // shouldn't happen
        }
    }

    fn readdir(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, mut reply: ReplyDirectory) {
        if let Some(StudIPEntry { kind: StudIPEntryType::Folder(StudIPFolder { children }), .. }) = self.entries.get(&ino) {
            let entries: Vec<&StudIPEntry> = children.iter().map(|id| self.get_from_id(id)).collect();
            for (i, e) in entries.into_iter().enumerate().skip(offset as usize) {
                reply.add(*self.inodes.get(&e.id).unwrap(), (i + 1) as i64, self.get_attr(e).kind, e.name.as_str());
            }
            reply.ok();
        } else {
            panic!(); // shouldn't happen
        }
    }
}

fn main() -> Result<(), minreq::Error> {
    let client = api::StudIPClient {
        api_url: String::from(logins::API_URL),
        auth: String::from(logins::AUTH)
    };
    let root = String::from(logins::ROOT);
    let fs = StudIPFS::new(client, root);

    // test
    let mountpoint = "./mnt";
    let options = vec![];
    fuse::mount(fs, mountpoint, &options).unwrap();

    Ok(())
}
