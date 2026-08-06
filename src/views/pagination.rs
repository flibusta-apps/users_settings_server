use serde::{Deserialize, Serialize};

pub const MAX_PAGE_SIZE: usize = 500;

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
    pub fn validate(&self) -> Result<(), String> {
        if self.page < 1 {
            return Err("page must be greater than or equal to 1".to_string());
        }
        if self.size < 1 {
            return Err("size must be greater than or equal to 1".to_string());
        }
        if self.size > MAX_PAGE_SIZE {
            return Err(format!(
                "size must be less than or equal to {}",
                MAX_PAGE_SIZE
            ));
        }
        Ok(())
    }

    pub fn skip(&self) -> i64 {
        self.page
            .saturating_sub(1)
            .saturating_mul(self.size)
            .try_into()
            .unwrap_or(i64::MAX)
    }

    pub fn take(&self) -> i64 {
        self.size.try_into().unwrap_or(i64::MAX)
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
        let total: usize = items_count.try_into().unwrap_or(0);
        let size = pagination.size.max(1);
        let pages = total.div_ceil(size);

        Self {
            items,
            total,
            page: pagination.page,
            size: pagination.size,
            pages,
        }
    }
}
