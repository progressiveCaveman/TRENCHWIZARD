use rltk::RandomNumberGenerator;

pub struct WeightedEntry {
    name: String,
    weight: i32,
}

impl WeightedEntry {
    pub fn new<S: ToString>(name: S, weight: i32) -> WeightedEntry {
        WeightedEntry {
            name: name.to_string(),
            weight,
        }
    }
}

#[derive(Default)]
pub struct WeightedTable {
    entries: Vec<WeightedEntry>,
    total_weight: i32,
}

impl WeightedTable {
    pub fn new() -> WeightedTable {
        WeightedTable {
            entries: Vec::new(),
            total_weight: 0,
        }
    }

    pub fn add<S: ToString>(mut self, name: S, weight: i32) -> WeightedTable {
        if weight > 0 {
            let entry = WeightedEntry::new(name.to_string(), weight);
            self.entries.push(entry);
            self.total_weight += weight;
        }
        self
    }

    pub fn roll(&self, rng: &mut RandomNumberGenerator) -> Option<String> {
        if self.entries.len() == 0 {
            return None;
        }

        let roll = rng.range(0, self.total_weight);
        let mut running_weight = 0;
        for e in self.entries.iter() {
            running_weight += e.weight;
            if roll < running_weight {
                return Some(e.name.to_owned());
            }
        }

        None
    }
}
