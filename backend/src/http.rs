use axum::extract::Query;
use dormtk_api_types::{ApiResponse, PageMeta, PageResponse};
use serde::Deserialize;

pub type ApiJson<T> = axum::Json<ApiResponse<T>>;
pub type PageJson<T> = axum::Json<PageResponse<T>>;

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Pagination {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

impl Pagination {
    pub fn normalized(self) -> NormalizedPagination {
        let page = self.page.unwrap_or(1).max(1);
        let page_size = self.page_size.unwrap_or(20).clamp(1, 200);

        NormalizedPagination { page, page_size }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NormalizedPagination {
    pub page: u32,
    pub page_size: u32,
}

impl NormalizedPagination {
    pub fn limit(self) -> i64 {
        self.page_size as i64
    }

    pub fn offset(self) -> i64 {
        ((self.page - 1) * self.page_size) as i64
    }

    pub fn meta(self, total: i64) -> PageMeta {
        PageMeta {
            page: self.page,
            page_size: self.page_size,
            total: total.max(0) as u64,
        }
    }
}

pub fn data<T>(data: T) -> ApiJson<T> {
    axum::Json(ApiResponse { data })
}

pub fn page<T>(items: Vec<T>, pagination: NormalizedPagination, total: i64) -> PageJson<T> {
    axum::Json(PageResponse {
        data: items,
        meta: pagination.meta(total),
    })
}

pub fn pagination(Query(query): Query<Pagination>) -> NormalizedPagination {
    query.normalized()
}
