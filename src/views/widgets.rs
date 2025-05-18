use ratatui::widgets::ListState;

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        let mut state = ListState::default();
        // Start with the first item selected
        if !items.is_empty() {
            state.select(Some(0));
        }
        StatefulList {
            state,
            items,
        }
    }

    pub fn add_items(&mut self, new_items: Vec<T>) {
        self.items.extend(new_items);
    }

    pub fn sort_by<F, K>(&mut self, f: F)
        where
            F: FnMut(&T) -> K,
            K: Ord,
    {
        self.items.sort_by_key(f);
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    i
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    i
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn first(&mut self) {
        self.state.select(Some(0));
    }

    pub fn last(&mut self) {
        self.state.select(Some(self.items.len()-1));
    }

    pub fn jump_up(&mut self, offset: i16 ) {
        let i = match self.state.selected() {
            Some(i) => {
                let result = (i as i16) - offset;
                if result < 0 {
                    0
                } else {
                    result as usize
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn jump_down(&mut self, offset: i16 ) {
        let i = match self.state.selected() {
            Some(i) => {
                let result = (i as i16) + offset;
                if result > self.items.len().try_into().unwrap() {
                    self.items.len() - 1
                } else {
                    result as usize
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}
