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

    /// Shrinks the current half‑open search range by keeping either the front `[start, middle)` or back `[middle + 1, end)` portion.
    fn bisect(&mut self, half_to_keep: Half) {
        let middle = self.search_range_middle();

        match half_to_keep {
            Half::Front => self.search_range.end = middle,
            Half::Back => self.search_range.start = (middle + 1).min(self.search_range.end),
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

    /// Performs a sorting step, discarding the half of the search range opposite the given step direction
    /// returns true if the range has collapsed and hence the element has been placed
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

#[cfg(test)]
mod tests {
    use super::*;

    // SearchSpace::to_range
    #[test]
    fn full_range_covers_all() {
        let length = 10;
        let range = SearchSpace::Full.to_range(3, length);
        assert_eq!(range, 0..length);
    }

    #[test]
    fn left_range_excludes_selection() {
        let length = 10;
        let selection = 4;
        let range = SearchSpace::Left.to_range(selection, length);
        assert_eq!(range, 0..selection);
    }

    #[test]
    fn right_range_excludes_selection() {
        let length = 10;
        let selection = 4;
        let range = SearchSpace::Right.to_range(selection, length);
        assert_eq!(range, (selection + 1)..length);
    }

    #[test]
    fn right_range_clamps_at_length() {
        let length = 10;
        let selection = 11;
        let range = SearchSpace::Right.to_range(selection, length);
        assert_eq!(range, length..length);
    }

    // select
    #[test]
    fn select_updates_selection_and_range() {
        let mut sorting = Sorting::new(vec![1, 2, 3, 4]);
        sorting.set_search_space(SearchSpace::Left);
        sorting.select(2).unwrap();
        assert_eq!(sorting.selection(), Some(&3));
        assert_eq!(sorting.search_range, 0..2);
    }

    #[test]
    fn select_rejects_out_of_bounds() {
        let mut sorting = Sorting::new(vec![1, 2, 3]);
        assert!(sorting.select(10).is_err());
    }

    // candidate
    #[test]
    fn candidate_is_middle_of_range() {
        let mut sorting = Sorting::new(vec![10, 20, 30, 40, 50]);
        sorting.set_search_space(SearchSpace::Full);
        assert_eq!(sorting.candidate(), Some(&30));
    }

    #[test]
    fn candidate_never_returns_selection() {
        let mut sorting = Sorting::new(vec![10, 20, 30, 40, 50]);
        sorting.set_search_space(SearchSpace::Full);
        sorting.select(2).unwrap(); // selection = 30 
        assert_ne!(sorting.candidate(), Some(&30));
    }

    // move_selection
    #[test]
    fn move_selection_rotates_left_segment() {
        let mut sorting = Sorting::new(vec![1, 2, 3, 4]);
        sorting.select(2).unwrap(); // selecting 3 
        sorting.move_selection(0);
        assert_eq!(
            sorting.iter().cloned().collect::<Vec<_>>(),
            vec![3, 1, 2, 4]
        );
        assert_eq!(sorting.selection(), Some(&3));
    }

    #[test]
    fn move_selection_rotates_right_segment() {
        let mut sorting = Sorting::new(vec![1, 2, 3, 4]);
        sorting.select(1).unwrap(); // selecting 2 
        sorting.move_selection(3);
        assert_eq!(
            sorting.iter().cloned().collect::<Vec<_>>(),
            vec![1, 3, 4, 2]
        );
        assert_eq!(sorting.selection(), Some(&2));
    }

    // step
    #[test]
    fn step_front_halves_range() {
        let mut sorting = Sorting::new(vec![1, 2, 3, 4, 5]);
        sorting.set_search_space(SearchSpace::Full);
        sorting.step(Half::Front);
        assert_eq!(sorting.search_range, 0..2); // middle of 0..5 is 2 
    }

    #[test]
    fn step_back_halves_range() {
        let mut sorting = Sorting::new(vec![1, 2, 3, 4, 5]);
        sorting.set_search_space(SearchSpace::Full);
        sorting.step(Half::Back);
        assert_eq!(sorting.search_range, 3..5);
    }

    // Search space configuration
    #[test]
    fn setting_search_space_recomputes_range() {
        let mut sorting = Sorting::new(vec![10, 20, 30, 40]);
        sorting.select(2).unwrap();
        sorting.set_search_space(SearchSpace::Left);
        assert_eq!(sorting.search_range, 0..2);
        sorting.set_search_space(SearchSpace::Right);
        assert_eq!(sorting.search_range, 3..4);
    }

    // Invariants
    #[test]
    fn selection_out_of_bounds_errors() {
        let mut sorting = Sorting::new(vec![1, 2, 3, 4, 5]);
        assert!(sorting.select(sorting.elements.len()).is_err())
    }

    #[test]
    fn search_range_is_always_valid() {
        let mut sorting = Sorting::new(vec![1, 2, 3, 4]);
        for mode in [SearchSpace::Full, SearchSpace::Left, SearchSpace::Right] {
            sorting.set_search_space(mode.clone());
            let range = &sorting.search_range;
            assert!(range.start <= range.end);
            assert!(range.end <= sorting.elements.len());
        }
    }

    // Integration
    #[test]
    fn step_until_done_places_element_correctly() {
        let mut sorting = Sorting::new(vec![5, 1, 3, 4, 2]);
        sorting.select(0).unwrap(); // selecting 5 
        sorting.set_search_space(SearchSpace::Full);
        while !sorting.step(Half::Back) {}
        assert_eq!(
            sorting.iter().cloned().collect::<Vec<_>>(),
            vec![1, 3, 4, 2, 5]
        );
    }

    #[test]
    fn exact_steps_even_number_of_elements() {
        let mut sorting = Sorting::new(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        sorting.set_search_space(SearchSpace::Full);
        sorting.select(7);
        let mut done = sorting.step(Half::Front);
        assert!(!done);
        done = sorting.step(Half::Back);
        assert!(!done);
        done = sorting.step(Half::Front);
        assert!(!done);
        done = sorting.step(Half::Front);
        assert!(done);
        assert_eq!(
            sorting.iter().cloned().collect::<Vec<_>>(),
            vec![0, 1, 2, 7, 3, 4, 5, 6, 8, 9]
        );
    }

    #[test]
    fn exact_steps_uneven_number_of_elements() {
        let mut sorting = Sorting::new(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        sorting.set_search_space(SearchSpace::Full);
        sorting.select(7);
        let mut done = sorting.step(Half::Front);
        assert!(!done);
        done = sorting.step(Half::Back);
        assert!(!done);
        done = sorting.step(Half::Front);
        assert!(!done);
        done = sorting.step(Half::Front);
        assert!(done);
        assert_eq!(
            sorting.iter().cloned().collect::<Vec<_>>(),
            vec![0, 1, 2, 7, 3, 4, 5, 6, 8, 9, 10]
        );
    }

    #[test]
    fn step_then_select_next_element() {
        let mut sorting = Sorting::new(vec![3, 1, 2]);
        sorting.select(0).unwrap();
        while !sorting.step(Half::Back) {}
        assert_eq!(sorting.selection(), Some(&1)); // next element after placing 3 
    }

    #[test]
    fn placing_at_end_in_right_space_keeps_selection_in_range() {
        let mut sorting = Sorting::new(vec![3, 1, 2]);
        sorting.set_search_space(SearchSpace::Right);
        sorting.select(0).unwrap(); // selecting 3
        while !sorting.step(Half::Back) {}

        // Assert: selection index is valid
        assert!(
            sorting.selection < sorting.elements.len(),
            "selection index {} is out of range {}",
            sorting.selection,
            sorting.elements.len()
        );

        // Assert: selection() returns a valid element
        assert!(
            sorting.selection().is_some(),
            "selection() unexpectedly returned None"
        );
    }
}
