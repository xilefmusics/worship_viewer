use serde::{Deserialize, Serialize};

use super::ListQuery;

/// Query parameters for `GET /api/v1/songs`: pagination plus optional sort and filters.
///
/// Pagination fields mirror [`ListQuery`] (they are not `flatten`ed so `actix_web::Query`
/// deserializes reliably from `application/x-www-form-urlencoded` query strings).
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct SongListQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub q: Option<String>,
    /// Sort order. With a non-empty `q`, defaults to `relevance` (search score). Without `q`, defaults to `id_desc`.
    pub sort: Option<SongSort>,
    /// Filter to songs whose `data.languages` contains this string (exact match on an array element).
    pub lang: Option<String>,
    /// Case-insensitive substring match against the stringified `data.tags` object (keys and values).
    pub tag: Option<String>,
}

/// Allowed values for the `sort` query parameter on `GET /api/v1/songs`.
#[cfg_attr(feature = "backend", derive(utoipa::ToSchema))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SongSort {
    /// Newest record id first (default when `q` is absent).
    #[default]
    IdDesc,
    IdAsc,
    TitleAsc,
    TitleDesc,
    /// Search relevance (default when `q` is present); uses full-text scores.
    Relevance,
}

impl From<ListQuery> for SongListQuery {
    fn from(list: ListQuery) -> Self {
        Self {
            page: list.page,
            page_size: list.page_size,
            q: list.q,
            sort: None,
            lang: None,
            tag: None,
        }
    }
}

impl SongListQuery {
    /// Same pagination semantics as [`ListQuery`].
    pub fn list_query(&self) -> ListQuery {
        ListQuery {
            page: self.page,
            page_size: self.page_size,
            q: self.q.clone(),
        }
    }

    /// Validates pagination ([`ListQuery::validate`]) and sort vs `q` rules.
    pub fn validate(self) -> Result<Self, String> {
        self.list_query().validate()?;
        let q_nonempty = self
            .q
            .as_ref()
            .is_some_and(|q| !q.trim().is_empty());
        if matches!(self.sort, Some(SongSort::Relevance)) && !q_nonempty {
            return Err("sort=relevance requires a non-empty q parameter".into());
        }
        Ok(self)
    }

    /// Serialize as a query string (for API clients). Pagination uses [`ListQuery::to_query_string`];
    /// adds `sort`, `lang`, and `tag` when set.
    pub fn to_query_string(&self) -> String {
        fn enc(s: &str) -> String {
            let mut out = String::with_capacity(s.len());
            for c in s.chars() {
                match c {
                    ' ' => out.push_str("%20"),
                    '&' => out.push_str("%26"),
                    '=' => out.push_str("%3D"),
                    '%' => out.push_str("%25"),
                    '+' => out.push_str("%2B"),
                    c => out.push(c),
                }
            }
            out
        }

        let mut q = self.list_query().to_query_string();
        let append = |q: &mut String, k: &str, v: &str| {
            if q.is_empty() {
                q.push('?');
            } else if !q.contains('?') {
                q.insert(0, '?');
            } else {
                q.push('&');
            }
            q.push_str(k);
            q.push('=');
            q.push_str(&enc(v));
        };

        if let Some(sort) = self.sort {
            let s = match sort {
                SongSort::IdDesc => "id_desc",
                SongSort::IdAsc => "id_asc",
                SongSort::TitleAsc => "title_asc",
                SongSort::TitleDesc => "title_desc",
                SongSort::Relevance => "relevance",
            };
            append(&mut q, "sort", s);
        }
        if let Some(ref lang) = self.lang {
            if !lang.is_empty() {
                append(&mut q, "lang", lang);
            }
        }
        if let Some(ref tag) = self.tag {
            if !tag.is_empty() {
                append(&mut q, "tag", tag);
            }
        }
        q
    }

    /// Effective sort: explicit `sort`, or inferred from presence of `q`.
    pub fn effective_sort(&self) -> SongSort {
        match self.sort {
            Some(s) => s,
            None => {
                let q_nonempty = self
                    .q
                    .as_ref()
                    .is_some_and(|q| !q.trim().is_empty());
                if q_nonempty {
                    SongSort::Relevance
                } else {
                    SongSort::IdDesc
                }
            }
        }
    }
}
