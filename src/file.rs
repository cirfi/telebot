//! A Telegram file which contains a readable source and a filename
//!
//! The filename should be such that it represents the content type.

use std::{io::Read, convert::TryFrom, path::PathBuf};
use failure::Error;
use error::ErrorKind;

#[derive(Serialize)]
#[serde(untagged)]
pub enum MediaFile {
    SingleFile(String),
    MultipleFiles(Vec<FileEntity>)
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum FileEntity {
    Photo {
        #[serde(rename = "type")]
        type_: &'static str,
        media: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        caption: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        parse_mode: Option<String>,
    },
    Video {}
}

pub struct FileList(pub Vec<FileWithCaption>);

impl FileList {
    pub fn to_metadata(&self) -> Option<MediaFile> {
        if self.0.len() == 0 {
            None
        } else if self.0.len() == 1 {
            //Some(MediaFile::SingleFile(self.0.iter().map(|x| x.file.name()).next().unwrap()))
            None
        } else {
            let entities = self.0.iter().map(|x| {
                FileEntity::Photo {
                    type_: "photo",
                    media: x.file.name(),
                    caption: x.caption.clone(),
                    parse_mode: x.parse_mode.clone()
                }
            }).collect();

            Some(MediaFile::MultipleFiles(entities))
        }
    }

    pub fn into_files(self) -> Option<Vec<File>> {
        if self.0.len() == 0 {
            None
        } else {
            Some(self.0.into_iter().map(|x| x.file).collect())
        }
    }

    pub fn push(&mut self, val: FileWithCaption) {
        self.0.push(val);
    }
}

/// A Telegram file which contains a readable source and a filename
pub enum File {
    Memory {
        name: String,
        source: Box<Read + Send>,
    },
    Disk {
        path: PathBuf
    },
    Telegram(String),
    Url(String)
}

impl File {
    pub fn name(&self) -> String {
        match self {
            File::Memory { name, .. } => format!("attach://{}", name),
            File::Disk { path, .. } => format!("attach://{}", path.file_name().unwrap().to_str().unwrap()),
            File::Telegram(id) => id.clone(),
            File::Url(url) => url.clone()
        }
    }
}

pub struct FileWithCaption {
    file: File,
    caption: Option<String>,
    parse_mode: Option<String>
}

impl FileWithCaption {
    pub fn new_empty(file: File) -> FileWithCaption {
        FileWithCaption {
            file: file,
            caption: None,
            parse_mode: None
        }
    }

    pub fn new(file: File, caption: String, parse_mode: String) -> FileWithCaption {
        FileWithCaption {
            file: file,
            caption: Some(caption),
            parse_mode: Some(parse_mode)
        }
    }
}
/// Construct a Telegram file from a local path
impl<'a> TryFrom<&'a str> for File {
    type Error = Error;

    fn try_from(path: &'a str) -> Result<Self, Self::Error> {
        let mut file = PathBuf::new();

        file.push(path);

        if file.is_file() {
            Ok(File::Disk { path: file })
        } else {
            Err(Error::from(ErrorKind::NoFile))
        }
    }
}

/// Construct a Telegram file from an object which implements the Read trait
impl<'a, S: Read + Send + 'static> TryFrom<(&'a str, S)> for File {
    type Error = Error;

    fn try_from((name, source): (&'a str, S)) -> Result<Self, Self::Error>
    where
        S: Read + Send,
    {
        Ok(File::Memory {
            name: name.into(),
            source: Box::new(source),
        })
    }
}
