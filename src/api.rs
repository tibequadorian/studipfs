use serde::Deserialize;


#[derive(Deserialize, Debug)]
pub struct Folder {
    pub subfolders: Vec<Subfolder>,
    pub file_refs: Vec<FileRef>,
    #[serde(flatten)]
    pub folder: Subfolder,
}

#[derive(Deserialize, Debug)]
pub struct Subfolder {
    pub id: String,
    pub user_id: String,
    pub parent_id: String,
    pub range_id: String,
    pub range_type: String,
    pub folder_type: String,
    pub name: String,
    pub description: String,
    pub mkdate: i32,
    pub chdate: i32,
    pub is_visible: bool,
    pub is_readable: bool,
    pub is_writable: bool,
}

#[derive(Deserialize, Debug)]
pub struct FileRef {
    pub id: String,
    pub file_id: String,
    pub folder_id: String,
    pub downloads: u32,
    pub description: String,
    pub content_terms_of_use_id: String,
    pub user_id: String,
    pub name: String,
    pub mkdate: i32,
    pub chdate: i32,
    pub size: u32,
    pub mime_type: String,
    pub storage: String,
    pub is_readable: bool,
    pub is_downloadable: bool,
    pub is_editable: bool,
    pub is_writable: bool,
}

pub struct StudIPClient {
    pub api_url: String,
    pub auth: String,
}

impl StudIPClient {
    fn get<T: Into<minreq::URL>>(&self, url: T) -> minreq::Request {
        minreq::get(url).with_header("Authorization", &self.auth)
    }

    pub fn read_file(&self, id: &String) -> Result<Vec<u8>, minreq::Error> {
        let url = format!("{}/file/{}/download", self.api_url, id);
        return Ok(self.get(url).send()?.into_bytes());
    }
    
    pub fn get_folder(&self, id: &String) -> Result<Folder, minreq::Error> {
        let url = format!("{}/folder/{}", self.api_url, id);
        return self.get(url).send()?.json();
    }
}
