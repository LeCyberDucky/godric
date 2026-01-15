use color_eyre::eyre::Result;

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
    #[error("Index {index} outside range {range:?}")]
    IndexOutOfRange {
        index: usize,
        range: std::ops::Range<usize>,
    },
}

#[derive(Clone, Debug)]
pub enum Half {
    Front,
    Back,
}

#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    strum_macros::Display,
    strum_macros::EnumString,
    strum_macros::EnumIter,
)]
pub enum SearchSpace {
    #[strum(to_string = "Full list")]
    Full,
    #[strum(to_string = "Left of selection")]
    #[default]
    Left,
    #[strum(to_string = "Right of selection")]
    Right,
}

impl SearchSpace {
    fn to_range(&self, selection: usize, length: usize) -> std::ops::Range<usize> {
        match self {
            SearchSpace::Full => 0..length,
            SearchSpace::Left => 0..selection,
            SearchSpace::Right => (selection + 1).min(length)..length,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Sorting<T> {
    elements: Vec<T>,
    selection: usize,
    search_space: SearchSpace,
    search_range: std::ops::Range<usize>,
}

impl<T> Default for Sorting<T> {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl<T> Sorting<T> {
    pub fn new(elements: Vec<T>) -> Self {
        let selection = 0;
        let search_space = SearchSpace::default();
        Self {
            selection,
            search_range: search_space.to_range(selection, elements.len()),
            elements,
            search_space,
        }
    }

    pub fn selection(&self) -> Option<&T> {
        // We assume that the selection is always in range
        self.elements.get(self.selection)
    }

    pub fn candidate(&self) -> Option<&T> {
        // It shouldn't be possible to end up outside the search space
        let mut index = self.search_range_middle();
        if index == self.selection {
            // Avoid comparing the selection to itself
            // Not sure if we should compare to the previous or next neighbour. Guess it doesn't really matter
            index = index.saturating_sub(1);
        }
        self.elements.get(index)
    }

    pub fn get_mut<I>(&mut self, index: I) -> Option<&mut I::Output>
    where
        I: std::slice::SliceIndex<[T]>,
    {
        self.elements.get_mut(index)
    }

    pub fn select(&mut self, id: usize) -> Result<()> {
        if id >= self.elements.len() {
            return Err(Error::IndexOutOfRange {
                index: id,
                range: 0..self.elements.len(),
            }
            .into());
        }
        self.selection = id;
        self.search_range = self.search_space.to_range(id, self.elements.len());
        Ok(())
    }

    fn search_range_middle(&self) -> usize {
        let std::ops::Range { start, end } = self.search_range;
        start + (end - start) / 2
    }

    fn bisect(&mut self, half_to_keep: Half) {
        let middle = self.search_range_middle();

        match half_to_keep {
            Half::Front => self.search_range.end = middle,
            Half::Back => self.search_range.start = middle + 1,
        }
    }

    fn move_selection(&mut self, target: usize) {
        if target < self.selection {
            self.elements[target..=self.selection].rotate_right(1);
        } else if target > self.selection {
            self.elements[self.selection..=target].rotate_left(1);
        }
        self.selection = target;
    }

    pub fn step(&mut self, direction: Half) -> bool {
        self.bisect(direction);

        let done = self.search_range.start == self.search_range.end;
        if done {
            // Placement found! Let's move the selection there!
            let mut target = self.search_range.start;
            let mut selection = self.selection;
            if target > selection {
                // Not sure why this is necessary, but it seems to be
                target -= 1;
            }
            self.move_selection(target);

            // Select element after previous selection
            if target <= selection {
                selection = (selection + 1).min(self.elements.len().saturating_sub(1));
            }
            self.select(selection)
                .expect("It shouldn't be possible to end up with an out of range selection.");
        }

        done
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.elements.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.elements.iter_mut()
    }

    pub fn search_space(&self) -> &SearchSpace {
        &self.search_space
    }

    pub fn set_search_space(&mut self, config: SearchSpace) {
        if config != self.search_space {
            self.search_range = config.to_range(self.selection, self.elements.len());
            self.search_space = config;
        }
    }
}
