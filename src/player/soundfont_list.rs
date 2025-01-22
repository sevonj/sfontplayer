use std::path::PathBuf;

use super::{playlist::FontMeta, PlayerError};

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

#[derive(Debug, Default, Clone)]
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

    pub fn add(&mut self, font: FontMeta) -> Result<(), PlayerError> {
        if self.contains(&font.get_path()) {
            return Err(PlayerError::FontAlreadyExists);
        }
        self.fonts.push(font);
        Ok(())
    }

    /// Remove font - safe for iteration.
    /// See also: `remove_marked`
    pub fn mark_for_removal(&mut self, index: usize) -> Result<(), PlayerError> {
        if index >= self.fonts.len() {
            return Err(PlayerError::FontIndex { index });
        }
        self.fonts[index].marked_for_removal = true;
        Ok(())
    }

    /// Remove all fonts marked for removal.
    pub fn remove_marked(&mut self) {
        for i in (0..self.fonts.len()).rev() {
            if !self.fonts[i].marked_for_removal {
                continue;
            }
            self.fonts.remove(i);

            // Check if deletion affected index
            if let Some(current) = self.selected {
                match i {
                    deleted if deleted == current => self.selected = None,
                    deleted if deleted < current => self.selected = Some(current - 1),
                    _ => (),
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.fonts.clear();
        self.selected = None;
    }

    pub fn contains(&self, filepath: &PathBuf) -> bool {
        for i in 0..self.fonts.len() {
            if self.fonts[i].get_path() == *filepath {
                return true;
            }
        }
        false
    }

    pub const fn get_fonts(&self) -> &Vec<FontMeta> {
        &self.fonts
    }

    pub fn get_font(&self, index: usize) -> Result<&FontMeta, PlayerError> {
        if index >= self.fonts.len() {
            return Err(PlayerError::FontIndex { index });
        }
        Ok(&self.fonts[index])
    }

    pub fn get_font_mut(&mut self, index: usize) -> Result<&mut FontMeta, PlayerError> {
        if index >= self.fonts.len() {
            return Err(PlayerError::FontIndex { index });
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

    pub fn set_selected_index(&mut self, value: Option<usize>) -> Result<(), PlayerError> {
        let Some(index) = value else {
            self.selected = None;
            return Ok(());
        };
        if index >= self.fonts.len() {
            return Err(PlayerError::FontIndex { index });
        }
        self.selected = Some(index);
        self.fonts[index].refresh();
        Ok(())
    }

    pub const fn get_sort(&self) -> FontSort {
        self.sort
    }

    pub fn set_sort(&mut self, sort: FontSort) {
        self.sort = sort;
        self.sort();
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_removal_happens_at_correct_place() {
        let mut font_list = FontList::default();
        assert_eq!(font_list.get_fonts().len(), 0);
        println!("{font_list:?}");

        font_list.add(FontMeta::new("FakeFont".into())).unwrap();
        assert_eq!(font_list.get_fonts().len(), 1);
        println!("{font_list:?}");

        font_list.mark_for_removal(0).unwrap();
        assert_eq!(font_list.get_fonts().len(), 1);
        println!("{font_list:?}");

        font_list.remove_marked();
        assert_eq!(font_list.get_fonts().len(), 0);
        println!("{font_list:?}");
    }
}
