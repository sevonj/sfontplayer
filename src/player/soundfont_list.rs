use core::{error, fmt};
use std::path::PathBuf;

use serde::Serialize;

use super::workspace::font_meta::FontMeta;

#[derive(PartialEq, Eq, Default, Clone, Copy, Debug)]
#[repr(u8)]
pub enum FontSort {
    #[default]
    NameAsc = 0,
    NameDesc = 1,
    SizeAsc = 2,
    SizeDesc = 3,
}
impl TryFrom<u8> for FontSort {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == Self::NameAsc as u8 => Ok(Self::NameAsc),
            x if x == Self::NameDesc as u8 => Ok(Self::NameDesc),
            x if x == Self::SizeAsc as u8 => Ok(Self::SizeAsc),
            x if x == Self::SizeDesc as u8 => Ok(Self::SizeDesc),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum FontListError {
    AlreadyExists,
    IndexOutOfRange,
}
impl error::Error for FontListError {}
impl fmt::Display for FontListError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyExists => {
                write!(f, "This soundfont already exists in the list.")
            }
            Self::IndexOutOfRange => {
                write!(f, "FontList index out of range.")
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct FontList {
    fonts: Vec<FontMeta>,
    sort: FontSort,
    selected: Option<usize>,
}

impl FontList {
    pub fn sort(&mut self) {
        // Store the selected
        let selected = if let Some(index) = self.selected {
            Some(self.fonts[index].clone())
        } else {
            None
        };
        // Sort
        match self.sort {
            FontSort::NameAsc => self.fonts.sort_by_key(|f| f.get_name().to_lowercase()),
            FontSort::NameDesc => {
                self.fonts.sort_by_key(|f| f.get_name().to_lowercase());
                self.fonts.reverse();
            }
            FontSort::SizeAsc => self.fonts.sort_by_key(FontMeta::get_size),
            FontSort::SizeDesc => {
                self.fonts.sort_by_key(FontMeta::get_size);
                self.fonts.reverse();
            }
        };
        // Find the selected again
        if let Some(selected) = selected {
            for i in 0..self.fonts.len() {
                if self.fonts[i].get_path() == selected.get_path() {
                    self.selected = Some(i);
                }
            }
        }
    }
    pub const fn get_sort(&self) -> FontSort {
        self.sort
    }
    pub fn set_sort(&mut self, sort: FontSort) {
        self.sort = sort;
        self.sort();
    }
    pub fn contains(&self, filepath: &PathBuf) -> bool {
        for i in 0..self.fonts.len() {
            if self.fonts[i].get_path() == *filepath {
                return true;
            }
        }
        false
    }
    pub fn add(&mut self, font: FontMeta) -> Result<(), FontListError> {
        if self.contains(&font.get_path()) {
            return Err(FontListError::AlreadyExists);
        }
        self.fonts.push(font);
        Ok(())
    }
    pub fn remove(&mut self, index: usize) -> Result<(), FontListError> {
        if index >= self.fonts.len() {
            return Err(FontListError::IndexOutOfRange);
        }
        self.fonts.remove(index);
        Ok(())
    }
    pub fn clear(&mut self) {
        self.fonts.clear();
    }
    pub const fn get_fonts(&self) -> &Vec<FontMeta> {
        &self.fonts
    }
    pub fn get_font(&self, index: usize) -> Result<&FontMeta, FontListError> {
        if index >= self.fonts.len() {
            return Err(FontListError::IndexOutOfRange);
        }
        Ok(&self.fonts[index])
    }
    pub fn get_font_mut(&mut self, index: usize) -> Result<&mut FontMeta, FontListError> {
        if index >= self.fonts.len() {
            return Err(FontListError::IndexOutOfRange);
        }
        Ok(&mut self.fonts[index])
    }
    pub fn get_selected(&self) -> Option<&FontMeta> {
        let index = self.selected?;
        Some(&self.fonts[index])
    }
    pub fn get_selected_mut(&mut self) -> Option<&mut FontMeta> {
        let index = self.selected?;
        Some(&mut self.fonts[index])
    }
    pub const fn get_selected_index(&self) -> Option<usize> {
        self.selected
    }
    pub fn select(&mut self, value: Option<usize>) -> Result<(), FontListError> {
        let Some(index) = value else {
            self.selected = None;
            return Ok(());
        };
        if index >= self.fonts.len() {
            return Err(FontListError::IndexOutOfRange);
        }
        self.selected = Some(index);
        Ok(())
    }
}
