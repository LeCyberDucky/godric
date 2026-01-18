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

#[derive(Clone, Debug)]
pub struct Sorting<T> {
    elements: Vec<T>,
    selection: usize,
    search_space: std::ops::Range<usize>,
}

impl<T> Default for Sorting<T> {
    fn default() -> Self {
        let elements: Vec<T> = Default::default();
        Self {
            selection: 0,
            search_space: 0..elements.len(),
            elements,
        }
    }
}

impl<T> Sorting<T> {
    pub fn new(elements: Vec<T>) -> Self {
        Self {
            selection: 0,
            search_space: 0..elements.len(),
            elements,
        }
    }

    pub fn selection(&self) -> Option<&T> {
        // We assume that the selection is always in range
        self.elements.get(self.selection)
    }

    pub fn candidate(&self) -> Option<&T> {
        // It shouldn't be possible to end up outside the search space
        let mut index = self.search_space_middle();
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
        self.search_space = 0..self.elements.len();
        Ok(())
    }

    fn search_space_middle(&self) -> usize {
        let std::ops::Range { start, end } = self.search_space;
        let middle = start + (end - start) / 2;
        middle
    }

    fn bisect(&mut self, half_to_keep: Half) {
        let middle = self.search_space_middle();

        match half_to_keep {
            Half::Front => self.search_space.end = middle,
            Half::Back => self.search_space.start = middle + 1,
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

        let done = self.search_space.start == self.search_space.end;
        if done {
            // Placement found! Let's move the selection there!
            let mut target = self.search_space.start;
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
}
