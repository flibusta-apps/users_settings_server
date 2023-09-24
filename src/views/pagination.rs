use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Pagination {
    #[serde(default = "default_page")]
    pub page: usize,
    #[serde(default = "default_size")]
    pub size: usize,
}

fn default_page() -> usize {
    1
}
fn default_size() -> usize {
    50
}

impl Pagination {
    pub fn skip(&self) -> i64 {
        ((self.page - 1) * self.size).try_into().unwrap()
    }

    pub fn take(&self) -> i64 {
        self.size.try_into().unwrap()
    }
}

#[derive(Serialize)]
pub struct Page<T>
where
    T: Serialize,
{
    pub items: Vec<T>,
    pub total: usize,
    pub page: usize,
    pub size: usize,
    pub pages: usize,
}

impl<T> Page<T>
where
    T: Serialize,
{
    pub fn create(items: Vec<T>, items_count: i64, pagination: Pagination) -> Self {
        Self {
            items,
            total: items_count.try_into().unwrap(),
            page: pagination.page,
            size: pagination.size,
            pages: (items_count as f64 / pagination.size as f64).ceil() as usize,
        }
    }
}
