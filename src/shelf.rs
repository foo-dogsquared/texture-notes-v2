use std::fs::{self, DirBuilder, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use globwalk;

use crate::error::Error;
use crate::helpers;
use crate::Object;
use crate::Result;

/// A struct holding the common export options.
#[derive(Debug, Clone)]
pub struct ExportOptions {
    strict: bool,
}

impl ExportOptions {
    /// Creates a new instance of the export options.
    /// By default, all of the options are set to false.
    pub fn new() -> Self {
        Self {
            /// This is used for exporting items to the filesystem.
            /// If the item already exists, it will cause an error.
            strict: false,
        }
    }

    /// Sets the strictness of the export.
    /// This is used when including the items (e.g., subjects, notes) in the database during the creation process.
    pub fn strict(
        &mut self,
        strict: bool,
    ) -> &mut Self {
        self.strict = strict;
        self
    }
}

/// The shelf is where it contains the subjects and its notes.
/// In other words, it is the base directory of the operations taken place in Texture Notes.
#[derive(Debug, Clone)]
pub struct Shelf {
    path: PathBuf,
}

impl Shelf {
    /// Create a new shelf instance.
    pub fn new<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        Self {
            path: path.as_ref().into(),
        }
    }

    /// Creates a shelf instance from the filesystem.
    pub fn from<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let notes_object = Shelf {
            path: path.to_path_buf(),
        };

        if !notes_object.is_valid() {
            return Err(Error::ValueError);
        }

        Ok(notes_object)
    }

    /// Returns the current path of the shelf.
    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }

    /// Sets the path of the shelf.
    /// Returns the old path.
    ///
    /// If the shelf is exported, it will also move the folder in the filesystem.
    pub fn set_path<P: AsRef<Path>>(
        &mut self,
        to: P,
    ) -> Result<PathBuf> {
        let old_path = self.path();
        let new_path = to.as_ref().to_path_buf();

        if self.is_valid() {
            fs::rename(&old_path, &new_path).map_err(Error::IoError)?;
        }

        self.path = new_path;

        Ok(old_path)
    }

    /// Checks if the shelf is valid.
    pub fn is_valid(&self) -> bool {
        self.path.is_dir()
    }

    /// Exports the shelf in the filesystem.
    /// If the shelf has a database, it will also export subjects at the filesystem.
    /// However, notes are not exported due to needing a dynamic output.
    pub fn export(&mut self) -> Result<()> {
        let dir_builder = DirBuilder::new();

        if !self.is_valid() {
            helpers::fs::create_folder(&dir_builder, self.path())?;
        }

        Ok(())
    }
}

/// A trait implementing the shelf operations.
pub trait ShelfItem<S> {
    fn path_in_shelf(
        &self,
        params: S,
    ) -> PathBuf;
    fn is_path_exists(
        &self,
        params: S,
    ) -> bool;
    fn export(
        &self,
        params: S,
    ) -> Result<()>;
    fn delete(
        &self,
        params: S,
    ) -> Result<()>;
}

/// A trait implementing the object with the additional shelf-related data.
pub trait ShelfData<S>: Object + ShelfItem<S> {
    fn data(
        &self,
        params: S,
    ) -> toml::Value;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note::Note;
    use crate::subjects::Subject;
    use tempfile;

    fn tmp_shelf() -> Result<Shelf> {
        let tmp_dir = tempfile::TempDir::new().map_err(Error::IoError)?;
        let shelf = Shelf::from(tmp_dir)?;

        Ok(shelf)
    }

    #[test]
    fn basic_note_usage() -> Result<()> {
        let mut shelf = tmp_shelf()?;
        let export_options = ExportOptions::new();

        assert!(shelf.export().is_ok());

        let test_subject_input: Vec<Subject> = vec!["Calculus", "Algebra", "Algebra/Precalculus"]
            .into_iter()
            .map(|subject| Subject::new(subject))
            .collect();
        let test_note_input: Vec<Note> = vec![
            "Precalculus Quick Review",
            "Introduction to Integrations",
            "Introduction to Limits",
        ]
        .into_iter()
        .map(|note| Note::new(note))
        .collect();

        let created_subjects: Vec<Subject> = test_subject_input
            .into_iter()
            .filter(|subject| subject.export(&shelf).is_ok())
            .collect();
        assert_eq!(created_subjects.len(), 3);

        // let created_notes = shelf.create_notes(
        //     &test_subject_input[0],
        //     &test_note_input,
        //     consts::NOTE_TEMPLATE,
        //     &export_options,
        // );
        // assert_eq!(created_notes.len(), 3);

        // let available_notes = shelf.get_notes(&test_subject_input[0], &test_note_input);
        // assert_eq!(available_notes.len(), 3);

        // let all_available_notes_from_fs = shelf.get_notes_in_fs(
        //     &test_subject_input[0].note_filter(&shelf),
        //     &test_subject_input[0],
        // )?;
        // assert_eq!(all_available_notes_from_fs.len(), 3);

        // let deleted_notes = shelf.delete_notes(&test_subject_input[0], &test_note_input);
        // assert_eq!(deleted_notes.len(), 3);

        // // It became 2 because the algebra subject is deleted along with the precalculus subject.
        // let deleted_subjects = shelf.delete_subjects(&test_subject_input);
        // assert_eq!(deleted_subjects.len(), 2);

        Ok(())
    }

    #[test]
    fn subject_instances_test() -> Result<()> {
        let mut shelf = tmp_shelf()?;

        let export_options: ExportOptions = ExportOptions::new();

        assert!(shelf.export().is_ok());

        let test_subject: Subject = Subject::new("Mathematics");
        assert_eq!(test_subject.is_path_exists(&shelf), false);

        test_subject.export(&shelf)?;
        assert_eq!(test_subject.is_path_exists(&shelf), true);
        Ok(())
    }

    #[test]
    #[should_panic]
    fn invalid_note_export() {
        let note_path = PathBuf::from("./test/invalid/location/is/invalid");
        let mut test_case = Shelf::new(note_path);
        assert!(test_case.export().is_ok());
    }

    #[test]
    #[should_panic]
    fn invalid_note_import() {
        let note_path = PathBuf::from("./this/is/invalid/note/location/it/does/not/exists/lol");

        assert!(Shelf::from(note_path).is_ok())
    }
}
